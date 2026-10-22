# The engine's messages: checks, warnings, stability notes and names (spec 0005, ADR 0015).
# Written by tools/i18n_extract.py from the Rust source. Do not edit: change the English in the
# `text!` call and run the tool again. A number argument arrives as text, already rounded.


## crates/ovcd-core/src/aft_arrangement.rs

check-name-thrusters-fit-beam = thrusters fit beam
check-thrusters-fit-beam = { $count } x { $envelope } m thrusters need { $needed } m across a { $beam } m beam
check-name-requested-thruster-count-fits = requested thruster count fits
check-requested-thruster-count-fits = { $fit } of { $requested } requested thrusters fit; { $requested } would need { $needed } m of { $beam } m beam
check-name-containers-fit-beam = containers fit beam
check-containers-fit-beam = { $count } x 40 ft units abreast need { $needed } m across a { $beam } m beam
check-name-units-fit-in-one-row = units fit in one row
check-units-fit-in-one-row = { $units } units in { $rows } row(s) of { $per_row }; a second row adds { $added } m to the stern block and comes straight off the cargo zone
check-name-wheelhouse-fits-beam = wheelhouse fits beam
check-wheelhouse-fits-beam = wheelhouse { $width } m wide on a { $beam } m beam
check-accom-beside-wheelhouse = { $units } x 40 ft units beside a { $wheelhouse } m wheelhouse need { $needed } m of { $beam } m beam
check-accom-beside-wheelhouse-rows = { $units } x 40 ft units beside a { $wheelhouse } m wheelhouse need { $needed } m of { $beam } m beam; they go in a row aft of the cab instead, which lengthens the block
check-name-accom-beside-wheelhouse = accom beside wheelhouse
check-name-gensets-fit-beam = gensets fit beam
check-gensets-fit-beam = { $count } x { $width } m gensets abreast need { $needed } m across a { $beam } m beam
check-name-gensets-fit-under-deck = gensets fit under deck
check-gensets-fit-under-deck = genset { $height } m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth { $depth } m
check-name-gensets-carry-load-with-one-in-reserve = gensets carry load with one in reserve
check-gensets-carry-load-with-one-in-reserve = { $running } of { $count } x { $set_kwe } kWe sets carry { $load } kWe
warn-aft-gas-stern-estimates = Gas-fuelled stern block: the genset envelopes beyond the published engine (alternator, base frame) and the engine-room and GVU-room clearances are ESTIMATES (no vendor GA). The exhaust uptake and the fuel vent mast are not in the air draft (assumed lowerable). Fuel is taken from the cargo boil-off (IGC Code chapter 16); no separate fuel tank is in the block.
check-name-wheelhouse-sees-over-cargo = wheelhouse sees over cargo
check-wheelhouse-sees-over-cargo = raised eye { $eye } m >= obstruction { $obstruction } m + { $margin } m margin (lift used { $lift } m of { $max_lift } m)
warn-aft-sightline-lift = Sightline needs { $lift } m of lift; the column gives { $max_lift } m. Raise the column or lower the cargo top.

## crates/ovcd-core/src/barge_capacity.rs

warn-barge-bulk-gm = Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet.
check-wheelhouse-blind-distance = blind distance ahead of the bow { $blind } m <= { $limit } m, light (ES-TRIN 7.02(2)); lift used { $lift } m of { $max_lift } m
warn-aft-blind-distance-lift = The blind distance needs { $needed } m of lift; the column gives { $max_lift } m. Raise the column or lower the cargo top.
check-feasible-geometry = cargo volume must be positive (tank/hold fits)
check-name-feasible-geometry = feasible geometry
check-name-not-overloaded = not overloaded
check-not-overloaded = volume-bound load must float within the moulded depth
check-name-freeboard-ok = freeboard ok
check-freeboard-ok = freeboard { $freeboard } m >= required { $required } m
check-name-cargo-zone-positive = cargo zone positive
check-cargo-zone-positive = cargo zone { $zone } m = LOA { $loa } - bow void { $bow_void } m - stern void { $stern_void } m
check-air-draft-condition-ballasted-light = ballasted light
check-air-draft-condition-light = light
check-name-air-draft-within-limit = air draft within limit
check-air-draft-within-limit = { $condition } air draft { $governing } m <= limit { $limit } m (laden is { $laden } m and is NOT the governing case)
check-trim-in-range = the hull floats within { $limit } degrees of trim (every condition)
check-trim-no-balance = no trim balances the load: { $conditions }
check-name-trim-in-range = trim in range
warn-barge-pushed = PUSHED barge: crew, battery and propulsion are on the tug and are NOT in this hull's lightship, capacity or air draft. The tug is a separate vessel and must be costed and air-draft-gated separately.
check-name-gm-positive = gm positive
check-gm-positive = hopper intact GM { $gm } m >= 0.15 m (first-order; KG_light @0.5D)

## crates/ovcd-core/src/barge_cost.rs

check-name-masses-positive = masses positive
check-masses-positive = mass-model inputs present
check-name-no-reliquefaction-in-scope = no reliquefaction in scope
check-no-reliquefaction-in-scope = cargo system excludes reliquefaction and nitrogen plant per the pressure-accumulation operating concept
check-name-fabrication-rates-are-estimates = fabrication rates are estimates
check-fabrication-rates-are-estimates = fabrication/supply rates are user-supplied estimates; replace them with yard quotes before relying on the total

## crates/ovcd-core/src/electric_energy.rs

check-service-below-schijf = service { $service } kn stays below the Schijf limit { $limit } kn
check-service-below-schijf-open-water = no confined limit (open water)
check-name-service-below-schijf = service below schijf
check-name-energy-positive = energy positive
check-energy-positive = energy { $energy } kWh/km at service speed
check-name-resistance-coeffs-first-order = resistance coeffs first order
check-resistance-coeffs-first-order = confined-water resistance (residuary/Lackenby/blockage) is a first-order estimate (RS1) — ITTC-1957 friction is standard; anchor with a model test/CFD before class use.

## crates/ovcd-core/src/gas_arrangement.rs

check-name-tank-length-over-diameter = tank length over diameter
check-tank-length-over-inner-diameter = tank length { $length } m / inner diameter { $diameter } m = { $ratio } <= { $max }
check-name-freeboard-at-payload = freeboard at payload
check-freeboard-at-payload = freeboard { $freeboard } m at the carried draught { $draught } m >= required { $required } m
check-stability-light-angle = GM { $gm } m (fluid), max GZ { $max_gz } m at { $angle }° — warning, not a failure: angle of max GZ { $value } < { $limit } (IS Code 2.2.3) with no ballast; every other criterion holds. Sail light with ballast
check-stability = GM { $gm } m (fluid), max GZ { $max_gz } m at { $angle }°
check-stability-fails = GM { $gm } m (fluid), max GZ { $max_gz } m at { $angle }° — fails: { $fails }
check-stability-light-angle-ballast = GM { $gm } m (fluid), max GZ { $max_gz } m at { $angle }° — warning, not a failure: angle of max GZ { $angle } < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take { $ballast } t of ballast (the Ballast, departure condition meets every criterion)
check-name-ballast-plan = Ballast plan
check-no-ballast-plan = { $condition }: no ballast plan meets every criterion, even with every ballast tank full.
check-name-air-draft = Air draft
check-condition-air-draft = { $condition }: air draft { $air } m is above the limit.
check-name-stability-not-assessed = Stability — not assessed
check-stability-not-assessed = stability could not be assessed: { $reason }

## crates/ovcd-core/src/gas_separation.rs

check-name-aft-end-bulkhead-space = aft end bulkhead space
check-aft-end-bulkhead-space = aft end-bulkhead space { $aft } m >= { $min } m ({ $tank_ends }, cl. 3.5.3)
check-fwd-end-bulkhead-space-plane-ends = plane tank ends
check-fwd-end-bulkhead-space-domed-ends = domed tank ends
check-name-opening-setback = opening setback
check-opening-distance = distance { $distance } m from the cargo area (end space included) to any entrance or openable window >= { $min } m (cl. 3.3.7 / 3.3.11)
check-opening-setback = aft separation { $aft } m >= { $min } m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11)
check-name-fwd-end-bulkhead-space = fwd end bulkhead space
check-fwd-end-bulkhead-space = forward end-bulkhead space { $fwd } m >= { $min } m ({ $tank_ends }, cl. 3.5.3)
warn-separation-clauses = Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release.

## crates/ovcd-core/src/gas_stability.rs

stab-limit-concept-stage = Concept-stage: the hull's sections, first-order weight heights, no downflooding openings.
stab-limit-no-wind-or-damage = No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3).
stab-limit-stern-block-mass = The self-propelled stern block's mass is not in the gas lightship, so KG and displacement leave it out; the true GM could be lower.
stab-limit-weight-split = Weight split sums to { $split } t vs lightship { $lightship } t.

## crates/ovcd-core/src/journey_derating.rs

check-name-capacity-is-volume-bound = capacity is volume bound
check-capacity-is-volume-bound = input capacity must be a feasible volume-bound design (not overloaded)
check-name-floats = floats
check-floats = channel must float some cargo (T_max { $t_max } m vs lightship draft)
check-name-ukc-consistent = ukc consistent
check-ukc-consistent = design depth must be >= UKC (pass depth + UKC, never a pre-reduced T_max)
check-name-utilisation-reasonable = utilisation reasonable
check-utilisation-reasonable = volume utilisation { $utilisation } >= floor { $floor }
warn-journey-draft-bound = Draft-bound: shed { $shed } t of { $capacity } t volume capacity to float at T_max = { $t_max } m (full-load draft { $full_load_draft } m).

## crates/ovcd-core/src/liquid_bulk/cost.rs

check-name-quantities-present = quantities present
check-tank-quantities-present = cargo tank quantities present
check-name-tank-items-are-lump-sums = tank items are lump sums
check-tank-items-are-lump-sums = the cargo tank items are lump sums from the tank layout, not a bill of quantities

## crates/ovcd-core/src/machinery.rs

warn-machinery-battery-not-sized = The battery energy of the hybrid is not sized yet, so the battery room is not in the stern block.
check-name-machinery-fits-beam = machinery fits beam
check-machinery-fits-beam = { $count } x { $width } m engines abreast need { $needed } m across a { $beam } m beam
check-name-machinery-fits-under-deck = machinery fits under deck
check-machinery-fits-under-deck = engine { $height } m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth { $depth } m
warn-machinery-power-not-sized = The installed power is not sized yet, so the engines and their room are not in the stern block; type an installed power to place them.
warn-machinery-no-engine = No engine of the table covers { $kw } kW per engine (the largest is 4,400 kW), so the engine room is not in the stern block.

## crates/ovcd-core/src/offsets.rs

msg-offsets-not-finite = row { $row } of the offsets table holds a value that is not a number
msg-offsets-too-few-stations = the offsets table needs at least 2 stations, not { $n }
msg-offsets-x-descending = station { $station } of the offsets table lies aft of the station before it; give the stations in ascending x
msg-offsets-too-few-points = station { $station } of the offsets table needs at least 2 points
msg-offsets-off-centre-line = station { $station } of the offsets table does not start and end on the centre line (y = 0)
msg-offsets-negative-y = station { $station } of the offsets table has a point with y below 0
msg-offsets-self-crossing = station { $station } of the offsets table crosses itself

## crates/ovcd-core/src/resistance.rs

check-name-lackenby-fit-subcritical = lackenby fit subcritical
check-lackenby-fit-subcritical = depth Froude { $froude } at the service speed; the shallow-water fit holds below { $limit }, above it a fixed factor stands in
check-service-speed-below-limiting = { $environment }: service { $service } kn <= 0.95 x Schijf limit { $limit } kn
check-power-within-installed = { $environment }: power { $power } kW <= installed { $installed } kW
check-blockage-in-range = { $environment }: blockage n = { $blockage } must be in (0, 1)
check-service-speed-fails-squat = { $environment }: at the service speed { $service } kn the squat leaves less than the under-keel clearance
check-name-coefficients-are-first-order = coefficients are first order
check-coefficients-are-first-order = Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard.

## crates/ovcd-core/src/squat.rs

check-name-keel-clears-bed = keel clears bed
check-keel-clears-bed = dynamic UKC { $ukc } m = static { $static_ukc } m - squat { $squat } m ({ $method }) at { $speed } kn
check-name-keel-clears-bed-with-margin = keel clears bed with margin
check-keel-clears-bed-with-margin = dynamic UKC { $ukc } m against a { $margin } m working margin
check-name-froude-depth-subcritical = froude depth subcritical
check-froude-depth-subcritical = depth Froude { $froude }; above 0.7 the flow approaches critical and both squat forms lose validity
check-name-blockage-within-barrass-range = blockage within barrass range
check-blockage-within-barrass-range = blockage { $blockage }; Barrass is fitted below about 0.35 and extrapolates above it
check-name-methods-agree-within-50pct = methods agree within 50pct
check-methods-agree-within-50pct = Barrass { $barrass } m vs ICORELS { $icorels } m, { $spread }% apart
warn-squat-icorels-not-evaluated = ICORELS not evaluated: depth Froude { $froude } at or above 0.99
warn-squat-open-water = no channel area given, so blockage is 0 and Barrass returns 0 — this is the OPEN-WATER case and squat is then ICORELS only

## crates/ovcd-core/src/vessel_standards.rs

check-name-unique-ids = unique ids
check-unique-ids = every key matches its standard_id
check-unique-ids-mismatch = key/standard_id mismatch: { $keys }
check-name-ranges-ordered = ranges ordered
check-ranges-ordered = every (min, max) range is ordered
check-ranges-ordered-inverted = inverted range: { $keys }
check-name-positive-dimensions = positive dimensions
check-positive-dimensions = all beams and lengths positive
check-positive-dimensions-bad = non-positive dimension: { $keys }
check-name-convoy-beam-consistent = convoy beam consistent
check-convoy-beam-consistent = convoy beam = unit beam x n_abreast (an identity for the built-in catalogue, which has no independent unit beam)
check-convoy-beam-consistent-bad = beam does not reconcile with n_abreast: { $keys }
check-name-source-cited = source cited
check-source-cited = every class cites its table
check-source-cited-missing = no source: { $keys }

## crates/ovcd-core/src/waves.rs

check-name-stawave1-short-wave-validity = stawave1 short wave validity
check-stawave1-short-wave-validity = lambda/Lpp = { $ratio } (lambda { $wavelength } m on Lpp { $lpp } m). STAwave-1 assumes short waves, lambda/Lpp < 0.5; outside that it UNDER-predicts, and near lambda/Lpp = 1 it can do so by more than 2x
check-name-wave-height-within-screening-range = wave height within screening range
check-wave-height-within-screening-range = H_s { $hs } m; above about 3 m a screening formula is not the right tool and voluntary speed reduction usually governs anyway
check-name-bow-length-supplied = bow length supplied
check-bow-length-supplied = L_BWL not given, assumed { $bow_length } m = 0.2 x Lpp (a blunt barge bow). A finer bow reduces added resistance, so this is the conservative assumption
warn-waves-period-estimated = no wave period given; T_p estimated at { $tp } s from H_s, which sets only the validity check, not the resistance
warn-waves-beam-seas-floor = beam-ish seas: this module returns a FLOOR, because drift, rudder and course-keeping loss is not modelled and it dominates off the bow

## crates/ovcd-format/src/class_input.rs

msg-class-scheme = `{ $input }` { $class } belongs to the scheme { $family }, not { $scheme }
msg-class-none = the scheme { $scheme } has no class for a { $kind } Formation, so no class applies
msg-class-moved = `{ $input }` { $typed } does not hold this length and beam; the vessel sits in { $class }
msg-class-beyond = this length and beam are beyond the largest { $scheme } class ({ $class })
msg-class-draught-above = `{ $input }` = { $value } is above the draught of class { $class } ({ $limit } m, the top of its range)
msg-class-clearance-above = `{ $input }` = { $value } is above the lowest bridge clearance of class { $class } ({ $limit } m)

## crates/ovcd-format/src/convoy.rs

check-name-within-lock-length = within lock length
check-lock-length = overall length { $length } m <= lock length { $limit } m
check-name-within-lock-width = within lock width
check-lock-width = overall beam { $width } m <= lock width { $limit } m

## crates/ovcd-format/src/formation.rs

warn-formation-sizing-not-converged = The pusher sizing did not converge in { $passes } passes (the power moved by more than { $tol } %). The power shown is that of the last pass.
note-formation-method-unverified = The Formation's resistance is that of its Equivalent Hull (the barges and the pusher as one hull) by the single-hull method, times the Formation resistance allowance. The method is unverified.
note-formation-coupling-force = Coupling force between the pusher and the pushed craft: { $force } kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B { $power } kW, L_S { $ls } m taken as the pusher length, B_S { $bs } m).

## crates/ovcd-format/src/offsets.rs

msg-offsets-no-table = an offsets Hull Form needs its offsets table (`{ $input }`)
msg-offsets-bad-row = row { $row } of the offsets table needs the numbers station_x_m, y_m and z_m
msg-offsets-csv-header = an offsets CSV starts with the header { $header }
msg-offsets-csv-line = line { $line } of the offsets CSV needs three numbers: station_x_m, y_m, z_m

## crates/ovcd-format/src/run.rs

msg-convoy-not-available = convoy results not yet available

## crates/ovcd-format/src/session.rs

msg-offsets-resampled = the offsets table has stations of unequal point counts: { $count } station(s) are resampled by arc length to { $points } points, with their own points kept

## crates/ovcd-format/src/shipd.rs

msg-shipd-no-parameters = a ShipD Hull Form needs its parameters (`{ $input }`)
msg-shipd-missing = `{ $input }` needs the ShipD parameter { $name }
msg-shipd-constraint = `{ $input }` fails ShipD constraint { $index } ({ $name })

## crates/ovcd-format/src/simulate.rs

warn-channel-no-bottom-width = The channel has a depth but no bottom width, so squat, the Schijf limiting speed and the blockage are not checked and the speed curves are for deep water only. The draught limit still uses the depth. Give the channel bottom width to check them.
check-name-intact-gm-at-reported-cargo = intact gm at reported cargo
check-intact-gm-reported-cargo = intact GM { $gm } m >= 0.15 m at the reported cargo { $cargo } t, draught { $draught } m (first-order; cargo KG of the full hold; KG_light @0.5D)
warn-liquid-legacy-conditions = Stability is computed for liquid bulk cargo with the IS Code loading conditions only.
name-governs-draught = draught — { $limit }
check-feasible-arrangement = No arrangement passes every check on this hull. The closest one is shown. It fails: { $fails }.
stab-limit-moulded-tanks = Ballast and consumables tanks are moulded volumes with no structure deduction; one tank per side and kind, and the consumables tank height is an estimate.

## crates/ovcd-format/src/text_ids.rs

name-condition-laden = Laden
name-condition-half-laden = Half laden
name-condition-light = Light
name-criterion-area-0-30 = Area 0–30°
name-criterion-area-0-40 = Area 0–40°
name-criterion-area-30-40 = Area 30–40°
name-criterion-gz-at-30 = GZ at ≥ 30°
name-criterion-angle-of-max-gz = Angle of max GZ
name-criterion-initial-gm = Initial GM (fluid)
name-criterion-floats = Floats with freeboard
name-check-stability-laden = Stability — laden
name-check-stability-half-laden = Stability — half laden
name-check-stability-light = Stability — light
name-check-stability-not-assessed = Stability — not assessed
name-check-cargo = Cargo > 0
name-check-feasible-arrangement = Feasible arrangement
name-check-tank-length = tank length over diameter
name-check-ok = OK
name-check-lock-length = within lock length
name-check-lock-width = within lock width
name-curve-deep-water = Deep water
name-governs-hold-volume = hold volume
name-governs-tank-volume = tank volume
name-governs-no-cargo-area = no cargo area (hull shorter than its fixed end spaces)
name-vessel-box = Box-hold dry bulk barge
name-vessel-hopper = Sloped-hopper dry bulk barge
name-vessel-gas = Gas barge, IMO Type C tanks
name-cargo-lightship-alone = Lightship alone exceeds the allowed draught.
name-cargo-nothing-fits = Nothing fits: lengthen or widen the hull.

## crates/ovcd-format/src/vdesign.rs

msg-file-not-json = the file is not JSON: { $error }
msg-file-the-file = the file
msg-file-duplicate-key = `{ $key }` appears twice in { $place }; a key may appear once
msg-file-not-object = the file must hold a JSON object
msg-file-format = the file needs `"format": "vdesign"`
msg-file-version-unsupported = format_version { $version } is not supported; this tool reads versions { $oldest } to { $newest }
msg-file-version-missing = the file needs an integer `format_version`
msg-file-engine-version = the file needs a non-empty string `engine_version`
msg-file-units = `units` must be "SI", not { $units }
msg-file-unknown-key = unknown top-level key `{ $key }` is ignored
msg-file-no-options = the file needs an `options` list
msg-file-empty = a Design needs at least one Option
msg-option-no-name = Option { $number } needs a non-empty string `name`
msg-option-names-differ = two Options are named '{ $option }'; names must differ
msg-option-no-inputs = needs an `inputs` object
msg-option-unknown-key = unknown key `{ $key }` is ignored
msg-choice = `{ $input }` = { $value } must be one of: { $choices }
msg-results-hash-shape = `results_hash` must be 64 hex digits
msg-results-hash-text = `results_hash` must be text
msg-ruleset-exact-renamed = IGC_2G_EXACT is now IGC_2G: the same per-tank rule of IGC 2.4.1.1
msg-ruleset-per-tank = IGC_2G now uses the per-tank rule of IGC 2.4.1.1 (d(Vc) from the gross volume of each tank, dome included, measured plate to plate per 2.4.2), so results may change
msg-input-note = `{ $input }`: { $note }
msg-block-coefficient-dropped = `{ $input }`: the block coefficient now comes from the hull shape; the stored value was dropped
msg-midship-coefficient-dropped = `{ $input }`: the midship coefficient now comes from the hull shape; the stored value was dropped
msg-v1-adn-g = `{ $input }` ADN_G is no longer offered (the engine applies IGC rules only to gas tanks); this version 1 file was opened as IGC_2G
msg-v1-tank-cap = `{ $input }` is no longer an input (there is no tank volume cap); it was dropped
msg-v1-max-tank-od = this version 1 file has no `{ $input }`; it now uses the default { $value } m (version 1 used a fixed 12 m cap, so results may change)
msg-marker-source = `source` must be "typed", "default", "class" or "reference"
msg-marker-value = a marked input needs a `value`
msg-marker-only = a marked input holds only `value` and `source`
msg-unknown-input = unknown input `{ $key }`
msg-input-unused-cargo = `{ $input }` is not used for the Cargo { $cargo }; ignored
msg-input-unused-when = `{ $input }` is not used when `{ $by }` is { $value }; ignored
msg-input-unused-containment = `{ $input }` is not used for a { $containment } Cargo Containment; ignored
msg-input-unused-formation = `{ $input }` is not used for a { $formation } Formation; ignored
msg-input-required = missing required input `{ $input }`
msg-input-filled = `{ $input }` was not stored in the file; wrote in the Defaults Profile value { $value }, marked default
msg-legacy-dome = this file has no `{ $input }`; it now holds { $value } m (marked default). The dome now counts in Vc for d(Vc) (IGC 2.4.1.1), so the results may change
msg-legacy-carriage = this file has no `{ $input }` for the LPG Cargo { $cargo }; it is read as { $condition } (marked default). The tank values the file lacks keep what the engine used before (design pressure, material, insulation, structure factor), so the results are unchanged; set the carriage condition and reset those inputs to take the defaults of that condition
msg-legacy-fixed-one = this file has no `{ $input }`; it now holds the fixed value the engine used before it was an input (marked default), so the results are unchanged
msg-legacy-fixed-many = this file has no { $inputs }; each now holds the fixed value the engine used before it was an input (marked default), so the results are unchanged
msg-inputs-not-stored = inputs not stored in the file: { $inputs }. A saved Design holds every input as used, so a Defaults Profile edit cannot change it; run `ovcd fill` to write them in from the Defaults Profile, marked default
msg-profile-case-keyed = `default_by_{ $by }` for { $key }
msg-profile-case-default = a `default`
msg-profile-no-default = the Defaults Profile '{ $profile }' gives no default for `{ $input }` ({ $case } is missing)
msg-profile-no-table-default-carriage = the Defaults Profile '{ $profile }' gives no default for `{ $input }` when the Cargo Containment is { $containment }, the Cargo { $cargo } and the carriage condition { $condition } (no row of `{ $key }.default_by_table` fits)
msg-profile-no-table-default = the Defaults Profile '{ $profile }' gives no default for `{ $input }` when the Cargo Containment is { $containment }, the Cargo { $cargo } (no row of `{ $key }.default_by_table` fits)
msg-profile-no-depth-ratio = the Defaults Profile '{ $profile }' gives no depth-to-beam ratio for { $group } Cargo (`depth_to_beam_ratio.default_by_cargo_group.{ $group }` is missing)
msg-currency = `{ $input }` must be a three-letter currency code in capitals, such as USD
msg-price-base-date = `{ $input }` must be a date as YYYY, YYYY-MM or YYYY-MM-DD, or null (none)
msg-string = `{ $input }` must be a string
msg-number-or-null = `{ $input }` must be a number or null (none)
msg-bool = `{ $input }` must be true or false
msg-hull-form-not-built = `{ $input }` = { $value } is not built yet: the engine computes raked_box, v_bottom, chine, offsets and shipd for now (spec 0008)
msg-parameter-set = `{ $input }` must be an object of named numbers, or null
msg-offsets-table = `{ $input }` must be a list of rows with `station_x_m`, `y_m` and `z_m` in metres, or null
msg-scheme = `{ $input }` = { $value } is not a scheme with a Vessel Class that can define a vessel (see `ovcd schema`)
msg-class-cannot-define = `{ $input }` = { $value } cannot define a vessel: class { $class } gives no length or beam, or its Formation ({ $formation }) is not one a Design has
msg-class-unknown = `{ $input }` = { $value } is not a Vessel Class (see `ovcd schema`)
msg-class-id = `{ $input }` must be a class id or null
msg-rule-sets = `{ $input }` must be a list of Rule Sets, each with `id`, `version`, `hash` and `source` (`default` or `typed`), no set twice, and the `ovcd` set always in it
msg-formation-object = `{ $input }` must be an object with a `kind`
msg-formation-kind = `{ $input }` must be one of: { $choices }
msg-formation-only-kind = `{ $input }`: a { $kind } holds only `kind`; `abreast` and `in_line` are for a pushed_convoy
msg-formation-count = `{ $input }` must be a whole number for a pushed_convoy
msg-formation-convoy-only = `{ $input }`: a pushed_convoy holds only `kind`, `abreast` and `in_line`
msg-hard-range = `{ $input }` = { $value } is out of range: must be { $bound } (hard limit)
msg-soft-range = `{ $input }` = { $value } is outside the usual range { $min } to { $max } (soft limit); accepted
msg-offsets-dimension = `{ $input }` = { $typed } differs from the offsets table's { $table }: an offsets hull takes its length, beam and depth from its table
msg-slender-hull = `{ $input }` = { $length } is { $ratio } times `{ $depth_input }` = { $depth }, over about { $limit }: a very slender hull for its depth (soft limit); accepted
msg-containment-cargo = `{ $input }` = { $containment } cannot carry { $cargo } ({ $phase }); use { $fits }
msg-freeboard-depth = `{ $input }` = { $freeboard } must be less than `{ $depth_input }` = { $depth } (hard limit)
msg-whole-number = `{ $input }` = { $value } must be a whole number (hard limit)
msg-insulation-disagree = `{ $input }` is { $kind } but `{ $thickness_input }` is { $thickness }: the type and the thickness disagree (soft check); accepted
msg-loading-above-marvs = `{ $input }` = { $value }: the vapour pressure of { $cargo } there is { $pressure } bar, above the MARVS of { $marvs } bar absolute; the tank is loaded to the filling limit, not above (soft check); accepted
msg-stress-from-input = `{ $input }`
msg-stress-from-basis = the `published_design` allowable stress basis
msg-stress-from-material = the material
msg-stress-above-igc = { $from } gives an allowable stress of { $stress } MPa, above the IGC 4.23.3.1 limit for { $material } of { $limit } MPa, the lesser of Rm/A = { $rm } / { $a } and Re/B = { $re } / { $b } (soft check); accepted
msg-freeboard-search = `{ $input }` = { $freeboard } must be at most { $limit }, the deepest hull `{ $search_input }` searches (hard limit)
msg-channel-ukc = `{ $input }` = { $depth } must be at least `{ $ukc_input }` = { $ukc } (hard limit)
msg-hold-width = `{ $input }` = { $beam } leaves a hold width of { $width } m after the two { $void } m wing voids; { $reason }
msg-negative-hold = a negative hold gives a negative cargo mass and draught (hard limit)
msg-hold-height = `{ $input }` = { $depth } leaves a hold height of { $height } m (depth less the { $double_bottom } m double bottom, plus the { $coaming } m coaming); { $reason }
msg-number = `{ $input }` must be a number
msg-input-error = `{ $input }`: { $error }
msg-option = Option '{ $option }': { $message }
