# Calibration source documents live in S3, not in git

Calibration draws on published GA drawings, spec sheets and theses, most of them "all rights reserved"
marketing or academic material. We store the files in a private S3 location and keep only a record of
each one in git — URL, retrieval date, checksum, licence and the particulars read from it. The repo stays
small and holds no third-party copyright material, and every calibration figure stays traceable to its
source.

## Amendment (2026-10-22): checksums of web sources

The record of a source holds a checksum. For a file kept in S3 it is always present. For a web page, the
checksum must be **attempted**, but it need not succeed: a page can be down, refuse a request, or have a
certificate that does not verify, and a page that changes gives a new hash. Each web source records which
happened: `retrieved` and `sha256` when it was achieved, or `checksum_attempted` (the date) and
`checksum_failed` (the reason) when not. A missing checksum is not a warning and not a gate.
