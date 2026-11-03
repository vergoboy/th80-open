# Phase 13.2 — USB Feature SET bridge and descriptor investigation

**Scope:** static inspection of the V504 raw firmware and comparison with the already captured host HID descriptors and vendor-driver reports. No USB/HID enumeration, physical handle access, Feature GET/SET, control transfer, firmware execution, or device state change was performed during this phase.

## 1. Address and evidence conventions

All firmware addresses below are raw-file offsets in the 184,608-byte image `X`. The binary is not an ELF and its load/flash base is not established. The often-used `0x01000000 + offset` mapping remains a hypothesis, not an address claim. The Thumb disassembly is the existing `/tmp/th80-X-thumb.dis`; the temporary ELF wrapper is an analysis artifact, not an original input.

Confidence labels used in this report are exactly: `CONFIRMED_STATIC`, `STRONG_STATIC_INFERENCE`, `WEAK_INFERENCE`, `FRONTEND_ONLY`, and `UNKNOWN`.

## 2. USB class setup handler

### 2.1 Candidate handler and request dispatch

- Entry: `0x1cb48`.
- Nearby pointer-literal pool: `0x1cd08–0x1cd34`.
- The handler reads the setup packet through `r1`, masks `bmRequestType` with `0x60`, and distinguishes standard requests from class requests.
- The class path has request-specific handling and a common/default path. The `SET_REPORT` (`bRequest=0x09`) path reaches `0x1cc24`, which calls `0x1cdd4` and returns status value `2`.
- `0x1cdd4` at `0x1cdd4–0x1cddc` calls `0x1c4ac` with `r1=0`. `0x1c4ac` selects a transfer descriptor within the context structure using a `0x24`-byte stride, writes transfer state/type fields, and calls `0x1c32a`.
- `0x1c32a` updates a bit in the state byte reached through the context's `+0x10` pointer. This is a scheduling/state transition, not a direct application-dispatch call.

Classification: `CONFIRMED_STATIC` for the request-handler branch and the `0x1cc24 → 0x1cdd4 → 0x1c4ac → 0x1c32a` call sequence. `STRONG_STATIC_INFERENCE` that this is the setup-stage arm of the SET_REPORT control-transfer path. The callback/transfer-completion registration edge is not fully resolved.

### 2.2 Transfer state machine and command-buffer copy

The most important recovered data-flow fragment is in the callback/state-machine body at `0x1c5b6`, reached by wrapper `0x1bd54` (`0x1bd54` calls `0x1c5b6`). The wrapper loads the context pointer literal `0x21c80` from the pool at `0x1c000`.

In the state-machine branch at `0x1c74a–0x1c7b4`:

1. It loads the transfer-state block pointer from the literal at `0x1c83c` (`0x00020113`) and adds two, producing a state structure based at `0x20115`.
2. It requires the state byte at `0x20118` to equal `3` for the command-buffer copy branch. A value of `2` takes a different staging/copy path; other values return to the state loop.
3. On state `3`, it clears byte `0x20117`, sets `r7 = 0x7b000`, and loads destination pointer `0x21240` from literal `0x1c854`.
4. The loop at `0x1c79c–0x1c7a6` copies bytes from `[0x7b000 + 0x18]` to `[0x21240 + index]`. The loop bound is byte `[0x2011a]` (the state structure's `+3` field).
5. At `0x1c7a8–0x1c7b0`, it calls `0x1c274` with `r1=0`, then directly calls the application dispatcher at `0x19c5e`.

The copy and dispatch sequence is therefore:

```text
staged transfer data at 0x7b018
    -- copy count [0x2011a] -->
shared command buffer 0x21240
    --> 0x1c274 completion/state update
    --> 0x19c5e application dispatcher
```

Classification: `CONFIRMED_STATIC` for the byte-copy loop, its source/destination addresses, the state-3 condition, and the direct dispatcher call. `STRONG_STATIC_INFERENCE` that the staged bytes are the completed host control-transfer data. The exact relationship between this callback state value and the preceding SET_REPORT setup branch is not proven by a direct call or a recovered callback registration table.

### 2.3 Other transfer helpers and indirect callbacks

- `0x1c040` copies a requested count from a transfer descriptor's buffer pointer (`context + 0x14` selected through a `0x24`-byte record) to a caller-provided destination. It exits early when the context flag at `+3` is nonzero.
- `0x1c066` copies from the fixed staging area `0x7b000 + offset + 0x18` into a caller-provided buffer.
- `0x1c274` updates a one-bit status in the byte addressed by the context's `+0x10` pointer.
- `0x1c284` is a transfer progression routine: it checks descriptor state, advances offsets, invokes `0x1c040`, and calls `0x1c156`/`0x1c274` as appropriate.
- `0x1c3e6`, `0x1c420`, `0x1c448`, `0x1c47e`, and `0x1c4ac` populate the `0x24`-byte transfer records. Their fields include endpoint/slot indices, state bytes, data pointers, lengths, and callback-related pointers.
- `0x1c5b6` uses an indirect callback from a table reached through context/global pointer `0x20148` (literal at `0x1c834`); it invokes a function pointer loaded at table offset `+8` in the transfer-completion path. It also uses callback entries at offsets `+4` in related paths.
- Wrapper `0x1bd54` calls `0x1c5b6`, but no direct `BL` caller or raw literal reference to its Thumb pointer was recovered. The registration/interrupt entry that invokes the wrapper remains unresolved.

Classification: `CONFIRMED_STATIC` for the helper behavior visible in the disassembly and the indirect `BLX` sites; `UNKNOWN` for the unresolved callback target/registration and the physical identity of the `0x7b000` staging region.

### 2.4 Feature SET bridge conclusion

The firmware contains both halves needed for a bridge: a `SET_REPORT` setup-stage path through `0x1cc24 → 0x1cdd4 → 0x1c4ac`, and a transfer-state branch that copies `0x7b018` into `0x21240` before calling `0x19c5e`. The bridge is highly plausible, but the exact callback-registration/dispatch edge tying that particular setup request to the state-3 callback has not been statically closed.

Overall: `STRONG_STATIC_INFERENCE` for SET_REPORT-to-dispatcher; `CONFIRMED_STATIC` for the staged-buffer-to-dispatcher transition itself.

## 3. HID descriptor recovery

### 3.1 Verified GET_DESCRIPTOR selection logic

The standard-request branch at `0x1cc3e–0x1ccb0` handles descriptor type `0x22` (HID Report Descriptor) and type `0x21` (HID class descriptor). For report descriptor type `0x22`, it selects by `wIndex`/interface index:

| Interface index | Pointer calculation from RAM base `0x201ac` | Maximum length | Host capture length |
|---:|---|---:|---:|
| 0 | `0x201ac + 0x10` | `0x3b` (59) | 59 |
| 1 | `0x201ac + 0x4b` | `0xb1` (177) | 177 |
| 2 | `0x201ac + 0xfc` | `0x14` (20) | 20 |

The handler clamps the requested `wLength` to the selected maximum before calling the common transfer setup helper at `0x1cd38`. For type `0x21`, it selects three 9-byte class-descriptor regions using base pointer `0x202ce` and offsets `0`, `0x19`, and `0x32`.

Classification: `CONFIRMED_STATIC` for descriptor type checks, interface-index selection, pointer arithmetic, and maximum lengths.

### 3.2 Comparison with captured host descriptors

The repository's `docs/hid-descriptors.md` contains the read-only host captures for TH80 Pro `3151:4010` and receiver `3151:4011`. They match the firmware's three report-descriptor lengths exactly:

- Interface 0: 59-byte keyboard Input descriptor, plus 1-byte Output item.
- Interface 1: 177-byte multi-collection Input descriptor with report IDs 1, 2, 3, 5, and 6.
- Interface 2: 20-byte vendor-defined Feature descriptor, Usage Page `0xffff`, Usage `0x02`, `Report Size=8`, `Report Count=64`, no Report ID item.

The interface-2 descriptor therefore declares one unnumbered 64-byte Feature payload. The host capture also records interface 2 as the Feature-bearing interface. This is consistent with the firmware's descriptor selection by interface index 2.

Classification: `CONFIRMED_STATIC` for the firmware's descriptor lengths and selection logic; `CONFIRMED_STATIC` for the contents of the existing host-captured descriptor files; `STRONG_STATIC_INFERENCE` that the firmware descriptor slots correspond to those same three host descriptors. The actual firmware descriptor bytes have not been recovered.

### 3.3 Embedded versus runtime-generated descriptor data

The literal pool at `0x1cd08` contains the pointer `0x000201ac`, not descriptor bytes. The pointer targets a RAM-like address area; the raw firmware bytes at file offset `0x201ac` are zero-filled. Searches for the captured interface-0 and interface-2 descriptor byte sequences in the raw firmware image did not find them. No verified initializer/copy site that populates the `0x201ac` region was identified in this phase.

Therefore it is not justified to call the descriptor a literal embedded array. It is a runtime-populated RAM descriptor area, or a pointer to one, with its initialization source still unknown.

Classification: `CONFIRMED_STATIC` that `0x1cd08` is a pointer-literal pool and that GET_DESCRIPTOR uses RAM-area pointers; `UNKNOWN` whether the descriptor bytes are generated at runtime, copied from a compressed/encoded template, or populated by another startup component.

## 4. Host/native transport correlation

Existing reports `PHASE5C_FEATURE_REPORT_CORRELATION.md`, `PHASE5D_FEATURE_BUFFER_LAYOUT.md`, `PHASE5D_HIDAPI_FEATURE_AUDIT.md`, `USB_PROTOCOL.md`, and `EP0_PROTOCOL.md` establish the supplied macOS native driver's generic libusb Feature calls:

- SET: `HidLibusb::send_feature` at `0x100048290`; `libusb_control_transfer` uses `bmRequestType=0x21`, `bRequest=0x09`, `wValue=0x0300`, `wIndex=0`, `wLength=64`.
- GET: `HidLibusb::get_feature` at `0x100048460`; uses `bmRequestType=0xa1`, `bRequest=0x01`, `wValue=0x0300`, `wIndex=0`, `wLength=64`.
- The target host descriptor puts the Feature report on interface 2, while this generic native routine supplies `wIndex=0`. The mismatch remains unresolved; no alternative Feature transport was found in the HIDAPI/IOKit path.
- The native SET future inserts a complemented byte-sum checksum at byte 7 or 8 depending on `dangledevtype`, excluding byte 0. The firmware checksum validator at `0x19c1e` includes byte 0 and uses it as the span selector. This is a structural discrepancy, not proof that all native commands are incompatible.

These are static driver findings from prior phases; the driver was not executed.

## 5. USB bridge call graph and unresolved edges

```text
USB HID class setup handler 0x1cb48
  SET_REPORT (bRequest 0x09)
    -> default path 0x1cc24
      -> 0x1cdd4
        -> 0x1c4ac (EP0 transfer descriptor setup)
          -> 0x1c32a (transfer-state bit update)

transfer completion wrapper 0x1bd54
  -> 0x1c5b6 (transfer state machine)
    [state at 0x20118 == 3]
    -> copy 0x7b018 -> 0x21240, count [0x2011a]
    -> 0x1c274 (completion/status update)
    -> 0x19c5e (application dispatcher)
      -> 0x19c1e (checksum validator)
```

The top and bottom halves are independently supported. The missing edge is the exact callback registration/USB interrupt dispatch that joins the SET_REPORT setup-stage transfer to wrapper `0x1bd54` and state value `3`. The transfer record's indirect callback table and its registration site must be resolved before promoting the entire chain to `CONFIRMED_STATIC`.

## 6. Safety and scope

No physical HID device was enumerated or opened for this phase. No Feature GET/SET, Output report, control transfer, endpoint read/write, firmware execution, reset, pairing, calibration, bootloader entry, or updater action was performed. The existing descriptor capture is repository evidence from earlier work, not a new hardware operation in this phase.
