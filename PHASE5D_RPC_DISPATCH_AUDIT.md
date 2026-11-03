# Phase 5D — RPC dispatch and field-consumer audit

Date: 2026-11-01  
Scope: static re-evaluation of the renderer RPC contract, native Tonic route registration, candidate poll functions, request-field dataflow and response conversion. No RPC was sent to the vendor driver.

## Artifact identity

- Native `iot_driver` SHA-256: `22c5dc67ba77e2373a5ffdbb0312ed8c8e73eb16e78037764eb676a030b2724b`.
- Renderer bundle: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js`.

## Executive verdict

- **CONFIRMED:** Renderer RPC descriptors register distinct unary raw routes `/driver.DriverGrpc/sendRawFeature` and `/driver.DriverGrpc/readRawFeature`.
- **CONFIRMED:** Native Tonic generated route symbols for `sendRawFeatureSvc<GrpcServer>` and `readRawFeatureSvc<GrpcServer>` exist.
- **CONFIRMED:** The native router compares the exact raw method path strings and selects distinct route descriptors: `/driver.DriverGrpc/sendRawFeature` → `0x1003cd908`, `/driver.DriverGrpc/readRawFeature` → `0x1003cd948`. Their descriptor poll slots contain Tonic unary wrapper poll functions `0x1000a0f50` and `0x100092ec0`, respectively.
- **STRONGLY SUPPORTED:** The inner device-path Feature helper polls are `0x1000aa0a0` (SET) and `0x1000a3ef0` (GET); they perform manager path lookup and dispatch Base Feature vtable slots `+0x30` and `+0x38`. These are helper futures, not the Tonic route wrapper polls.
- **UNKNOWN:** The final indirect `UnaryService::call` vtable target that connects each raw route wrapper to the distinct `GrpcServer::send_raw_feature` / `read_raw_feature` handler future. The router-to-wrapper edge is proven; the wrapper-to-handler callable edge remains indirect and has not been resolved to a unique function pointer.
- **CONFIRMED:** `devicepath` is passed into manager-list lookup; `dangledevtype` is normalized and propagated to the native Feature SET future; the renderer sets `checksumtype`, but no native consumer has been identified in the candidate path.
- **UNKNOWN:** The exact `DJDevInstance` field compared with `devicepath`, and complete native error-to-protobuf conversion for all branches.

## Renderer contract

The bundled renderer method descriptors (minified source around line 1554) specify:

```js
sendRawFeature: {
  path: "/driver.DriverGrpc/sendRawFeature",
  requestStream: false,
  responseStream: false,
  requestType: SendMsg,
  responseType: ResSend
}
readRawFeature: {
  path: "/driver.DriverGrpc/readRawFeature",
  requestStream: false,
  responseStream: false,
  requestType: ReadMsg,
  responseType: ResRead
}
```

At renderer line 1556, the SET helper constructs `SendMsg`, assigns `devicepath`, `msg`, `checksumtype` and `dangledevtype`, then calls `sendRawFeature`. It converts `ResSend.err` to a boolean (`true` when empty). The GET helper constructs `ReadMsg`, assigns `devicepath`, calls `readRawFeature`, and returns `ResRead.msg` as a Buffer or `undefined` if absent. **CONFIRMED frontend behavior.**

The YZW common Feature helper at line 1564 performs a raw SET followed by raw GET. It computes `dangledevtype` as 2 for a listed dongle mouse, 1 for a listed dongle keyboard, and 0 otherwise. `checksumtype` has protobuf default 0. These frontend values do not prove native interpretation of `checksumtype`.

## Native route registration and candidate handlers

Native `DriverGrpcServer<GrpcServer>::call` is at `0x100048d50`. It routes HTTP/2 gRPC paths using static route data in `0x1003cd8a8..0x1003cd968`. Generated Tonic route symbols exist for both raw methods.

Candidate poll functions:

| Candidate | VA | Recovered behavior | Grade |
|---|---:|---|---|
| SendMsg-shaped poll | `0x1000aa0a0` | Reads/normalizes `dangledevtype`, calls `get_dev_from_path` at `0x1000aa1da`, dispatches Base Feature SET slot `+0x30` | **STRONGLY SUPPORTED as raw SET body** |
| ReadMsg-shaped poll | `0x1000a3ef0` | Calls `get_dev_from_path` at `0x1000a3f5d`, dispatches Base Feature GET slot `+0x38` | **STRONGLY SUPPORTED as raw GET body** |

The raw route closure drop-glue symbols are shared/folded with ordinary `sendMsg`/`readMsg` closure types (`0x100040a60` send-shaped and `0x100040990` read-shaped). Shared drop glue is not a unique dispatch edge. The native `DriverGrpcServer::call` path matcher at `0x100048d50` makes the exact raw comparisons: the 33-byte string at `0x10036fce7` (`/driver.DriverGrpc/sendRawFeature`) is compared at `0x100049032..0x100049041`; equality branches to `0x10004932b`, which selects descriptor `0x1003cd908` at `0x10004931f`. The 33-byte string at `0x10036fd08` (`/driver.DriverGrpc/readRawFeature`) is compared at `0x10004904e..0x10004905d`; equality branches to `0x1000493f0`, which selects descriptor `0x1003cd948` at `0x100049383`. Each descriptor begins with a 32-byte future-vtable tuple `{drop_fn, size=0x520, align=8, poll_fn}`: send raw poll slot `0x1003cd920` = `0x1000a0f50`; read raw poll slot `0x1003cd960` = `0x100092ec0`. These are exact URI-to-Tonic-wrapper-poll edges.

The wrapper futures then call the common inner request/map future poll `0x1000abd80`; its service dispatch is an indirect vtable call at `0x1000ac5ed` (`call *0x20(%rax)`). The specific `UnaryService::call` vtable target is not statically tied to a unique raw handler function in the recovered evidence. The candidate helper polls `0x1000aa0a0` and `0x1000a3ef0` are called from outer futures at `0x100099b76` / `0x1000af3a6` and `0x1000a665d` / `0x1000c038d`; their enclosing code uses `GrpcServer::send_msg_by_dev_path` / `read_msg_by_dev_path` helper futures. The binary does not expose a unique static edge showing which enclosing future corresponds to each raw route rather than the ordinary routes. Thus the raw URI-to-Tonic-wrapper mapping is confirmed, but the final wrapper-to-handler/poll-function mapping remains unknown.

**CONFIRMED:** Exact raw URI → Tonic unary wrapper poll mapping. **UNKNOWN:** Exact raw URI → concrete `GrpcServer::send_raw_feature` / `read_raw_feature` handler poll mapping, because the final call uses an indirect `UnaryService` vtable at `0x1000ac5ed` and its route-specific function pointer was not recovered. Do not substitute the helper poll functions for that missing edge.

## Device-path lookup

`iot_driver::grpc_server::get_dev_from_path` is at `0x1000603a0`. It obtains `DJDeviceManger::get_dev_list`, iterates the `Arc<DJDevInstance>` collection, and captures the supplied `devicepath` string in the filter closure environment around `0x10006041d`. The matching `Arc` is retained and returned to the candidate handler.

- **CONFIRMED:** The lookup is manager-list based and consumes the request path string.
- **UNKNOWN:** The final predicate's concrete `DJDevInstance` field and exact string comparison routine; the full closure body has not been uniquely reconstructed.

## `dangledevtype` propagation

The candidate SET poll reads the protobuf field around `0x1000aa13a` and normalizes it:

- input 1 → 1;
- input 2 → 2;
- all other values → 0.

The normalized byte is passed in RCX to Base `pure_send_feature` vtable slot `+0x30` at `0x1000aa799` and `0x1000aa7d7`. The Base future captures it at environment offset `+0x398` (`BleHid` factory `0x100038e78`; `YZWDangleCommon` factory `0x1000610d0`). The value propagates into the nested `DJDevice::send_feature` future (`0x100091101..0x100091124` and `0x10008ed5a..0x10008ed74`).

The `DJDevice` SET future poll at `0x1000959e0` consumes it around `0x100095be2`: for target metadata selector `+0x3A==2`, values 0 and 1 select the two complemented additive checksum layouts, and value 2 skips checksum insertion. **CONFIRMED candidate dataflow/effect.** The raw URI-to-poll edge remains strongly supported, so the request-to-device route as a whole is not fully proven.

## `checksumtype` propagation

The renderer sets `checksumtype` in `SendMsg`. No native read/use of this field has been identified in the candidate SET poll or Base/`DJDevice` Feature future chain. The native checksum insertion is controlled by `dangledevtype`, not a proven `checksumtype` consumer.

- **CONFIRMED:** Frontend sets the protobuf field.
- **UNKNOWN:** Any native consumer or operational effect of `checksumtype`.

Do not infer semantics from frontend enum labels or treat it as interchangeable with `dangledevtype`.

## Response and error conversion

- **CONFIRMED frontend SET conversion:** `ResSend.err` becomes a boolean based on whether the error string is empty.
- **CONFIRMED frontend GET conversion:** `ResRead.msg` becomes a Buffer; missing response data becomes `undefined`.
- **UNKNOWN native conversion:** A complete instruction-level mapping of all libusb errors, sender errors, lookup failures and internal handler failures into `ResSend.err` / `ResRead.msg` has not been recovered. Tonic transport errors are separately delivered through the frontend callback error argument.

## Device-specific sender chain

Existing Phase 4.2/5C evidence records:

- `3151:4010`: support match → metadata tags `(2,2)` → `BleHid` API wrapper → `DJDevice::new` selects `HidLibusb`.
- `3151:4011`: support match with `dongle_common=true` → metadata tags `(1,2)` → `YZWDangleCommon` wrapper → `DJDevice::new` selects `HidLibusb`.

Both devices therefore share the same generic libusb Feature methods if the raw RPC handler reaches the recovered Feature helper. The raw URI-to-Tonic-wrapper poll mapping is **CONFIRMED**; the final wrapper-to-handler/helper edge remains **UNKNOWN** because of the indirect service vtable call.

## Final RPC answer

The renderer raw RPC contract and native URI routing are recovered to the Tonic wrapper-poll level: sendRawFeature selects descriptor `0x1003cd908` / poll `0x1000a0f50`; readRawFeature selects descriptor `0x1003cd948` / poll `0x100092ec0`. The wrapper-to-handler call still crosses an indirect vtable at `0x1000ac5ed`, whose route-specific target was not recovered. Candidate device-path helper polls (`0x1000aa0a0`, `0x1000a3ef0`) perform the expected Feature lookup/dispatch but are not uniquely bound to the raw URI. `checksumtype` native consumption and complete response/error conversion also remain unknown.

No driver process was launched, no RPC was sent, and no hardware I/O was performed.
