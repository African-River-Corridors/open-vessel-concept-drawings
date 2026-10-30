#!/usr/bin/env bash
# Run the web smoke test: build the web app in test mode (it exposes a read-only hook the test uses),
# then drive headless Chrome against it. Needs the WebAssembly package (bash tools/build_web.sh),
# Chrome, and the CLI built at target/debug/ovd (cargo build -p ovd-cli).
set -euo pipefail
cd "$(dirname "$0")/.."
[ -f wasm-pkg/ovd_web_bg.wasm ] || { echo "e2e-web: no app/wasm-pkg (run bash tools/build_web.sh)" >&2; exit 1; }
export OVD_CLI="${OVD_CLI:-$(cd .. && pwd)/target/debug/ovd}"
[ -x "$OVD_CLI" ] || { echo "e2e-web: no CLI at $OVD_CLI (cargo build -p ovd-cli)" >&2; exit 1; }
npx vite build --config vite.web.config.ts --mode test --outDir dist-web-test
node --test e2e/web-smoke.test.mjs
