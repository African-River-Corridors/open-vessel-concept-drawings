# The engine's messages: checks, warnings, stability notes and names (spec 0005, ADR 0014).
# Written by tools/i18n_extract.py from the Rust source. Do not edit: change the English in the
# `text!` call and run the tool again. A number argument arrives as text, already rounded.


## crates/ovcd-core/src/aft_arrangement.rs

check-thrusters-fit-beam = { $count } x { $envelope } m thrusters need { $needed } m across a { $beam } m beam
check-requested-thruster-count-fits = { $fit } of { $requested } requested thrusters fit; { $requested } would need { $needed } m of { $beam } m beam
check-containers-fit-beam = { $count } x 40 ft units abreast need { $needed } m across a { $beam } m beam
check-units-fit-in-one-row = { $units } units in { $rows } row(s) of { $per_row }; a second row adds { $added } m to the stern block and comes straight off the cargo zone
check-wheelhouse-fits-beam = wheelhouse { $width } m wide on a { $beam } m beam
check-accom-beside-wheelhouse = { $units } x 40 ft units beside a { $wheelhouse } m wheelhouse need { $needed } m of { $beam } m beam
check-accom-beside-wheelhouse-rows = { $units } x 40 ft units beside a { $wheelhouse } m wheelhouse need { $needed } m of { $beam } m beam; they go in a row aft of the cab instead, which lengthens the block
check-gensets-fit-beam = { $count } x { $width } m gensets abreast need { $needed } m across a { $beam } m beam
check-gensets-fit-under-deck = genset { $height } m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth { $depth } m
check-gensets-carry-load-with-one-in-reserve = { $running } of { $count } x { $set_kwe } kWe sets carry { $load } kWe
warn-aft-gas-stern-estimates = Gas-fuelled stern block: the genset envelopes beyond the published engine (alternator, base frame) and the engine-room and GVU-room clearances are ESTIMATES (no vendor GA). The exhaust uptake and the fuel vent mast are not in the air draft (assumed lowerable). Fuel is taken from the cargo boil-off (IGC Code chapter 16); no separate fuel tank is in the block.
check-wheelhouse-sees-over-cargo = raised eye { $eye } m >= obstruction { $obstruction } m + { $margin } m margin (lift used { $lift } m of { $max_lift } m)
warn-aft-sightline-lift = Sightline needs { $lift } m of lift; the column gives { $max_lift } m. Raise the column or lower the cargo top.

## crates/ovcd-core/src/barge_capacity.rs

check-feasible-geometry = cargo volume must be positive (tank/hold fits)
check-not-overloaded = volume-bound load must float within the moulded depth
check-freeboard-ok = freeboard { $freeboard } m >= required { $required } m
check-cargo-zone-positive = cargo zone { $zone } m = LOA { $loa } - bow void { $bow_void } m - stern void { $stern_void } m
check-air-draft-condition-ballasted-light = ballasted light
check-air-draft-condition-light = light
check-air-draft-within-limit = { $condition } air draft { $governing } m <= limit { $limit } m (laden is { $laden } m and is NOT the governing case)
warn-barge-pushed = PUSHED barge: crew, battery and propulsion are on the tug and are NOT in this hull's lightship, capacity or air draft. The tug is a separate vessel and must be costed and air-draft-gated separately.
check-gm-positive = hopper intact GM { $gm } m >= 0.15 m (first-order; KG_light @0.5D)
warn-barge-bulk-gm = Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet.

## crates/ovcd-core/src/barge_cost.rs

check-masses-positive = mass-model inputs present
check-no-reliquefaction-in-scope = cargo system excludes reliquefaction and nitrogen plant per the pressure-accumulation operating concept
check-fabrication-rates-are-estimates = fabrication/supply rates are user-supplied estimates; replace them with yard quotes before relying on the total

## crates/ovcd-core/src/electric_energy.rs

check-service-below-schijf = service { $service } kn stays below the Schijf limit { $limit } kn
check-service-below-schijf-open-water = no confined limit (open water)
check-energy-positive = energy { $energy } kWh/km at service speed
check-resistance-coeffs-first-order = confined-water resistance (residuary/Lackenby/blockage) is a first-order estimate (RS1) — ITTC-1957 friction is standard; anchor with a model test/CFD before class use.

## crates/ovcd-core/src/gas_arrangement.rs

check-tank-length-over-diameter = tank length { $length } m / outer diameter { $diameter } m = { $ratio } <= { $max }
check-freeboard-at-payload = freeboard { $freeboard } m at the carried draught { $draught } m >= required { $required } m
check-stability = GM { $gm } m (fluid), max GZ { $max_gz } m at { $angle }°
check-stability-fails = GM { $gm } m (fluid), max GZ { $max_gz } m at { $angle }° — fails: { $fails }
check-stability-not-assessed = stability could not be assessed: { $reason }

## crates/ovcd-core/src/gas_separation.rs

check-cofferdam-min-width = cofferdam { $cofferdam } m >= { $min } m (cl. 3.6.13)
check-opening-setback = aft separation { $aft } m >= { $min } m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11)
check-fwd-end-bulkhead-space = forward end-bulkhead space { $fwd } m >= { $min } m ({ $tank_kind }, cl. 3.5.3 / 3.6.3)
check-fwd-end-bulkhead-space-pressure-tank = pressure tank
check-fwd-end-bulkhead-space-standard = standard
warn-separation-clauses = Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release.

## crates/ovcd-core/src/gas_stability.rs

stab-limit-concept-stage = Concept-stage: prismatic box section, first-order weight heights, no downflooding openings.
stab-limit-no-wind-or-damage = No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval).
stab-limit-stern-block-mass = The self-propelled stern block's mass is not in the gas lightship, so KG and displacement leave it out; the true GM could be lower.
stab-limit-weight-split = Weight split sums to { $split } t vs lightship { $lightship } t.

## crates/ovcd-core/src/journey_derating.rs

check-capacity-is-volume-bound = input capacity must be a feasible volume-bound design (not overloaded)
check-floats = channel must float some cargo (T_max { $t_max } m vs lightship draft)
check-ukc-consistent = design depth must be >= UKC (pass depth + UKC, never a pre-reduced T_max)
check-utilisation-reasonable = volume utilisation { $utilisation } >= floor { $floor }
warn-journey-draft-bound = Draft-bound: shed { $shed } t of { $capacity } t volume capacity to float at T_max = { $t_max } m (full-load draft { $full_load_draft } m).

## crates/ovcd-core/src/resistance.rs

check-service-speed-below-limiting = { $environment }: service { $service } kn <= 0.95 x Schijf limit { $limit } kn
check-power-within-installed = { $environment }: power { $power } kW <= installed { $installed } kW
check-blockage-in-range = { $environment }: blockage n = { $blockage } must be in (0, 1)
check-coefficients-are-first-order = Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard.

## crates/ovcd-core/src/squat.rs

check-keel-clears-bed = dynamic UKC { $ukc } m = static { $static_ukc } m - squat { $squat } m ({ $method }) at { $speed } kn
check-keel-clears-bed-with-margin = dynamic UKC { $ukc } m against a { $margin } m working margin
check-froude-depth-subcritical = depth Froude { $froude }; above 0.7 the flow approaches critical and both squat forms lose validity
check-blockage-within-barrass-range = blockage { $blockage }; Barrass is fitted below about 0.35 and extrapolates above it
check-methods-agree-within-50pct = Barrass { $barrass } m vs ICORELS { $icorels } m, { $spread }% apart
warn-squat-icorels-not-evaluated = ICORELS not evaluated: depth Froude { $froude } at or above 0.99
warn-squat-open-water = no channel area given, so blockage is 0 and Barrass returns 0 — this is the OPEN-WATER case and squat is then ICORELS only

## crates/ovcd-core/src/vessel_standards.rs

check-unique-ids = every key matches its standard_id
check-unique-ids-mismatch = key/standard_id mismatch: { $keys }
check-ranges-ordered = every (min, max) range is ordered
check-ranges-ordered-inverted = inverted range: { $keys }
check-positive-dimensions = all beams and lengths positive
check-positive-dimensions-bad = non-positive dimension: { $keys }
check-convoy-beam-consistent = convoy beam = unit beam x n_abreast
check-convoy-beam-consistent-bad = beam does not reconcile with n_abreast: { $keys }
check-source-cited = every class cites its table
check-source-cited-missing = no source: { $keys }

## crates/ovcd-core/src/waves.rs

check-stawave1-short-wave-validity = lambda/Lpp = { $ratio } (lambda { $wavelength } m on Lpp { $lpp } m). STAwave-1 assumes short waves, lambda/Lpp < 0.5; outside that it UNDER-predicts, and near lambda/Lpp = 1 it can do so by more than 2x
check-wave-height-within-screening-range = H_s { $hs } m; above about 3 m a screening formula is not the right tool and voluntary speed reduction usually governs anyway
check-bow-length-supplied = L_BWL not given, assumed { $bow_length } m = 0.2 x Lpp (a blunt barge bow). A finer bow reduces added resistance, so this is the conservative assumption
warn-waves-period-estimated = no wave period given; T_p estimated at { $tp } s from H_s, which sets only the validity check, not the resistance
warn-waves-beam-seas-floor = beam-ish seas: this module returns a FLOOR, because drift, rudder and course-keeping loss is not modelled and it dominates off the bow

## crates/ovcd-format/src/run.rs

msg-convoy-not-available = convoy results not yet available

## crates/ovcd-format/src/simulate.rs

check-air-draft-limit = { $air_draft } m vs limit { $limit } m
name-governs-draught = draught — { $limit }
check-feasible-arrangement = No arrangement passes every check on this hull. The closest one is shown. It fails: { $fails }.

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
name-check-air-draft = Air draft (empty, governing)
name-check-tank-length = tank length over diameter
name-check-ok = OK
name-curve-deep-water = Deep water
name-governs-hold-volume = hold volume
name-governs-tank-volume = tank volume
name-governs-no-cargo-area = no cargo area (hull shorter than its fixed end spaces)
name-vessel-box = Box-hold dry bulk barge
name-vessel-hopper = Sloped-hopper dry bulk barge
name-vessel-gas = Gas barge, IMO Type C tanks
name-cargo-lightship-alone = Lightship alone exceeds the allowed draught.
name-cargo-nothing-fits = Nothing fits: lengthen or widen the hull.
