# Phase 5C — Frontend RPC dispatch resolution

Date: 2026-11-01  
Scope: static correlation of renderer RPC definitions, native Tonic routes, request-field consumers, and candidate handler futures.

## Executive result

- **CONFIRMED:** The renderer defines distinct unary routes `/driver.DriverGrpc/sendRawFeature` and `/driver.DriverGrpc/readRawFeature`; these are not aliases of `sendMsg` and `readMsg`.
- **CONFIRMED:** `sendRawFeature` uses `SendMsg → ResSend`; `readRawFeature` uses `ReadMsg → ResRead`.
- **STRONGLY SUPPORTED:** The native SendMsg-shaped poll function at `0x1000aa0a0` is the raw SET handler, and the ReadMsg-shaped poll at `0x1000a3ef0` is the raw GET handler.
- **UNKNOWN:** The exact Tonic router function-pointer → poll-function mapping that uniquely assigns those poll bodies to the raw URI rather than ordinary `sendMsg`/`readMsg`.
- **CONFIRMED:** The candidate SET handler performs device-path lookup and consumes `dangledevtype`; the candidate GET handler performs device-path lookup.
- **UNKNOWN:** The exact `devicepath` field/record member compared in the final iterator predicate; `checksumtype` native consumption; complete error/response conversion.

## Renderer RPC contract

Artifact: `../.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js`.

The bundled gRPC method descriptor is in the minified bundle around source line 1554:

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

At bundled line 1556 the renderer helper constructs `SendMsg`, sets `devicepath`, `msg`, `checksumtype`, and `dangledevtype`, then calls `client.sendRawFeature`. It converts `ResSend.err` to a boolean (`true` when the error string is empty). The read helper constructs `ReadMsg`, sets `devicepath`, calls `client.readRawFeature`, and converts `ResRead.msg` to a `Buffer`, or returns `undefined` when no response is supplied. **CONFIRMED frontend behavior.**

The `SendMsg` protobuf default values are empty `devicepath`, empty message bytes, `checksumtype=0`, and `dangledevtype=0`. `ReadMsg` carries `devicepath`.

At bundled line 1564, the YZW common Feature helper calls `sendRawFeature` and then `readRawFeature` for the request/response sequence. The frontend computes `dangledevtype` as 2 for a listed dongle mouse, 1 for a listed dongle keyboard, and 0 otherwise. **CONFIRMED for this frontend helper.** This does not mean every YZW operation uses the raw methods.

## Native Tonic routes and candidate handlers

The native Mach-O contains generated Tonic service route symbols for:

- `sendRawFeatureSvc<GrpcServer>` with `SendMsg` and `ResSend`;
- `readRawFeatureSvc<GrpcServer>` with `ReadMsg` and `ResRead`.

The closure drop-glue symbols are folded/shared with the ordinary `sendMsgSvc` and `readMsgSvc` closure types. That shared drop glue is not a unique route-to-handler edge.

Candidate handler futures:

| Candidate | VA | Observed request shape and behavior | Evidence |
|---|---:|---|---|
| SendMsg-shaped poll | `0x1000aa0a0` | Reads `dangledevtype`, normalizes it, calls `get_dev_from_path` at `0x1000aa1da`, then invokes Base Feature SET slot `+0x30`. | **CONFIRMED candidate body** |
| ReadMsg-shaped poll | `0x1000a3ef0` | Calls `get_dev_from_path` at `0x1000a3f5d`, then invokes Base Feature GET slot `+0x38`. | **CONFIRMED candidate body** |

`DriverGrpcServer<GrpcServer>::call` is at `0x100048d50`. It performs HTTP/2 gRPC path routing using static route data in the `0x1003cd8a8..0x1003cd968` region. The raw and ordinary routes share folded closure drop-glue addresses (`0x100040a60` for send-shaped and `0x100040990` for read-shaped wrappers). No unique static pointer edge from the raw URI's route object to `0x1000aa0a0` / `0x1000a3ef0` has been recovered.

Accordingly, the raw URI → candidate poll assignment remains **STRONGLY SUPPORTED**, not CONFIRMED. Do not upgrade this based solely on shared drop glue or matching protobuf request types.

## Device-path lookup

`iot_driver::grpc_server::get_dev_from_path` is at `0x1000603a0`. It queries `DJDeviceManger::get_dev_list`, iterates the `Arc<DJDevInstance>` collection, and captures the supplied `devicepath` string in the iterator/filter closure environment around `0x10006041d`. The matching `Arc<DJDevInstance>` is retained and returned to the handler.

The final string-comparison predicate and exact `DJDevInstance` field compared with `devicepath` are not resolved. **CONFIRMED:** lookup is manager-list based and takes the supplied path. **UNKNOWN:** which concrete record field and comparison routine establish equality.

## Request-field consumers

### `dangledevtype`

The SendMsg-shaped poll reads the field around `0x1000aa13a` and normalizes values:

- 1 → 1
- 2 → 2
- all other values → 0

The normalized byte is passed in RCX to Base vtable slot `+0x30` at `0x1000aa799` and `0x1000aa7d7`. The Base `pure_send_feature` factory captures it at future environment offset `+0x398` (`0x100038e78` for the BleHid factory; corresponding YZWDangleCommon factory `0x1000610d0`). The Base SET future propagates it into the nested `DJDevice::send_feature` future (`0x100091101..0x100091124` and `0x10008ed5a..0x10008ed74`).

The `DJDevice` SET future poll at `0x1000959e0` consumes the selector around `0x100095be2` to choose checksum generation/insertion layout. For metadata selector `+0x3A=2`: values 0 and 1 use the two complemented additive checksum layouts; value 2 skips checksum insertion. **CONFIRMED candidate native dataflow/effect.**

### `checksumtype`

The renderer writes `checksumtype` into the protobuf request. No consumer for this field has been recovered in the candidate native SET path or the Base/DJDevice Feature future chain. Its native effect remains **UNKNOWN**. Do not infer behavior from frontend enum labels or confuse it with `dangledevtype`.

### `devicepath`

Both candidate handler bodies pass the path into `get_dev_from_path`. The exact comparison target is still unresolved, as above.

## Response and error conversion

**CONFIRMED frontend conversion:** SET returns a boolean based on whether `ResSend.err` is empty; GET returns the `ResRead.msg` bytes as a Buffer or `undefined`.

**UNKNOWN native detail:** a complete instruction-level mapping from every libusb error / internal handler error to `ResSend.err` and `ResRead.msg`, including all fallback branches, has not been recovered. Tonic transport errors are separately surfaced through the frontend callback error parameter.

## Device-specific candidate transport path

The prior Phase 4.2 analysis recovered these constructions:

- `3151:4010`: support match → metadata tags `(2,2)` → `BleHid` API wrapper around `DJDevice` → `DJDevice::new` selects `HidLibusb`.
- `3151:4011`: support match with `dongle_common=true` → metadata tags `(1,2)` → `YZWDangleCommon` wrapper around `DJDevice` → `DJDevice::new` selects `HidLibusb`.

Both underlying `DJDevice` objects use the same HidSender Feature methods if the candidate raw routes are assigned to these poll bodies. The route-specific edge remains **STRONGLY SUPPORTED**.

## Final RPC answer

The raw RPC contract and frontend field handling are confirmed. The native candidate SET/GET poll bodies and their device lookup / Feature method chains are recovered. However, the exact raw URI → poll function mapping is not conclusively proven because Tonic route wrappers share folded closure types with ordinary routes and no unique router pointer edge has been established. Therefore raw RPC dispatch is **not fully/conclusively mapped**.
