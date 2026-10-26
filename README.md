# Rust Office v0.5.1 — Manager + independent editor windows

Windows desktop prototype using **Next.js 15**, **React 19**, **Tailwind CSS 3**, **Tauri 2**, and **Rust**. Word, Excel and PowerPoint open as separate windows from the Office Manager. Each editor is identified by a UUID session ID.

## What's implemented

- Office Manager dashboard with new Word/Excel/PowerPoint, Open, recent/recovery list, resume, and remove recovery entry.
- One Tauri EXE and multiple WebviewWindow editors. Switching applications does **not** replace the current editor state.
- Rust-managed document snapshots under the Windows per-user application data directory (`.../dev.rustoffice.modern/recovery/*.json`). Snapshots are debounced at ~850ms and also attempted on window close. Manager scans persisted snapshots after restart.
- Explicit Save/Save As to DOCX/XLSX/PPTX, plus simple import/export based on the existing v0.4 Rust format modules.
- Existing Ribbon, Word plain-text editor, Excel grid and PowerPoint basic slide editor.
- GitHub Actions for Windows x64, TypeScript check, Rust check/tests, Tauri `--no-bundle`, and portable EXE artifact.

## Important limitations

This is an **MVP**, not an Office 2019 replacement. Word formatting, Excel formulas and advanced slide features are not implemented. Opening and re-saving complicated Office files may lose unsupported content. Always work on copies of valuable documents. Recovery snapshots are **not** the same as saving to the original file.

Autosave here means **recovery snapshots**, not silent overwriting of the Office file. Snapshots are stored per document and survive normal editor close or application crash *after the latest successful snapshot*. Edits made within the last ~850ms before a hard crash can be lost. The manager retains sessions until explicitly removed.

Removing a recovery entry while its editor is still open is not supported; close the editor first.

## Run locally

Prerequisites: Windows 10/11, Node.js 22, Rust MSVC toolchain, Visual Studio C++ Build Tools and WebView2.

```powershell
npm install
npm run desktop
```

For frontend only:

```powershell
npm run dev
```

The browser alone cannot access Tauri commands; use the desktop app to test manager functionality.

## GitHub Actions portable EXE

### Download the EXE from Actions

1. Push the updated source to `main` or `master`, or select the workflow and click **Run workflow**.
2. Open the successful run (click its run title, not just the Actions list).
3. In **Summary**, use **Download portable ZIP containing RustOfficeModern.exe**, or scroll to **Artifacts** and select **RustOfficeModern-Windows-x64-portable**.
4. Extract the ZIP. It contains `RustOfficeModern.exe`, `README.md`, `SHA256SUMS.txt`, and `BUILD-INFO.json`.

The summary link is added by the updated workflow and will appear on new runs only. Artifacts require GitHub sign-in and are retained for 30 days. If Chrome blocks the download, packaging alone does not resolve the security verdict; inspect the exact warning and build report. The workflow does not automatically obtain a signing certificate.

To build and package locally on Windows:

```powershell
npm install
npm run desktop:build
npm run desktop:package
```

The packaged executable is `dist/RustOfficeModern.exe`. Microsoft Edge WebView2 Runtime must be installed on the destination computer.

Push the project contents, including `.github/workflows/windows-build.yml`, to GitHub. Run **Build Rust Office v0.5 Windows x64** from the Actions tab. The workflow builds `src-tauri/target/release/rust-office-modern.exe` and publishes `RustOfficeModern-Windows-x64-portable.zip` as an artifact.

The executable depends on Microsoft Edge WebView2 Runtime being available on Windows. `--no-bundle` does not create an installer.

## How it works

- Main window (`main`) loads the manager.
- `create_session` or `import_session` creates a recovery JSON with a UUID.
- `open_editor_window` creates or focuses `editor-<uuid>` and loads `index.html?view=editor&session=<uuid>`.
- Editor loads `get_session`, edits in React, and calls `snapshot_session` after changes.
- `save_session` writes the chosen Office format and marks the recovery entry clean.
- On next app launch, `list_sessions` allows recovery and reopening.

## Validation

This source bundle is structurally checked but **not compiled in this environment** because Rust/Cargo and Windows SDK are unavailable here. GitHub Actions is the build verification source of truth.

## v0.5.1 fix — Tauri E0255

The `open_document` helper in `src-tauri/src/lib.rs` is no longer annotated with `#[tauri::command]` and is no longer registered in `generate_handler!`. It is called internally by `sessions::import_session`, which remains the public Tauri command used by the frontend. This prevents Tauri's generated `__cmd__open_document` and `__tauri_command_name_open_document` macros from colliding with the imported helper in `sessions.rs`.

Build validation was not performed locally (Cargo unavailable). The workflow performs `cargo check`, tests, and Windows release build.
