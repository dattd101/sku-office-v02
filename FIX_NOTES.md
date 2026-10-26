# Editor white-screen repair

The manager is now a dedicated `/` route and all editors load `/editor/?session=<uuid>`. Previously both routes shared a statically exported `index.html` and the initial render depended on `window.location.search`, causing a server/client render mismatch (hydration) for editors.

The editor now renders a loading state and an explicit session failure message, plus a page-level JavaScript error reporter. The original document format handlers, recovery, and save functionality are preserved.

## Build

On Windows with Node 22 and Rust installed: `npm install`, `npm run build`, `npm run desktop:build`. The build invokes Next.js export automatically. Verify `out/editor/index.html` exists. Test new/open/recovery for all three editors.

Note: This patch has not been runtime-tested on Windows.
