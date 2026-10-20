# App fonts

The app bundles these fonts as woff2. Each file is the released font converted from TTF to woff2 with
fontTools, without other change: the glyphs, metrics and names are the same. The conversion drops
only the `DSIG` table (an old signature table that browsers ignore). The SIL Open Font License 1.1
allows this, and each font keeps its licence in `app/public/licenses/`.

| File | Licence |
|---|---|
| `BarlowSemiCondensed-Regular.woff2` (from `crates/ovcd-draw/standard/fonts/`), `-Medium`, `-SemiBold` | `BarlowSemiCondensed-OFL.txt` |
| `SourceSerif4-Variable.woff2`, `SourceSerif4-Italic-Variable.woff2` | `SourceSerif4-OFL.txt` |
| `IBMPlexMono-Regular.woff2` | `IBMPlexMono-OFL.txt` |
| `ArchitectsDaughter-Regular.woff2` | `ArchitectsDaughter-OFL.txt` |

To convert a TTF again:

```sh
python3 -c "from fontTools.ttLib import TTFont; f = TTFont('X.ttf'); f.flavor = 'woff2'; f.save('X.woff2')"
```

This needs `pip install fonttools brotli`.
