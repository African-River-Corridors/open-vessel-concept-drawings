<!-- unreleased -->
- Rule Set `abs-steel-barges` 1.0.0 (issue #740, spec 0013): the hull steel scantlings of ABS Steel Barges at the current edition, the January 2026 Rules (Part 3 January 2026, Part 5 July 2025), every paragraph re-read in the July 2025 Rules. The formulas of the panel model are now Rules: formula shapes in the core with their coefficients as data, one Rule per formula, each with its paragraph and a worked example; the panel model sizes its members with them, and each member names its Rule Set, edition and paragraph. The numbers are the 2021 ones; 3-2-3/1.1.2 and 1.7 are now 2.1.2 and 2.7. The scantlings take the rule length of 3-1-1/3.1 (96 % of the length on a waterline at 85 % of the depth) in place of the length overall, so every gas hull's steel, lightship, draught, stability and cost move (on the 140 x 29 m example, the panel steel falls from 3,591 t to 3,522 t); 5-1-2/5.17 takes L without the 225 m cap, as its text says. The set joins the coastal default stacks; its Rules are sizing rows, listed as not evaluated. The 76.5 m LNG pushed barge of the test fixtures (`gas_igc_2g_exact`) now has no arrangement that passes every check: the lighter hull fails the light condition's angle of max GZ, so the search shows the closest one.

<!-- design-impact -->
### ticket/740-abs-steel-barges (2026-10-31)

Between `main 0bab67f` and `this branch 6fb3155e`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.53 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.54 m of 6.00 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 3.00 m at the carried draught 2.70 m >= required 0.3 m | freeboard 3.04 m at the carried draught 2.66 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 6.05 m (fluid), max GZ 2.53 m at 29° | GM 6.13 m (fluid), max GZ 2.56 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 6.07 m (fluid), max GZ 2.53 m at 29° | GM 6.14 m (fluid), max GZ 2.55 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.53 m (fluid), max GZ 3.48 m at 28° | GM 12.88 m (fluid), max GZ 3.49 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.63 m (fluid), max GZ 3.47 m at 28° | GM 12.99 m (fluid), max GZ 3.49 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[27].message` | barge form factor at the service speed: Fn 0.191, Cp 0.951, L/B 4.98, B/T 6.00; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 4.98 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.192, Cp 0.951, L/B 4.98, B/T 6.08; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 4.98 (range 7.59 to 16.29) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 3.00 m >= required 0.3 m | freeboard 3.04 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[5].message` | 2 x 1.42 m thrusters need 4.64 m across a 16.30 m beam | 2 x 1.35 m thrusters need 4.50 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 4.64 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | blind distance ahead of the bow over 250 m, light (ES-TRIN 7.02(2)): lift needed 9.88 m of 6.00 m | blind distance ahead of the bow over 250 m, light (ES-TRIN 7.02(2)): lift needed 9.90 m of 6.00 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[23].message` | freeboard 2.21 m at the carried draught 3.49 m >= required 0.3 m | freeboard 2.26 m at the carried draught 3.44 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[24].message` | GM 3.52 m (fluid), max GZ 1.28 m at 25° | GM 3.55 m (fluid), max GZ 1.32 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[25].message` | GM 3.52 m (fluid), max GZ 1.28 m at 25° | GM 3.55 m (fluid), max GZ 1.32 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 8.27 m (fluid), max GZ 2.86 m at 29° | GM 8.53 m (fluid), max GZ 2.87 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 8.31 m (fluid), max GZ 2.86 m at 29° | GM 8.57 m (fluid), max GZ 2.86 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[28].message` | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.70, B/T 4.65; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.69, B/T 4.72; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | freeboard 2.21 m >= required 0.3 m | freeboard 2.26 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].message` | 2 x 2.03 m thrusters need 5.85 m across a 16.30 m beam | 2 x 1.96 m thrusters need 5.72 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].message` | 2 of 2 requested thrusters fit; 2 would need 5.85 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 5.72 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[1]` | The blind distance needs 9.88 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | The blind distance needs 9.90 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.28 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.30 m of 6.00 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[22].message` | freeboard 2.21 m at the carried draught 3.49 m >= required 0.3 m | freeboard 2.27 m at the carried draught 3.43 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[23].message` | GM 5.13 m (fluid), max GZ 1.86 m at 27° | GM 5.21 m (fluid), max GZ 1.93 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[24].message` | GM 5.13 m (fluid), max GZ 1.86 m at 27° | GM 5.22 m (fluid), max GZ 1.93 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[25].message` | GM 10.09 m (fluid), max GZ 3.50 m at 29° | GM 10.49 m (fluid), max GZ 3.53 m at 30° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[26].message` | GM 10.14 m (fluid), max GZ 3.50 m at 29° | GM 10.55 m (fluid), max GZ 3.53 m at 30° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[27].message` | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.30, B/T 4.90; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 7.30 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.154, Cp 0.959, L/B 7.29, B/T 5.00; outside its range: Fn 0.15 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 7.29 (range 7.59 to 16.29) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[3].message` | freeboard 2.21 m >= required 0.3 m | freeboard 2.27 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[5].message` | 2 x 2.09 m thrusters need 5.99 m across a 17.20 m beam | 2 x 2.03 m thrusters need 5.85 m across a 17.20 m beam | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 5.99 m of 17.20 m beam | 2 of 2 requested thrusters fit; 2 would need 5.85 m of 17.20 m beam | changed |
| sweep default depth gas 90x14 | `checks[22].message` | freeboard 5.21 m at the carried draught 2.49 m >= required 0.3 m | freeboard 5.22 m at the carried draught 2.48 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[23].message` | GM 3.54 m (fluid), max GZ 1.90 m at 39° | GM 3.55 m (fluid), max GZ 1.90 m at 39° | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 3.54 m (fluid), max GZ 1.89 m at 38° | GM 3.56 m (fluid), max GZ 1.89 m at 38° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 8.56 m (fluid), max GZ 2.41 m at 25° | GM 8.61 m (fluid), max GZ 2.41 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 6.09 m (fluid), max GZ 1.87 m at 25° | GM 6.12 m (fluid), max GZ 1.87 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[27].message` | barge form factor at the service speed: Fn 0.186, Cp 0.962, L/B 5.58, B/T 5.58; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 5.58 (range 7.59 to 16.29) | barge form factor at the service speed: Fn 0.186, Cp 0.963, L/B 5.58, B/T 5.60; outside its range: Fn 0.19 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), L/B 5.58 (range 7.59 to 16.29) | changed |
| sweep default depth gas 90x14 | `checks[3].message` | freeboard 5.21 m >= required 0.3 m | freeboard 5.22 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[22].message` | freeboard 3.18 m at the carried draught 1.82 m >= required 0.3 m | freeboard 3.22 m at the carried draught 1.78 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[23].message` | GM 3.56 m (fluid), max GZ 1.67 m at 33° | GM 3.64 m (fluid), max GZ 1.68 m at 33° | changed |
| ... | 81591 more changed quantities | | | |
