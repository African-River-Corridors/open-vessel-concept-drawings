#!/usr/bin/env bash
# Run the web smoke test: build the web app in test mode (it exposes a read-only hook the test uses),
# then drive headless Chrome against it. Needs the WebAssembly package (bash tools/build_web.sh),
# Chrome, and the CLI built at target/debug/ovcd (cargo build -p ovcd-cli).
set -euo pipefail
cd "$(dirname "$0")/.."
[ -f wasm-pkg/ovcd_web_bg.wasm ] || { echo "e2e-web: no app/wasm-pkg (run bash tools/build_web.sh)" >&2; exit 1; }
export OVCD_CLI="${OVCD_CLI:-$(cd .. && pwd)/target/debug/ovcd}"
[ -x "$OVCD_CLI" ] || { echo "e2e-web: no CLI at $OVCD_CLI (cargo build -p ovcd-cli)" >&2; exit 1; }
npx vite build --config vite.web.config.ts --mode test --outDir dist-web-test
node --test e2e/web-smoke.test.mjs
