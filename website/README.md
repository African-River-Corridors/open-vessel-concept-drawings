# Website and docs

The OVD website and docs. Live at **https://www.africanrivercorridors.com/ovd** (docs at
`/ovd/docs`).

## How it is served

- `.github/workflows/website.yml` builds this folder and deploys it to GitHub Pages on each push to
  `main` that touches `website/`, `docs/` or a root `.md` file. Pull requests build and check the
  links but do not deploy.
- The ARC website (Vercel) proxies `/ovd` and `/ovd/*` to the Pages site with rewrites. There is
  one source, this folder, and no copy on the ARC side.
- The ARC site redirects `/path/` to `/path` and `/page.html` to `/page`. So every URL here is an
  absolute `/ovd/...` path with no trailing slash and no `.html`. A relative URL would resolve
  against the wrong folder at `/ovd`. GitHub Pages serves `page.html` for `/page`.
- The ARC rewrites map two folder URLs to their index page: `/ovd` and `/ovd/docs`. Do not add
  other folder index pages; name the page instead (`app.html`, served as `/ovd/app`).
- The web app page is `app.html`, served at `/ovd/app`. Its files (JS, CSS, font, `.wasm`) are
  under `/ovd/app-files/`, never `app/`: a folder named `app/` next to `app.html` makes GitHub
  Pages redirect `/app` to `/app/` on its own host, which breaks the proxy.
- `build.py --app-dist app/dist-web` copies the web build in. Without the flag the "coming soon"
  `app.html` stays, so the docs build with no Rust or Node.
- The `github.io` address serves the same files, but its links point to `/ovd/...`, so browse the
  ARC address.

## Layout

| Path | What it is |
| --- | --- |
| `index.html`, `site.css`, `tokens.css` | The home page. Plain HTML and CSS, no build step. |
| `app.html` | The "web app, coming soon" page. With `--app-dist`, the web app itself (files in `app-files/`). |
| `assets/` | Logo, illustrations. `favicon.svg` is at the root. |
| `mkdocs.yml`, `hooks.py`, `overrides/` | The docs: mkdocs-material with the OVD theme. |
| `docs-theme/ovd.css` | The docs theme over `tokens.css`. |
| `build.py` | Builds everything into `_site/`. |
| `check_links.py` | Fails on any broken internal link in `_site/`. |

The picture in "Made in the tradition" is a screenshot of the desktop app with LRG Gas 89 open
(`catalogue/reference_vessels.yaml`, id `lrg-gas-89`). `assets/examples/lrg-gas-89.vdesign` is its design
file: open it in the app, show Drawings, Side elevation, theme Light, and capture the window. Keep the
full PNG and the two WebP sizes in `assets/screenshots/`.

The docs are the repo's own Markdown: README, ROADMAP, GLOSSARY, CONTRIBUTING, CODE_OF_CONDUCT,
SECURITY, CHANGELOG, `docs/adr/` and `docs/specs/`. `build.py` copies them into `_docs/` at build
time. Edit the files where they are; never in `_docs/`. A new ADR or spec joins the nav by itself.
Links between these files keep working; a link to any other repo file goes to GitHub.

## Build it locally

```bash
python3 -m venv website/.venv && website/.venv/bin/pip install -r website/requirements.txt
website/.venv/bin/python website/build.py && python3 website/check_links.py
```

The pins in `requirements.txt` are exact. mkdocs is pinned below 2.0, which drops the plugin and
theme systems this build uses.
