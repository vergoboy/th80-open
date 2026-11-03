# Phase 13.11 — Rongyuan API and Frontend Flow

## 1. Executive result

The protected EPOMAKER frontend contains **two distinct Rongyuan service modules**. The older `api.rongyuan.tech:3814/v1` module includes `/downloadDevUpgradeFile`, but the inspected active firmware-update routine uses a different module and host: `api2.rongyuan.tech:3816`.

The active update flow is:

1. POST the current device type ID as `dev_id` to `/api/v2/get_fw_version`.
2. Read the returned firmware version metadata, including `file_path`.
3. GET `/download/{encoded file_path}` as an array buffer.
4. Inflate the raw-deflate payload and pass the decompressed bytes to the device-specific upgrade method.

This is a firmware distribution flow. It does not expose an SDK download operation in the inspected frontend. No API request was made: the actual runtime `deviceType.id` and server-returned `file_path` are not available from the static release archive, and no public API contract establishes that archive ID 742 is the exact `dev_id` value.

## 2. Protected frontend and configuration identity

Frontend bundle:

- Path: `/home/vergoboy/Projects/projects/th80/.phase3-work/mac/EPOMAKER Driver 1.0.0/EPOMAKER Driver.app/Contents/Resources/app/dist/index.js`
- SHA-256: `7d53737095fe5756fb0a18d5aecb7593555b9bb61a9d5d160ae0b988694c1e87`
- Inspection date: 2026-11-01

Company configuration:

- `CurrentCompany.json` says `currentCompany: EPOMAKER`; SHA-256 `85cd3b9278330f0f9a69292168cef089f118d8c150610627ac0d6954b4729541`.
- `company/company_EPOMAKER/CONFIG.json` says `company: EPOMAKER`, `displayName: EPOMAKER`, `blockCloud: false`; SHA-256 `f96e1c6ad25f885edb4abbf87b4b0af7de35b204b5ad2e7bcaee0e12f0353c4b`.

The bundle and configuration were read only.

## 3. Active firmware-update service: `api2.rongyuan.tech:3816`

Relevant minified bundle offsets are byte offsets in the protected `dist/index.js` file.

### 3.1 Base URLs and version lookup

At offset `3,842,634`, the active service defines:

```js
exports.server_path = "https://api2.rongyuan.tech:3816/api/v2";
```

At offset `3,842,695`, its download base is:

```js
const u = "https://api2.rongyuan.tech:3816/download";
```

At offset `3,843,939`, the firmware-version function is equivalent to:

```js
getFwVersion = (deviceTypeId) =>
  POST(server_path + "/get_fw_version", { dev_id: deviceTypeId }, { timeout: 8000 });
```

The helper returns an error wrapper when `data.code` is nonzero and otherwise exposes the response data. The call site in `getFwVersion()` (within the firmware-upgrade store) obtains `currentDev.deviceType.id` and passes it as the `dev_id` value.

### 3.2 Version metadata consumed by the frontend

The update decision code reads:

- `version_str`
- `lowest_app_version_str`
- `file_path`

It compares the returned firmware version against `currentDevVersion`, checks the minimum compatible application version, and retains the response as `fwVersion` when an update is required. `file_path` is then passed to the download method.

The archive directory label contains `(ID 742)`, but the frontend does not hard-code `742`. The static code sends `currentDev.deviceType.id`; equality between that runtime ID and the archive's 742 label is not established without the actual local device database/API response.

### 3.3 Active download request and response handling

At offset `3,845,076`, the active service's `downloadDevUpgradeFile` is equivalent to:

```js
httpDownload("/" + encodeURI(filePath))
```

`httpDownload` builds a GET URL from:

```text
https://api2.rongyuan.tech:3816/download/{encodeURI(file_path)}
```

It uses Axios `GET` with `responseType: "arraybuffer"`. This helper does not add a product ID, `company` field, Cookie header or platform header to the download request in the inspected code. The version POST helper has an 8-second timeout; the GET helper shown does not set an explicit timeout.

At offset `3,888,443`, the active `upgradeDev()` path calls `inflateRawSync(e.data)` and passes the resulting bytes to `currentDev.upgrade(...)`. The decompression is raw DEFLATE handling, not evidence of an SDK or firmware source archive. The inspected code does not show a cryptographic firmware checksum/signature verification step in this path.

Error handling in the GET helper converts an HTTP response/error into an error result; `upgradeDev` checks the result's `errCode` before continuing. The final device upgrade call is not executed in this investigation.

### 3.4 Active call sequence

```text
upgradeDev()
  └─ checkFwVersion()
       └─ getFwVersion()
            └─ POST https://api2.rongyuan.tech:3816/api/v2/get_fw_version
                 JSON: { "dev_id": currentDev.deviceType.id }
                 response metadata includes version_str,
                 lowest_app_version_str and file_path
  └─ downloadDevUpgradeFile(fwVersion.file_path)
       └─ GET https://api2.rongyuan.tech:3816/download/{encoded file_path}
            responseType: arraybuffer
  └─ inflateRawSync(response.data)
  └─ currentDev.upgrade(decompressedBytes, ...)
```

Classification: `FRONTEND_ONLY` for this request/response contract. It is reconstructed from static frontend code; no server response or live device context was obtained.

## 4. Separate older service: `api.rongyuan.tech:3814/v1`

The bundle also configures `kSeverAddr = "https://api.rongyuan.tech:3814/v1"` and contains a separate exported `webService.downloadDevUpgradeFile` implementation.

That implementation posts to `/downloadDevUpgradeFile` with a body equivalent to:

```json
{
  "deviceId": "<function argument>",
  "company": "EPOMAKER"
}
```

Its helper sets `responseType: "stream"`, `withCredentials: true`, a 20-second timeout, and writes the stream to a local file named `devUpgrade_<argument>`. It reports progress from the response `content-length`. The corresponding generic POST helper also adds a `Cookie` header from the stored session and a `platform` header, but the stream helper for this endpoint does not explicitly add those headers; it does set `withCredentials: true`.

**Important distinction:** the inspected `upgradeDev()` method calls `a.webUpgradeService.downloadDevUpgradeFile`, whose implementation is the `api2.rongyuan.tech:3816/download/{file_path}` GET described above. It does not call the separate `webService.downloadDevUpgradeFile` POST implementation. The two methods share a name but have different hosts, methods, request bodies and response handling. Do not conflate them.

The `api.rongyuan.tech:3814/v1` module is present in the bundle and may support other app workflows; this report only establishes that it is not the method called by the inspected active `upgradeDev()` flow.

## 5. Legacy updater executable's API generation

The Windows updater contains these strings:

- `http://api.rongyuan.tech:3000/v1/downloadDevUpgradeFile`
- `http://api.rongyuan.tech:3000/v1/admin_upload_firmware`
- `/getVersion`
- Model fields including `devId`, `version`, `lowestAppVersion`, `deviceId` and `devices`.

This is a third API generation/endpoint set: plain HTTP on port 3000, separate from the Mac frontend's `api.rongyuan.tech:3814` and active `api2.rongyuan.tech:3816` services. It appears in the updater's compiled string data; no request was sent and no runtime behavior was observed. See [updater forensics](PHASE13_11_UPDATER_STATIC_FORENSICS.md).

## 6. Product ID 742 and request parameters

Directly confirmed:

- `742` appears in the V504 archive directory name.
- The active frontend uses `currentDev.deviceType.id` as `dev_id` in the version POST.
- The returned `file_path` is used as a path component in the active GET download.
- The older `/downloadDevUpgradeFile` POST body uses `deviceId` and `company: EPOMAKER`.

Not confirmed:

- That the runtime `deviceType.id` equals 742.
- The actual `file_path` for the V504 record.
- Whether the server's returned `file_path` names the exact ZIP's `X` image, a compressed copy, or another firmware package.
- Whether a server response includes any additional metadata not referenced by the frontend.
- Any SDK source or development package response.

Do not substitute 742 for a missing runtime parameter merely because it appears in the archive folder name.

## 7. API safety and public metadata investigation

No endpoint was called. Although `get_fw_version` appears to be a version lookup, the actual `dev_id` is not proven to be 742 and the active flow requires the server's returned `file_path` before a download request can be constructed. The old `/downloadDevUpgradeFile` endpoint's argument semantics are not established. The service documentation/authentication contract was not publicly discoverable in the searches performed. Consequently, no request was necessary or sufficiently grounded to meet the user's read-only/exact-parameter condition.

Public searches on 2026-11-01 for `api.rongyuan.tech`, `api2.rongyuan.tech`, the ports and endpoint names, TH80/V504/product ID 742 and Yichip SDK file names did not return API documentation, public manifests, SDK catalogues or a verifiable development package. Public EPOMAKER pages document end-user driver/firmware distribution, not a vendor SDK release channel:

- [EPOMAKER Driver page](https://epomaker.com/blogs/software/epomaker-driver)
- [EPOMAKER Firmware catalogue](https://epomaker.com/blogs/firmware)
- [Yichip YC3121 public repository](https://github.com/Yichip-Microelectronics/YC3121)

These public pages were inspected 2026-11-01. No API response body, HTTP headers, content type or current endpoint availability is claimed.

## 8. Distribution conclusion

The frontend reveals a version-metadata lookup followed by firmware-file retrieval and decompression. The active API flow is a firmware delivery mechanism, not an SDK source distribution mechanism. The second/older `/downloadDevUpgradeFile` POST and the Windows updater's port-3000 endpoint show historical/parallel Rongyuan API surfaces, but neither contains evidence of SDK source delivery.

The exact V504 archive itself has no SDK files, and the updater has no embedded SDK resources. The original V504 development package remains unrecovered; the strongest classification remains `SAME_SDK_LINEAGE` based on Phase 13.10's Yichip YC3121 evidence, not on the distribution API.

## 9. Cross-references

- [V504 archive analysis](PHASE13_11_V504_DISTRIBUTION_AND_ARCHIVE_ANALYSIS.md)
- [Updater static forensics](PHASE13_11_UPDATER_STATIC_FORENSICS.md)
- [Findings and next steps](PHASE13_11_FINDINGS_AND_NEXT_STEPS.md)
- [Phase 13.10 source package recovery](PHASE13_10_V504_SOURCE_PACKAGE_RECOVERY.md)
