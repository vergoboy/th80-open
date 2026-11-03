# Phase 13 — Firmware RGB Analysis

**Status:** static investigation complete for the supplied artifact; RGB implementation and device-side command semantics remain unproven. No firmware was modified, executed, patched or flashed. No keyboard/receiver was opened, enumerated or contacted.

## 1. Executive conclusion

The supplied `X` payload contains substantial code that LLVM can decode coherently under ARM Thumb assumptions, plus embedded Yichip SDK material. The exact instruction set is not proven. A candidate flash mapping at `0x01000000` is strongly supported by internal absolute pointers, but the exact load address, CPU implementation, boot/application boundary and firmware-to-board match are not formally established.

The best external hardware lead is the original TH80 review identifying a Yichip YC3121-L main MCU and Beken BK3632 wireless controller. That is a TH80-family sample, not proof of the exact PCB revision in the user's TH80 Pro. The V504 updater package does target the normal-mode USB identity `3151:4010`, interface 2, and EPOMAKER warns that V504 is for ANSI V500/V500+ units, not V2xx or ISO keyboards. This improves package relevance but does not establish the firmware currently installed in the user's unit.

The firmware contains GPIO and timer/PWM SDK functionality. There is no recovered LED-driver IC identity, LED bus/protocol, RGB state structure, RGB mode table, persistent lighting record, Fn+Backspace handler or `0x87`/`0x89` command parser. The physical RGB root cause therefore cannot be diagnosed from this image.

## 2. Artifact identity and provenance

### Firmware payload

- Path: `../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X`
- Size: `184608` bytes (`0x2d120`)
- SHA-256: `4b22336ec9ffaaac3372791a1154a2c2a833e3305a4cadbbdcab9184252c0520`
- `file`: generic `data`; no ELF/PE/Intel HEX container.
- Package sidecars: `support_config.toml`, `selections_config.toml`, `upgrade_config.toml`, `Upgrade Guide.pdf`, and the Windows updater.
- The package support configuration lists `3151:4010`, Usage Page `0xffff`, Usage `0x02`, interface 2, and bootloader PID `0x4001`. It does not list receiver PID `0x4011`.
- EPOMAKER's TH80 Pro product page explicitly says V504 supports ANSI V500/V500+ and must not be flashed to V2xx or ISO units. The user's exact PCB revision and currently installed firmware remain unknown.

### Vendor application artifacts inspected alongside firmware

- Native driver: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/iot_driver`
  - 6,075,584 bytes; SHA-256 `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`
  - Mach-O 64-bit x86_64.
- Frontend bundle: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js`
  - 4,370,837 bytes; SHA-256 `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87`.

## 3. Architecture and memory-map assessment

### Instruction set

LLVM's ARM Thumb decoders produce a coherent-looking instruction stream in sampled regions. `thumbv6m-none-eabi` and `thumbv7m-none-eabi` decode the initial samples to the same sequence, predominantly 16-bit Thumb-1-style instructions with 32-bit BL calls. Several indirect code pointers have bit 0 set, which is compatible with the ARM Thumb state-bit convention.

However, the YC3121 product brief describes a **32-bit RISC core** and does not identify it as ARM or Thumb. The decoder output and odd pointers make ARM Thumb a plausible hypothesis, but they do not prove the ISA. A proprietary RISC instruction set could share enough encodings to produce plausible disassembly. No ELF header, architecture tag, linker map, symbol table or trusted load-address metadata exists in `X`. Do not call the target Cortex-M0 or ARMv6-M as a confirmed fact.

### Candidate image base and vector/dispatch structures

A candidate mapping of file offset zero to address `0x01000000` is strongly supported by internal pointers:

- At file offset `0x000000`, the code loads a literal from `0x198` and branches through it.
- The 32-bit word at file offset `0x198` is `0x01000401`. Under the candidate base, the Thumb target is file offset `0x400`.
- At file offset `0x10000`, repeated `LDR literal; BX` stubs load words from a literal pool beginning at `0x10128`. The first words include `0x01010401`, `0x010102a7`, `0x010102a9`, `0x010102ab`, and `0x0101054d`, which map to in-image Thumb targets under the same base.
- The targets around `0x102a6`, `0x10400`, and `0x1054c` decode as plausible Thumb routines.

This is a consistent image-base hypothesis, not proof of the physical flash mapping. The `0x10000` structure is a vector/interrupt-dispatch-stub candidate, not a conventional Cortex-M vector table as stored, and its precise role is unresolved. No bootloader/application boundary has been established.

### Peripheral and RAM references

Aligned literal-pool scans show a repeated cluster of low peripheral-like addresses, including `0x000f8540`, `0x000f8700`, `0x000f6020`, `0x000f8800`, `0x000f8720`, `0x000f8b00`, and `0x000f8c00`. This is unlike the common STM32 `0x400xxxxx` peripheral map; no STM32 map should be imposed.

A GPIO SDK routine around file offsets `0x0c50–0x0cd4` loads `0x000f8700` from the literal at `0x0d94`, indexes a register bank and tests bit masks. The nearby error-reporting path references the embedded `yc_gpio.c` source path at `0x0d68`. This supports `0x000f8700` being part of a GPIO register region. Other `0x000fxxxx` values are not mapped confidently to named peripherals.

No reliable SRAM base/size or persistent-storage address is recoverable from this image alone. The YC3121 product brief describes the family as a 32-bit RISC SoC with selectable 512 KB/1 MB secured flash, 64 KB SRAM, GPIO, SPI, IIC, DMA, USB and eight 32-bit PWM-capable timers. Those are family capabilities, not proof of the exact `-L` part or this keyboard's RGB implementation.

## 4. Embedded SDK and peripheral evidence

Readable firmware strings include:

- `ROYUAN`, `Gaming Keyboard`, `HID Config`, `HID Interface`.
- SDK source-path remnants: `yc_gpio.c`, `yc_uart.c`, `yc_timer.c`, `yc_systick.c`, `yc_sysctrl.c`, `yc_wdt.c`, `yc_ssc.c`.
- SDK API names: `GPIO_Init`, `GPIO_SetBits`, `GPIO_WriteBit`, `TIM_PWMInit`, `TIM_SetPWMPeriod`, `TIM_PWMDifferential`, `SysTick_Config`, UART and sleep/watchdog helpers.

The strings establish that Yichip-style GPIO/timer/PWM SDK material is embedded. They do not prove those APIs are called for RGB, nor identify timer channels, pins, PWM frequencies, LED power rails, a serial protocol, a separate driver IC or a DMA/SPI/I2C LED chain.

No obvious `RGB`, `LED`, `BACKLIGHT`, `BRIGHTNESS` or named lighting-mode strings were recovered from the firmware string scan. This negative result is weak: firmware may use numeric tables and stripped identifiers.

## 5. Relevant disassembly findings

All addresses below are **file offsets in the raw `X` payload**, not confirmed runtime virtual addresses.

### 5.1 Startup and dispatch candidates

- `0x0000`: `ldr r0, [pc, #0x194]; bx r0`.
- `0x0198`: literal `0x01000401`, candidate reset/startup target under base `0x01000000`.
- `0x10000–0x100a0`: repeated indirect branch stubs; literal pool at `0x10128` contains pointers into the image.

If the ARM Thumb interpretation is correct, the pointer low bit selects Thumb state; exact reset semantics and vector-table ownership remain unconfirmed.

### 5.2 GPIO SDK and the misleading `0x87` immediate

- `0x0c50`: GPIO-like routine prologue.
- `0x0c5c`: `movs r1, #0x87`, followed by construction of an error message and a call to the common error logger at `0x04fc`.
- The nearby literal/string at `0x0d68` is the path `.../sdk/yc_gpio.c`; `0x0d94` contains `0x000f8700`.
- Similar adjacent routines use error IDs `0x80` and `0x88` for validation paths.

Therefore, the literal `0x87` found at this code location is a **GPIO SDK error identifier**, not evidence of HID vendor opcode `0x87`. It must not be used as an opcode-handler cross-reference.

### 5.3 Checksum helper

At `0x19c1e–0x19c5c`, a Thumb routine:

1. Reads a selector byte from `[r0]`.
2. For selector values 7 or 8, sums bytes 0–7 with wrapping 8-bit accumulation, then includes byte 8.
3. For other selector values, sums bytes 0–6, then includes byte 7.
4. Complements the low 8 bits and returns success only when the result is zero.

This is a complemented wrapping-byte-sum check over either an 8-byte span (other selectors) or a 9-byte span (selector 7/8). It resembles the generic complemented-sum logic recovered in the vendor transport, but its selector values and byte spans do not directly match the vendor transport selector branches as documented. No call graph connects this firmware helper to the RGB opcodes or to the vendor Feature report parser; do not claim these are the same checksum routine.

### 5.4 Generic switch-table candidate

The earlier candidate around `0x19c40` was re-evaluated. It is inside the checksum loop above, not a 36-entry HID opcode dispatcher. The previous tentative switch-table interpretation is withdrawn.

## 6. MCU and RGB hardware conclusions

### MCU

An independent TH80 review reports `Yichip YC3121-L` as the keyboard MCU and `Beken BK3632` as the Bluetooth/2.4 GHz controller. The firmware's `yc_*` SDK paths and Thumb code are compatible with a Yichip embedded-controller lead. However:

- the review concerns a TH80-family sample, not a verified photograph of this user's TH80 Pro PCB;
- the V504 package support configuration targets VID:PID `3151:4010`, but does not identify the chip marking;
- no PCB photo, silkscreen revision or IC marking from the user's unit is available.

**Assessment:** YC3121-L is the best-supported family candidate, not a confirmed identity for the user's exact board. BK3632 is likewise an external review lead, not a confirmed observation of this board.

### LED controller and signalling

EPOMAKER documents south-facing per-key RGB and per-key RGB customization for TH80 Pro. No separate LED-controller IC or LED bus protocol has been identified. Possible architectures include MCU timer/PWM/GPIO driving, a serial LED chain, or an external LED driver; the current artifact does not discriminate among them.

No evidence establishes:

- whether LEDs are driven directly by MCU GPIO/PWM or through a separate driver;
- whether a hardware driver is present;
- LED pin mapping, PWM channels/frequency, SPI/I2C/DMA use or bit-banged signalling;
- a global LED enable/power-gate signal;
- RGB state storage in MCU flash, controller NVM, receiver memory or volatile RAM;
- startup defaults, sleep/wake restoration or persistence behavior.

### RGB modes and state

The vendor frontend has mode/value/color fields (see `PHASE13_VENDOR_PROTOCOL_RGB_CORRELATION.md`), but the firmware image has no recovered mode enum, color lookup table, speed table, brightness table or RGB configuration structure. Firmware-side representation remains unknown.

## 7. Confidence ledger

| Question | Finding | Confidence |
|---|---|---|
| Raw image identity | Exact 184,608-byte V504 package sidecar `X`, matching SHA-256 | High |
| Instruction set | Coherent Thumb-like decode and odd code pointers; YC3121 brief only says 32-bit RISC | Moderate hypothesis; exact ISA unknown |
| Image base | `0x01000000` explains multiple in-image pointers | Strong hypothesis, not confirmed |
| MCU | YC3121-L lead from TH80 review + Yichip SDK strings | Moderate for TH80 family; unconfirmed for user's board |
| Wireless controller | BK3632 lead from same review | Moderate for reviewed sample; unconfirmed for user's board |
| GPIO peripheral region | `0x000f8700` used in `yc_gpio.c` error/bit-mask routine | High for GPIO-related register use |
| PWM capability | SDK symbols and YC3121 family brief show timer/PWM capability | High for family capability; unknown RGB use |
| RGB controller/protocol | Not recovered | Unknown |
| RGB state/persistence | Not recovered | Unknown |
| Firmware handlers for `0x87`/`0x89` | Not recovered | Unknown |
| Fn+Backspace implementation | Not recovered | Unknown |

## 8. Reproducibility

Environment/tool versions recorded during this pass:

- LLVM `23.1.4` (`llvm-objdump`, `llvm-objcopy`)
- GNU Binutils `2.47` (`objdump`, `strings`, `readelf`)
- `file` `5.48`
- GNU coreutils `sha256sum` `9.12`
- Python `3.14.8`
- `otool`, Ghidra, radare2, Capstone and pyelftools were unavailable in the environment.

Commands (run from `th80-open/`):

```bash
FW='../TH80_ANSI_Firmware_V504/TH80 ANSI Firmware V504(ID 742)/X'
sha256sum "$FW"
stat -c '%n %s bytes' "$FW"
file "$FW"
strings -a -t x -n 3 "$FW"

llvm-objcopy -I binary -O elf32-littlearm \
  --binary-architecture=arm \
  --rename-section .data=.text,alloc,load,readonly,code \
  "$FW" /tmp/th80-X-analysis.o

llvm-objdump -D --triple=thumbv6m-none-eabi \
  --section=.text /tmp/th80-X-analysis.o > /tmp/th80-X-thumb.dis
llvm-objdump -D --triple=thumbv7m-none-eabi \
  --section=.text --start-address=0 --stop-address=0x40 \
  /tmp/th80-X-analysis.o
llvm-objdump -D --triple=thumbv6m-none-eabi \
  --section=.text --start-address=0x19c1e --stop-address=0x19c60 \
  /tmp/th80-X-analysis.o
```

`/tmp/th80-X-analysis.o` is a temporary wrapper around the unchanged raw file; it is not a firmware conversion suitable for flashing. `--triple=thumbv6m-none-eabi` and `thumbv7m-none-eabi` are decoder assumptions. The raw image was never executed.

## 9. References

- `HARDWARE.md`, `RESEARCH.md`, `FIRMWARE_ANALYSIS.md`, `UPDATER_REVERSE_ENGINEERING.md`
- `PHASE5C_FEATURE_REPORT_CORRELATION.md`, `PHASE5C_RPC_DISPATCH_RESOLUTION.md`
- `PHASE5D_FEATURE_BUFFER_LAYOUT.md`, `PHASE5D_RPC_DISPATCH_AUDIT.md`
- `PHASE8_STATIC_EVIDENCE_CLOSURE.md`
- `PHASE12_1_RGB_DIAGNOSTICS.md`, `PHASE12_1_CLOSURE.md`, `PHASE12_2_LIGHTING_APP_AUDIT.md`
- EPOMAKER TH80 Pro product page: https://epomaker.kr/products/epomaker-th80-kit
- TH80 hardware review: https://next.lab501.ro/featured/review-epomaker-th80-budgerigar-switch-monet-mda-v2/2
- YC3121 product brief: https://device.report/m/b6af047a35586162da294eab94c8bef523b008da89e7abc3c63d7a1ad311a91d.pdf
