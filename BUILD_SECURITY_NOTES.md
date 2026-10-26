# Windows build and Chrome download warning

This project is an **unsigned development build**. A Chrome Safe Browsing warning is not, by itself, proof of malware, nor is an unsigned EXE proof of safety.

## What changed

- Upload the contents of `dist/` directly to GitHub Actions Artifacts; no nested ZIP.
- Record EXE SHA-256 in `SHA256SUMS.txt`.
- Print Authenticode signature status (expected `NotSigned` for development builds).
- Run Microsoft Defender on the staged EXE and fail the build if the scan cannot run or detects a matching threat.
- Add a restrictive Tauri CSP; verify the app's UI on Windows before release.

## How to test

1. Push this source to a test branch, then run the Windows workflow.
2. Confirm TypeScript, Rust checks/tests, frontend and EXE builds succeed.
3. Confirm Defender scan step succeeds and artifact contains `RustOfficeModern.exe`, `README.md`, `SHA256SUMS.txt` (not a nested ZIP).
4. Download artifact. On Windows, verify SHA-256 using `Get-FileHash .\RustOfficeModern.exe -Algorithm SHA256` and compare to `SHA256SUMS.txt`.
5. Run in a disposable Windows VM with WebView2 installed; verify Word, Excel, PowerPoint, opening/saving and UI rendering.
6. If Chrome still blocks the artifact, inspect Safe Browsing details and scan the exact built EXE; do not disable protection.

## Limitations

- No Windows runner, Rust toolchain, or executable execution is available in the editing environment; CI and runtime success are **not** claimed.
- Defender scan is one signal, not a comprehensive security audit.
- Dependency versions are not fully locked; add and commit `package-lock.json` (generated with online npm install) and `src-tauri/Cargo.lock` for reproducible builds. Then switch CI from `npm install` to `npm ci`.
- A signed production release requires a legitimate code-signing certificate and secure key handling.
