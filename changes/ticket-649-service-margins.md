<!-- unreleased -->
- Formations (issue #649): the `formation` block of a Formation with its tug has a new `service_margins` object and a note. It shows the still-water power in deep water at the design speed, the sea margin 1.15 ("traditionally about 15 % of the propeller design PD power") and the engine margin 1/0.90 (the service point at 90 % of the specified MCR), both sourced from MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17, and the power with both margins. The margins are shown, not used: the tug is still sized with `tug_installed_power_margin` (1.20), so no power, size or speed moves and the pushed allowance fitted in #359 stands. No shallow-water margin: the shallow-water environments already carry that resistance. For the three reference ATBs the model gives 0.29 to 0.43 of the published installed power, and 0.31 to 0.46 with the margins; the rest is bollard pull and the owner's choice. Results hashes move (new keys only).

<!-- design-impact -->
### ticket/649-service-margins (2026-10-23)

Between `main bd497d9` and `this branch 7ab01434`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `formation.notes[2]` | absent | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 397 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 507 kW, for an installed power of 476 kW. | changed |
| calibration amaggi-box-2000 | `formation.service_margins.engine_margin` | absent | 1.11111111111 | changed |
| calibration amaggi-box-2000 | `formation.service_margins.environment` | absent | Deep water | changed |
| calibration amaggi-box-2000 | `formation.service_margins.power_with_margins_kw` | absent | 506.790640598 | changed |
| calibration amaggi-box-2000 | `formation.service_margins.sea_margin` | absent | 1.15 | changed |
| calibration amaggi-box-2000 | `formation.service_margins.still_water_power_kw` | absent | 396.618762207 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.notes[2]` | absent | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 266 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 340 kW, for an installed power of 319 kW. | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.service_margins.engine_margin` | absent | 1.11111111111 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.service_margins.environment` | absent | Deep water | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.service_margins.power_with_margins_kw` | absent | 340.09577544 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.service_margins.sea_margin` | absent | 1.15 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.service_margins.still_water_power_kw` | absent | 266.161911214 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.notes[2]` | absent | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 349 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 446 kW, for an installed power of 419 kW. | changed |
| calibration canal-box-200x35x12-cby222 | `formation.service_margins.engine_margin` | absent | 1.11111111111 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.service_margins.environment` | absent | Deep water | changed |
| calibration canal-box-200x35x12-cby222 | `formation.service_margins.power_with_margins_kw` | absent | 445.812977657 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.service_margins.sea_margin` | absent | 1.15 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.service_margins.still_water_power_kw` | absent | 348.897112949 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.notes[2]` | absent | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 349 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 446 kW, for an installed power of 419 kW. | changed |
| calibration canal-box-200x35x12-imt201 | `formation.service_margins.engine_margin` | absent | 1.11111111111 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.service_margins.environment` | absent | Deep water | changed |
| calibration canal-box-200x35x12-imt201 | `formation.service_margins.power_with_margins_kw` | absent | 445.812977657 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.service_margins.sea_margin` | absent | 1.15 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.service_margins.still_water_power_kw` | absent | 348.897112949 | changed |
| calibration canal-box-200x35x12-imt205 | `formation.notes[2]` | absent | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 349 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 446 kW, for an installed power of 419 kW. | changed |
| calibration canal-box-200x35x12-imt205 | `formation.service_margins.engine_margin` | absent | 1.11111111111 | changed |
| calibration canal-box-200x35x12-imt205 | `formation.service_margins.environment` | absent | Deep water | changed |
| calibration canal-box-200x35x12-imt205 | `formation.service_margins.power_with_margins_kw` | absent | 445.812977657 | changed |
| calibration canal-box-200x35x12-imt205 | `formation.service_margins.sea_margin` | absent | 1.15 | changed |
| calibration canal-box-200x35x12-imt205 | `formation.service_margins.still_water_power_kw` | absent | 348.897112949 | changed |
| calibration canal-box-200x35x12-pb2012 | `formation.notes[2]` | absent | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 349 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 446 kW, for an installed power of 419 kW. | changed |
| calibration canal-box-200x35x12-pb2012 | `formation.service_margins.engine_margin` | absent | 1.11111111111 | changed |
| calibration canal-box-200x35x12-pb2012 | `formation.service_margins.environment` | absent | Deep water | changed |
| calibration canal-box-200x35x12-pb2012 | `formation.service_margins.power_with_margins_kw` | absent | 445.812977657 | changed |
| calibration canal-box-200x35x12-pb2012 | `formation.service_margins.sea_margin` | absent | 1.15 | changed |
| calibration canal-box-200x35x12-pb2012 | `formation.service_margins.still_water_power_kw` | absent | 348.897112949 | changed |
| calibration canal-box-200x35x12-rci250 | `formation.notes[2]` | absent | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 349 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 446 kW, for an installed power of 419 kW. | changed |
| calibration canal-box-200x35x12-rci250 | `formation.service_margins.engine_margin` | absent | 1.11111111111 | changed |
| calibration canal-box-200x35x12-rci250 | `formation.service_margins.environment` | absent | Deep water | changed |
| calibration canal-box-200x35x12-rci250 | `formation.service_margins.power_with_margins_kw` | absent | 445.812977657 | changed |
| ... | 566 more changed quantities | | | |
