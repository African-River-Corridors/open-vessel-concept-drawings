// A tiny static server that behaves like the live site: the page is at /ovd/app (no slash) and its
// files are under /ovd/app-files/. Serves .wasm as application/wasm.
import http from "node:http";
import fs from "node:fs";
import path from "node:path";

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".css": "text/css",
  ".wasm": "application/wasm",
  ".woff2": "font/woff2",
  ".svg": "image/svg+xml",
};

export function serve(dir, port = 0) {
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, "http://localhost").pathname;
    let file = null;
    if (url === "/ovd/app") file = path.join(dir, "index.html");
    else if (url.startsWith("/ovd/app-files/")) {
      const rel = path.normalize(url.slice("/ovd/app-files/".length));
      if (!rel.startsWith("..")) file = path.join(dir, rel);
    }
    if (!file || !fs.existsSync(file) || !fs.statSync(file).isFile()) {
      res.writeHead(404).end("not found");
      return;
    }
    res.writeHead(200, { "content-type": TYPES[path.extname(file)] ?? "application/octet-stream" });
    fs.createReadStream(file).pipe(res);
  });
  return new Promise((resolve) =>
    server.listen(port, "127.0.0.1", () => resolve({ server, origin: `http://127.0.0.1:${server.address().port}` })),
  );
}
