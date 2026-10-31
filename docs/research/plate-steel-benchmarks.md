# Research note: published barge midship sections and steel lists (plate-level benchmark)

Retrieved 2026-10-31. For spec 0013 ("Calibration", D16) and issue #751. Concept and preliminary design only (ADR 0011);
never approval. Data: `calibration/plate_steel_benchmarks.yaml` (each value with its document, table or section, URL,
retrieval date and SHA-256).

## How to read this note

- Every value in the data file is in the cited text, which we read. A value read off a drawing is not used unless the
  number is printed legibly.
- A benchmark is compared with, never tuned to (D16). It is not calibration of the lightship: none of these gives a
  weighed steel weight of a built barge.
- Licence: values are taken as facts with attribution. No figure, drawing or long text is copied into the repo.

## Result

Five usable sources were found. None is a full plate-level midship section **and** steel list of a built barge near
140 x 29 m. Ranked by fit:

| Rank | Source | Vessel | Plate-level detail | Steel list | Licence |
|---|---|---|---|---|---|
| 1 | Salazar-Dominguez et al., J. Mar. Sci. Eng. 2021, 9, 99 | Built LR ocean deck barge, 122.45 x 30.5 x 7.6 m | Deck and bottom plate, deck and bottom longitudinal, frame and stiffener spacing, bulkhead positions, hull girder modulus | No | CC BY 4.0 |
| 2 | Victor, EMship master thesis, Liege, 2021 | Design of an inland container push barge, 83 x 11.45 x 2.8 m | Full midship section, 17 items (initial and optimised), spacings | Yes, 17 rows, 379.56 t | MatheO open access, non-commercial; see below |
| 3 | Crowley, NINILCHIK spec sheet | Built ABS Ocean deck barge, 260 x 72 x 16 ft | Every main plate, stiffener and transverse chord size | No | Owner's spec sheet |
| 4 | Cameron et al., USCG MSC, 1997 | Two built inland tank barges, 275 and 290 x 54 ft | Deck plate and deck longitudinal only; fleet deck range | No | US Government work |
| 5 | Setyaji et al., Jurnal Teknik Perkapalan 13(1), 2025 | 330 ft coal barge, 100.58 x 27.43 x 6.10 m | Stiffener sizes and counts; plate thickness only in an illegible figure | Lightship 1669 t only | CC BY-SA 4.0 |

What each gives the plate model:

- **Rank 1** is the best size and service match (ocean deck barge, B = 30.5 m, L = 120 m). It checks the deck and
  bottom panels and the hull girder modulus. Its other members are colour-coded only (Figure 7). Class is Lloyd's
  Register, so the rule minima differ from the ABS rule set.
- **Rank 2** is the only source with both a plate-level midship section and a steel weight list by item from the same
  model. It can test the plate model's arithmetic end to end on an inland double-hull hull. It is a student design,
  small, and not weighed.
- **Rank 3** is a built barge in OVD's own coastal rule family (ABS Ocean), with every main member sized. It gives no
  spacing, so a panel check needs an assumed spacing.
- **Rank 4** is a lower bound on inland deck plating (5/16 in on 23 of 26 barges) and a fleet range of deck stiffener
  spacing (23.5 to 26 in).
- **Rank 5** has the best beam (27.4 m) and a lightship, but no readable plate thickness.

**Licence of rank 2.** The MatheO cover page allows reading, downloading, indexing and use as data for software for any
lawful purpose with full attribution. It forbids commercial use of the document and asks for the author's consent to
modify or summarise it. We take only numbers, as facts, with the full citation. The maintainer should confirm this
reading before the data is used in code.

## Search line

Searched 2026-10-31, about 90 minutes, web search plus direct reads. What was searched, where, and why each candidate
failed:

- **Open-access journals (MDPI, DOAJ, Indonesian OJS journals):** rank 1 and rank 5 found. Other Indonesian barge
  papers (Undip, ITS, BRIN, Polibatam) give stresses or finite element results without a scantling table, or show
  thickness only in figures.
- **University theses:** Liege MatheO (EMship) gave rank 2; other EMship theses found were a container ship, a tanker
  and a bulk carrier, not barges. ITS Surabaya full texts (for example repository 55556, a heavy-cargo barge) return
  401 without a login. A University of Rijeka record (a bunker barge drawing, per the search summary) returned HTTP 418 to our client. A TU
  Delft repository search found midship optimisation theses of an offshore vessel and a dredger, not barges.
- **Course notes:** MIT OCW 13.122 (2003) problem set 1 has a 440 ft barge midship section, but it is an idealised
  exercise (one plate thickness, 0.5 in, for every strake), so it is not a benchmark. MIT 2.701 (2014) project 7 is a
  product tanker. No Webb Institute barge handout was found.
- **US Government:** the USCG MSC 1997 study gave rank 4; its Appendix B (a worked tank barge example) is not in the
  served PDF. USCG plan review guideline PRG C1-12 (inland tank barge structures) and the ROSA P PDF both returned
  "Access Denied" to direct download (the study was read through a second fetch). The George G. Sharpe 1993 reports for
  the USCG on single and double hull inland tank barges were not found online. US Army Corps of Engineers barge
  solicitations name dimensions and class; their drawing packages sit behind SAM.gov attachments and were not reached.
  NTSB barge reports turned up by the search (crane barge, deck barge, tank barge I.O.S. 3301) were not read in full;
  their search summaries show no scantling table.
- **Ship Structure Committee:** no SSC report with a barge midship section was found. The search returned SSC-439
  (high-speed craft) and SSC-446 (design standards) by title only; neither was read.
- **Owner and broker spec sheets:** Crowley NINILCHIK gave rank 3. Crowley 400-L (122 x 30.5 x 7.6 m, lightship
  3,505 t) and the Crowley HDB barges (122 x 36.5 x 7.6 m, lightship 4,256 t) give no scantlings. BOABARGE 43/44
  (91.44 x 30.48 x 6.10 m, DNV 1A1) gives web frame spacing 2.286 m and stiffener spacing 0.686 m but no thickness.
  These are lightship cross-checks at most, not plate-level benchmarks.
- **EU inland projects:** INBAT (2001 to 2004, lightweight push barges): the CORDIS and TRIMIS project pages, as
  summarised by the search, list objectives and results but no deliverable with scantlings; no deliverable was found. The WWF Danube ship design report (2009, "Hull weight considerations") was found only on Scribd and
  was not read.
- **Other:** a 5000 t work barge paper (West African Journal of Industrial and Academic Research, 2013) was found only
  as a Scribd copy and an AJOL record; it was not read, so it is not cited.

## Gaps and next steps

- No source gives a plate-by-plate steel weight of a **built** barge. Rank 2 is the closest, as a design.
- A legible copy of the 330 ft midship section (rank 5), or the authors' data of rank 1 ("request to corresponding
  author", per its Data Availability Statement), would close most of the gap. Asking is the maintainer's call.
- Paid sources not bought: Billingsley, "Structural Design of River Barges", Marine Technology 21(2), 1984 (SNAME);
  ship structures textbooks with worked barge examples.
