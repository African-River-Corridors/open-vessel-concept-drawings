<!-- unreleased -->
- Stern block (issue #702, option 1): with a specified Propulsor at a transom or pram stern, the stern block lays out `propulsor_count` units at a pitch of the Propulsor's own envelope (1.35 D) plus the thruster clearance. Before, it laid out the default block's three 2.60 m thrusters, so `thruster_offsets_m` and the Propulsor Layout disagreed, and the layout warned. Now they agree, and the layout warns only when the beam takes fewer units. Under a tunnel stern nothing changes. Moved: the text of the two thruster beam checks on 104 cases, and the requested-count check now passes on the two 60 x 10 gas hulls (two units fit where three did not). No cargo, mass, draught or stability value moves. The eight Options of `examples/gas-140x29-coastal-lng/` are re-stamped.

<!-- design-impact -->
### ticket/702-stern-block-propulsor-count (2026-10-30)

Between `main 2e0eafc` and `this branch a7b10c12`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 16.30 m beam | 2 x 1.35 m thrusters need 4.50 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].message` | 3 x 2.60 m thrusters need 10.20 m across a 16.30 m beam | 2 x 1.55 m thrusters need 4.91 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 4.91 m of 16.30 m beam | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 17.20 m beam | 2 x 1.55 m thrusters need 4.91 m across a 17.20 m beam | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 17.20 m beam | 2 of 2 requested thrusters fit; 2 would need 4.91 m of 17.20 m beam | changed |
| sweep default depth gas 90x14 | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.62 m thrusters need 5.04 m across a 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 5.04 m of 14.00 m beam | changed |
| sweep downflooding opening gas 90x11.4 | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 11.40 m beam | 2 x 1.55 m thrusters need 4.91 m across a 11.40 m beam | changed |
| sweep downflooding opening gas 90x11.4 | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 11.40 m beam | 2 of 2 requested thrusters fit; 2 would need 4.91 m of 11.40 m beam | changed |
| sweep gas IGC_2G 60x10 | `checks[5].message` | 2 x 2.60 m thrusters need 7.00 m across a 10.00 m beam | 2 x 1.22 m thrusters need 4.23 m across a 10.00 m beam | changed |
| sweep gas IGC_2G 60x10 | `checks[6].message` | 2 of 3 requested thrusters fit; 3 would need 10.20 m of 10.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.23 m of 10.00 m beam | changed |
| sweep gas IGC_2G 60x10 | `checks[6].ok` | False | True | changed |
| sweep gas IGC_2G 90x14 | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.35 m thrusters need 4.50 m across a 14.00 m beam | changed |
| sweep gas IGC_2G 90x14 | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 14.00 m beam | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[5].message` | 2 x 2.60 m thrusters need 7.00 m across a 10.00 m beam | 2 x 1.22 m thrusters need 4.23 m across a 10.00 m beam | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[6].message` | 2 of 3 requested thrusters fit; 3 would need 10.20 m of 10.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.23 m of 10.00 m beam | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[6].ok` | False | True | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.35 m thrusters need 4.50 m across a 14.00 m beam | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 14.00 m beam | changed |
| sweep gas cargo LNG | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.22 m thrusters need 4.23 m across a 14.00 m beam | changed |
| sweep gas cargo LNG | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.23 m of 14.00 m beam | changed |
| sweep gas cargo LNG_ethane | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.22 m thrusters need 4.23 m across a 14.00 m beam | changed |
| sweep gas cargo LNG_ethane | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.23 m of 14.00 m beam | changed |
| sweep gas cargo LPG_butane | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.28 m thrusters need 4.37 m across a 14.00 m beam | changed |
| sweep gas cargo LPG_butane | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.37 m of 14.00 m beam | changed |
| sweep gas cargo LPG_mix | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.28 m thrusters need 4.37 m across a 14.00 m beam | changed |
| sweep gas cargo LPG_mix | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.37 m of 14.00 m beam | changed |
| sweep gas cargo LPG_propane | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 2 x 1.28 m thrusters need 4.37 m across a 14.00 m beam | changed |
| sweep gas cargo LPG_propane | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 4.37 m of 14.00 m beam | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 11.40 m beam | 2 x 1.42 m thrusters need 4.64 m across a 11.40 m beam | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 11.40 m beam | 2 of 2 requested thrusters fit; 2 would need 4.64 m of 11.40 m beam | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 11.40 m beam | 2 x 1.35 m thrusters need 4.50 m across a 11.40 m beam | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 11.40 m beam | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 11.40 m beam | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 11.40 m beam | 2 x 1.35 m thrusters need 4.50 m across a 11.40 m beam | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 11.40 m beam | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 11.40 m beam | changed |
| sweep gas carriage LPG_mix fully_pressurised | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 11.40 m beam | 2 x 1.42 m thrusters need 4.64 m across a 11.40 m beam | changed |
| sweep gas carriage LPG_mix fully_pressurised | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 11.40 m beam | 2 of 2 requested thrusters fit; 2 would need 4.64 m of 11.40 m beam | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[5].message` | 3 x 2.60 m thrusters need 10.20 m across a 11.40 m beam | 2 x 1.35 m thrusters need 4.50 m across a 11.40 m beam | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[6].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 11.40 m beam | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 11.40 m beam | changed |
| ... | 170 more changed quantities | | | |
