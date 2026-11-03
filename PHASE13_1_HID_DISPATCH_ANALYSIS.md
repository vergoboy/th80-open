# Phase 13.1 — HID Reception, Checksum and Command Dispatch Analysis

**Scope:** static analysis of the supplied V504 raw firmware, vendor macOS native driver and frontend bundle. No firmware execution, modification, hardware communication, HID enumeration, Feature GET/SET, USB transfer or updater execution was performed.

## 1. Executive findings

1. A firmware command dispatcher exists at file offset `0x19c5e`. It validates a global command buffer using the checksum routine at `0x19c1e`, then selects command handlers by direct comparisons and two compact switch tables. **`CONFIRMED_STATIC` under the candidate Thumb disassembly.**
2. The checksum routine has one direct caller: the dispatcher at `0x19c68`. The dispatcher has two direct callers: `0x187d0` and `0x1c7b0`. No additional direct `BL` references were found in the complete disassembly. Indirect callers remain possible. **`CONFIRMED_STATIC` for direct references; `UNKNOWN` for indirect callers.**
3. The command buffer used by the dispatcher is the pointer literal `0x00021240` (file literal at `0x19ef4`). One caller at `0x187d0` demonstrably copies a framed packet into that buffer before dispatch. **`CONFIRMED_STATIC`.**
4. The upper command dispatch contains a direct `0x87` branch and a compact table beginning at `0x19cda`. The `0x89` table entry resolves to a return-only epilogue at `0x19cf8`, not to a command-specific handler. **`CONFIRMED_STATIC` for the table bytes and target; `STRONG_STATIC_INFERENCE` that this dispatcher has no `0x89` implementation.**
5. A candidate USB setup/class-request handler exists at `0x1cb48`. It recognizes HID-like class requests, but its request table maps `GET_REPORT` (`bRequest=0x01`) and `SET_REPORT` (`bRequest=0x09`) to a common default path, not to dedicated report handlers. The exact setup-handler registration and Feature SET data bridge remain unresolved. **`STRONG_STATIC_INFERENCE` / `UNKNOWN`.**
6. The vendor native driver sends Feature SET with `bRequest=0x09` and Feature GET with `bRequest=0x01`, both using `wValue=0x0300`, `wIndex=0`, and 64 bytes. The candidate firmware request handler does not show corresponding report handlers, while the vendor support record identifies interface 2. The end-to-end compatibility path is therefore not established.
7. The firmware checksum routine and native checksum builder are not byte-for-byte equivalent: the firmware includes byte 0 in its sum and uses byte 0 itself as the span selector; the native builder excludes byte 0 and uses `dangledevtype` to choose the checksum destination.

## 2. Artifact identity and address convention

All original artifacts were rehashed before analysis:

| Artifact | Size | SHA-256 |
|---|---:|---|
| Firmware `X` | 184,608 bytes | `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520` |
| Native `iot_driver` | 6,075,584 bytes | `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b` |
| Frontend `dist/index.js` | 4,370,837 bytes | `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87` |

Firmware path, relative to `th80-open/`:

`../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X`

Native driver and frontend paths remain under `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/`.

The addresses in this report are **raw firmware file offsets**. A candidate mapping `0x01000000 + file_offset` remains an unconfirmed analysis hypothesis. The Thumb decoder assumption is plausible but the exact CPU ISA and physical flash base are not proven. Candidate virtual addresses are therefore illustrative only; for example, offset `0x19c5e` would map to `0x01019c5e` under that hypothesis, with a Thumb-state pointer bit where appropriate.

## 3. Checksum validator and direct call graph

### 3.1 Routine at `0x19c1e`

Disassembly:

```asm
19c1e: ldrb  r2, [r0]       ; selector = buffer[0]
19c22: cmp   r2, #7
19c24: beq   19c3c
19c26: cmp   r2, #8
19c28: beq   19c3c
; other selector: sum buffer[0..6] with wrapping u8 addition
19c38: ldrb  r0, [r0, #7]
; selector 7 or 8: sum buffer[0..7]
19c4a: ldrb  r0, [r0, #8]
19c4c: adds  r0, r1
19c4e: mvns  r0, r0
19c50: lsls  r0, r0, #24
19c52: lsrs  r0, r0, #24
19c54: beq   19c5a
```

Exact behavior:

| `buffer[0]` | Bytes accumulated before final checksum byte | Final checksum byte | Validation |
|---|---|---|---|
| Anything except `7` or `8` | `buffer[0]` through `buffer[6]` | `buffer[7]` | `~(sum(buffer[0..7])) mod 256 == 0` |
| `7` or `8` | `buffer[0]` through `buffer[7]` | `buffer[8]` | `~(sum(buffer[0..8])) mod 256 == 0` |

The accumulator is truncated to 8 bits after each addition. The helper returns a success/failure value through the common epilogue at `0x196e4`. The selector is the first command-buffer byte, not a separate argument.

Classification: `CONFIRMED_STATIC` for the algorithm under the candidate Thumb decode; `UNKNOWN` for the physical firmware ISA/base.

### 3.2 Direct call graph

```text
0x187d0 ─┐
         ├──> dispatcher 0x19c5e ──> checksum validator 0x19c1e (call at 0x19c68)
0x1c7b0 ─┘
```

Reproducible direct-reference search:

```bash
rg -n 'bl\s+0x19c1e|bl\s+0x19c5e' /tmp/th80-X-thumb.dis
```

Observed results:

- `0x19c68`: `BL 0x19c1e` — the only direct checksum-helper caller.
- `0x187d0`: `BL 0x19c5e` — first direct dispatcher caller.
- `0x1c7b0`: `BL 0x19c5e` — second direct dispatcher caller.

No further direct `BL` references were found. This does not exclude function-pointer calls or a second decoder-visible entry path.

### 3.3 Dispatcher buffer and validation order

At `0x19c60–0x19c68`, the dispatcher sets a status byte at candidate RAM address `0x000200d8`, loads the command-buffer pointer from literal `0x19ef4` (`0x00021240`), and calls the checksum validator. If validation fails it branches to the error/exit path at `0x19d54`; otherwise it reads the opcode from `[0x00021240]` and dispatches.

The global pointer value `0x00021240` is an observed literal. Its RAM/peripheral classification is not based on a confirmed memory map.

### 3.4 Caller `0x187d0`: packet copy before dispatch

The preceding routine begins at `0x187b0` and uses:

- source packet buffer pointer: literal at `0x187f0` = `0x000211da`;
- destination command buffer pointer: literal at `0x187f4` = `0x00021240`;
- status flag pointer: literal at `0x187f8` = `0x000200d8`.

It reads the first source byte as a length-like value, subtracts two, copies that many bytes from source offset `+1` into the command buffer starting at offset zero, sets the status flag, then calls the dispatcher at `0x187d0`. This establishes a framed-packet-to-command-buffer path. The upstream transport that populates `0x000211da` is not identified here.

Classification: `CONFIRMED_STATIC` for the copy and call; `UNKNOWN` for the transport and the semantics of the two-byte framing adjustment.

### 3.5 Caller `0x1c7b0`: USB-region callback candidate

The call at `0x1c7b0` lies inside the long function beginning at `0x1c5b6`. A wrapper at `0x1bd54` directly calls `0x1c5b6`. The surrounding region:

- allocates a 64-byte local buffer and passes it to helper `0x1d45e` (`0x1c5b8–0x1c5c2`);
- uses helper routines around `0x1c066`, `0x1c152` and `0x1c274`;
- accesses a peripheral-like address region including literal `0x000f6020`;
- eventually calls the application dispatcher at `0x1c7b0`.

This is a plausible USB/control-transfer-side processing path, but no complete data-pointer trace proves that the received HID Feature SET payload is the exact buffer validated at `0x19c1e`. The `0x1bd54` wrapper has no recovered direct caller, and callback registration is not established.

Classification: `STRONG_STATIC_INFERENCE` that this is a transport/event processing path; `UNKNOWN` for its precise USB endpoint role and the Feature SET-to-command-buffer edge.

## 4. Application command dispatch

The dispatcher begins at `0x19c5e`. It validates the shared command buffer, reads opcode byte zero and then uses direct comparisons plus two compact switch tables serviced by the helper at `0x1d510`.

### 4.1 Lower command table at `0x19c82`

The table is reached after the dispatcher compares the opcode against `0x16` and calls `0x1d510`. Under the helper's byte-entry/halfword-offset convention:

| Opcode | Table entry | Target | Target action |
|---:|---:|---:|---|
| `0x07` | `0x3f` | `0x19d00` | Calls `0x19bb6` |
| `0x08` | `0x42` | `0x19d06` | Calls `0x19afe` |

These are real code-referenced dispatch entries, not isolated byte occurrences.

### 4.2 Direct upper opcode branches

The dispatcher compares opcodes `0x86` and `0x87` directly:

| Opcode | Branch target | Handler call | Classification |
|---:|---:|---:|---|
| `0x86` | `0x19db8` | `0x19614` | `CONFIRMED_STATIC` |
| `0x87` | `0x19dbe` | `0x195ca` | `CONFIRMED_STATIC` |

### 4.3 Upper table at `0x19cda`

For opcodes `0x89` and above, the dispatcher subtracts `0x89` and enters the second compact table through `0x1d510`.

| Opcode | Entry byte | Computed target | Static target behavior |
|---:|---:|---:|---|
| `0x89` | `0x0f` | `0x19cf8` | `POP {r4, pc}` return-only epilogue |
| `0x8a` | `0x78` | `0x19dca` | Calls `0x1956e` |
| `0x8b` | `0x7b` | `0x19dd0` | Calls `0x1954e` |
| `0x8c` | `0x7e` | `0x19dd6` | Calls `0x1951e` |
| `0x8d` | `0x81` | `0x19ddc` | Calls `0x194e0` |
| `0x8e` | `0x09` | `0x19cec` | Branches to the common return path |
| `0x8f` | `0x90` | `0x19dfa` | Enters a state/flag update path |
| `0x90` | `0x60` | `0x19d9a` | Calls `0x196a4` |

The `0x89` entry is particularly significant: within this dispatcher table it does not call a handler and resolves to the return epilogue. This is a `STRONG_STATIC_INFERENCE` that this firmware dispatch path has no `0x89` command-specific implementation. It does not rule out an unrelated alternate command path or a different firmware revision.

The `0x8c` target `0x1951e` constructs a 64-byte response descriptor using request bytes and a pointer calculation. This is consistent with a paged/data response path, but the data's semantic identity is not proven. It must not be called a per-key RGB handler solely because the frontend also defines a generic `0x8c` user-picture constant.

## 5. USB/HID setup request investigation

### 5.1 Candidate setup handler at `0x1cb48`

The routine reads setup packet byte zero, masks it with `0x60`, and distinguishes standard requests from class requests. When the masked type is `0x20`, it reads `bRequest` from setup byte one and uses the `0x1d510` compact switch helper with a table beginning at `0x1cb66`.

The class request table maps:

| HID class `bRequest` | Standard HID name | Target |
|---:|---|---:|
| `0x02` | GET_IDLE | `0x1cc18` |
| `0x03` | GET_PROTOCOL | `0x1cbb4` |
| `0x0a` | SET_IDLE | `0x1cbbc` |
| `0x0b` | SET_PROTOCOL | `0x1cba6` |
| `0x01` | GET_REPORT | `0x1cc24` default path |
| `0x09` | SET_REPORT | `0x1cc24` default path |

The default path at `0x1cc24` calls `0x1cdd4` and returns value `2`. The precise meaning of that return value is not fully decoded, but it is not a dedicated GET_REPORT or SET_REPORT data handler in this table.

Classification: `STRONG_STATIC_INFERENCE` that this is a USB standard/class setup-request handler and that the listed class requests are dispatched as shown. `UNKNOWN` whether it is the only handler registered for the vendor HID interface.

### 5.2 Descriptor handling and correction to the earlier lead

The standard-request branch beginning at `0x1cc2e` handles `bRequest=0x06` and tests descriptor type values `0x22` and `0x21`, consistent with HID report and HID class descriptors. It selects source pointers from a table of RAM-like addresses.

**Correction:** file offset `0x1cd08` is not itself the HID descriptor. It is a literal-pool word (`0x000201ac`) followed by additional pointer literals (`0x00020104`, `0x00020420`, `0x00021ca0`, etc.). The actual descriptor bytes are not recovered from that literal pool. Their initialization and exact lengths/report IDs remain unresolved.

Classification: `CONFIRMED_STATIC` for descriptor-type selection and pointer-literal values; `UNKNOWN` for the actual descriptor contents and interface/report layout.

### 5.3 Control transfer helper candidates

Functions around `0x1cd38`, `0x1c47e`, `0x1c4ac`, `0x1c066`, `0x1c274` and the callback-like region `0x1c5b6–0x1c830` participate in control-buffer or transfer-state operations. The function at `0x1cd38` stores buffer/length fields and calls `0x1c47e`; `0x1c47e` builds a transfer descriptor using a `0x24`-byte per-entry stride. These are static data-structure observations, not proof that a particular Feature report reaches the application dispatcher.

The direct call at `0x1c7b0` proves that the `0x1c5b6` region can invoke the application dispatcher. It does not prove that the call was triggered by HID `SET_REPORT`, nor that the 64-byte stack buffer at the top of the function is copied into the dispatcher's global buffer `0x00021240`.

### 5.4 End-to-end HID bridge status

The following edges are established or remain open:

| Edge | Status | Evidence |
|---|---|---|
| USB setup packet parsing at `0x1cb48` | `STRONG_STATIC_INFERENCE` | `bmRequestType` and `bRequest` dispatch; HID descriptor types `0x21/0x22` |
| HID `SET_REPORT` request to a dedicated handler | `UNKNOWN` | `bRequest=0x09` maps to default path in the candidate class table |
| USB/control callback region to application dispatcher | `STRONG_STATIC_INFERENCE` | `0x1bd54 → 0x1c5b6`, and `0x1c5b6` contains call at `0x1c7b0 → 0x19c5e` |
| Exact Feature SET payload to global buffer `0x00021240` | `UNKNOWN` | No proven pointer/copy chain from setup data stage into that buffer |
| Feature report ID/length for the exact firmware interface | `UNKNOWN` | Descriptor bytes not decoded |
| Exact `SET_REPORT` → opcode selector → command handler path | `UNKNOWN` | Material data-flow edges unresolved |

The candidate memory mapping and Thumb decode assumptions remain in force. Do not claim that the frontend's Feature SET data has been proven to reach `0x19c5e`.

## 6. Vendor native driver comparison

Prior static analysis in `PHASE5C_FEATURE_REPORT_CORRELATION.md` and `PHASE5D_FEATURE_BUFFER_LAYOUT.md` established the native generic Feature transport:

| Method | `bmRequestType` | `bRequest` | `wValue` | `wIndex` | `wLength` |
|---|---:|---:|---:|---:|---:|
| `HidLibusb::send_feature` at VA `0x100048290` | `0x21` | `0x09` | `0x0300` | `0` | `64` |
| `HidLibusb::get_feature` at VA `0x100048460` | `0xa1` | `0x01` | `0x0300` | `0` | `64` |

The support metadata and earlier descriptor audit identify the TH80 Pro vendor Feature interface as interface 2. The native generic libusb methods hardcode `wIndex=0`. The reason and operational validity of that mismatch are unknown.

The native SET future uses `dangledevtype` (not the frontend protobuf `checksumtype`) to select checksum construction:

| `dangledevtype` | Native sum | Destination |
|---:|---|---:|
| `0` | offsets `1..7` | offset `7` |
| `1` | offsets `1..8` | offset `8` |
| `2` | checksum insertion skipped | — |

The destination's preexisting value participates in the native sum if it was supplied by the caller; otherwise it is zero-filled. The transport sends the first 64 bytes.

### 6.1 Byte-for-byte comparison

The algorithms are **not equivalent**:

- Firmware selects checksum position from command byte `buffer[0]`: opcodes `0x07` and `0x08` use checksum byte 8; other opcodes use checksum byte 7.
- Firmware includes `buffer[0]` in the accumulated sum.
- Native code excludes `buffer[0]`, and selects destination byte 7 or 8 from `dangledevtype`, independent of the opcode.
- Native code sums the preexisting destination byte; firmware adds the designated final checksum byte after summing the preceding span.

Even when both paths choose the same destination offset, the summed byte ranges differ by the inclusion/exclusion of offset zero and the shifted checksum byte. Matching one's-complement arithmetic alone does not establish protocol compatibility.

Classification: `CONFIRMED_STATIC` for each independently recovered algorithm; `STRONG_STATIC_INFERENCE` that the two implementations are structurally incompatible as written; `UNKNOWN` whether another native/firmware route or device metadata mode changes the actual deployed behavior.

## 7. Important unresolved items

- Exact CPU ISA, physical flash base and board/firmware match.
- Registration of `0x1cb48` as the active setup handler for the vendor HID interface.
- Why the candidate class request table has no dedicated GET_REPORT/SET_REPORT branch.
- Descriptor contents, report IDs, report lengths and interface selection.
- Exact source/destination pointers from EP0 data stage to `0x00021240`.
- Whether the `0x1c5b6` callback path is the actual Feature SET receive path.
- Whether the dispatcher has indirect callers not visible as direct `BL` references.
- Whether the ordinary vendor `/sendMsg` and `/readMsg` routes reach the native candidate Feature futures; prior Phase 5C/5D route-to-handler limitations remain.

## 8. Reproducibility commands

```bash
FW='../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X'
sha256sum "$FW"
rg -n 'bl\s+0x19c1e|bl\s+0x19c5e' /tmp/th80-X-thumb.dis
sed -n '/^ *19c1e:/,/^ *19c5e:/p' /tmp/th80-X-thumb.dis
sed -n '/^ *19c5e:/,/^ *19d30:/p' /tmp/th80-X-thumb.dis
sed -n '/^ *1cb48:/,/^ *1ccc8:/p' /tmp/th80-X-thumb.dis
```

No hardware or firmware operation was performed. All frontend and native-driver observations remain host-side evidence; firmware-side claims are limited to the static code paths described above.
