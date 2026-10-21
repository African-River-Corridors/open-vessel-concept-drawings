# The input schema's labels, choices and group names (spec 0005).
# Written by crates/ovcd-format/tests/i18n_inputs.rs from the schema. Do not edit: change the
# label in the Rust source and run `OVCD_WRITE_TEXT=1 cargo test -p ovcd-format --test i18n_inputs`.

## Groups

group-required = Required
group-vessel-class = Vessel Class
group-hull-and-containment = Hull and containment
group-gas-tanks = Gas tanks
group-note-gas-tanks = The tank outer diameter (the cap below and the result) is measured to the outside of the insulation; the clearances and the drawing use that face. The steel diameter is that less twice the insulation thickness. The wall thickness is worked out from the pressure-vessel formula unless a thickness is typed. The design values below depend on the Cargo and, for LPG, on the carriage condition; each shows its source or marks an estimate.
group-dry-bulk-hold = Dry-bulk hold
group-aft-block = Aft block
group-waterway-limits = Waterway Limits
group-speed-and-power = Speed and power
group-cost-rates = Cost rates
group-note-cost-rates = Example rates — replace with yours.

## Inputs

input-cargo = Cargo
choice-cargo-lng = LNG
choice-cargo-lng-ethane = LNG / Ethane
choice-cargo-lpg-propane = LPG (Propane)
choice-cargo-lpg-butane = LPG (Butane)
choice-cargo-lpg-mix = LPG (60/40 Propane/Butane)
choice-cargo-bauxite = Bauxite

input-hull-form = Hull Form
choice-hull-form-raked-box = Raked box

input-formation = Formation
choice-formation-self-propelled = Self-propelled
choice-formation-pushed-barge = Pushed barge
choice-formation-pushed-convoy = Pushed convoy

input-length-overall-m = Length overall

input-beam-m = Beam

input-moulded-depth-m = Moulded depth

input-vessel-scheme = Class Scheme

input-vessel-class = Vessel Class

input-cargo-containment = Cargo Containment
input-cargo-containment-default-by = cargo_phase
choice-cargo-containment-box-hold = Box hold
choice-cargo-containment-sloped-hopper-hold = Sloped hopper hold
choice-cargo-containment-type-c-tanks = Type C tanks

input-block-coefficient = Block coefficient

input-min-freeboard-m = Minimum freeboard

input-bow-rake-length-to-depth-ratio = Bow rake length over depth

input-pushed-stern-void-m = Stern void of a pushed barge

input-lightship-factor-t-per-m3 = Lightship factor
input-lightship-factor-t-per-m3-default-by = cargo_containment

input-ruleset = Ruleset
choice-ruleset-igc-2g = IGC_2G

input-lpg-carriage-condition = LPG carriage condition
choice-lpg-carriage-condition-fully-pressurised = Fully pressurised
choice-lpg-carriage-condition-semi-pressurised = Semi-pressurised (semi-refrigerated)
choice-lpg-carriage-condition-fully-refrigerated = Fully refrigerated (cryogenic)

input-lpg-loading-limit-basis = LPG density and loading limit basis
choice-lpg-loading-limit-basis-igc-15-1-3 = IGC 15.1.3 reference temperature (default)
choice-lpg-loading-limit-basis-igc-15-5-2 = IGC 15.5.2: 45 C reference when pressurised
choice-lpg-loading-limit-basis-refrigerated-density = Before: refrigerated density, fill ratio as loaded

input-lpg-loading-temperature-c = LPG loading temperature
input-lpg-loading-temperature-c-default-by = Cargo, LPG carriage condition

input-fill-ratio = Fill ratio
input-fill-ratio-default-by = Cargo, LPG carriage condition

input-max-tank-od-m = Maximum tank outer diameter

input-tank-design-pressure-mpa = Tank design pressure (MARVS)
input-tank-design-pressure-mpa-default-by = Cargo, LPG carriage condition

input-tank-design-temperature-c = Tank design temperature
input-tank-design-temperature-c-default-by = Cargo, LPG carriage condition

input-tank-material = Tank material
input-tank-material-default-by = Cargo, LPG carriage condition
choice-tank-material-304l = 304L stainless steel
choice-tank-material-316l = 316L stainless steel
choice-tank-material-9ni = 9 % nickel steel
choice-tank-material-s690ql1 = S690QL1 quenched and tempered steel

input-tank-liquid-pressure = Tank liquid pressure
choice-tank-liquid-pressure-static = Static (a = 1), IGC 4.28.1.2
choice-tank-liquid-pressure-igc-4-28-2 = IGC 4.28.2 sea-going accelerations
choice-tank-liquid-pressure-none = None

input-tank-allowable-stress-basis = Tank allowable stress basis
choice-tank-allowable-stress-basis-igc-4-23-3-1 = IGC 4.23.3.1: lesser of Rm/A and Re/B
choice-tank-allowable-stress-basis-published-design = Before the IGC check (S690QL1 at 285 MPa)

input-tank-allowable-stress-mpa = Tank allowable stress (override)

input-tank-insulation-type = Tank insulation type
input-tank-insulation-type-default-by = Cargo, LPG carriage condition
choice-tank-insulation-type-unspecified = Unspecified
choice-tank-insulation-type-none = None
choice-tank-insulation-type-pu-foam = Polyurethane foam
choice-tank-insulation-type-panel = Insulation panel

input-tank-insulation-thickness-mm = Tank insulation thickness
input-tank-insulation-thickness-mm-default-by = Cargo, LPG carriage condition

input-tank-structure-factor = Tank structure factor
input-tank-structure-factor-default-by = Cargo, LPG carriage condition

input-tanks-in-line = Tanks in line

input-tank-support-height-m = Tank support height

input-tank-dome-height-m = Tank dome height (override)
input-tank-dome-height-m-default-by = Cargo, LPG carriage condition

input-tank-dome-diameter-m = Tank dome diameter (override)
input-tank-dome-diameter-m-default-by = Cargo, LPG carriage condition

input-wing-tank-gap-m = Wing tank gap

input-wing-tank-height-ratio = Wing tank height over tank diameter

input-tank-wall-thickness-mm = Tank wall thickness (override)

input-optimise-depth = Optimise depth

input-hold-wing-void-width-m = Wing void width

input-hold-double-bottom-m = Hold double bottom

input-hopper-slope-h-per-v = Hopper slope (horizontal over vertical)

input-hatch-coaming-height-m = Hatch coaming height

input-n-holds = Number of holds

input-bulk-end-bulkhead-m = End bulkhead allowance

input-hold-fill-ratio = Hold fill ratio

input-inner-bottom-plate-mm = Inner bottom plate thickness

input-hopper-plate-mm = Hopper plate thickness

input-hold-bulkhead-plate-mm = Hold bulkhead plate thickness

input-hatch-coaming-plate-mm = Hatch coaming plate thickness

input-propulsive-efficiency = Propulsive efficiency

input-max-speed-kn = Maximum speed

input-thruster-count = Thruster count (most)

input-thruster-envelope-m = Thruster envelope

input-thruster-clearance-m = Thruster clearance

input-thruster-room-length-m = Thruster room length

input-battery-unit-count = Battery units
input-battery-unit-count-default-by = cargo_phase

input-accommodation-unit-count = Accommodation units
input-accommodation-unit-count-default-by = cargo_phase

input-unit-height-m = Unit height

input-unit-side-gap-m = Unit side gap

input-unit-row-gap-m = Unit row gap

input-unit-foundation-m = Unit foundation height

input-wheelhouse-length-m = Wheelhouse length

input-wheelhouse-width-m = Wheelhouse width

input-wheelhouse-height-m = Wheelhouse height

input-wheelhouse-gap-m = Wheelhouse gap

input-wheelhouse-eye-height-m = Wheelhouse eye height above floor

input-sightline-margin-m = Sightline margin

input-wheelhouse-max-lift-m = Wheelhouse maximum lift

input-air-draft-limit-m = Air-draft limit

input-draught-limit-m = Draught limit

input-channel-depth-m = Channel depth

input-channel-under-keel-clearance-m = Under-keel clearance

input-channel-bottom-width-m = Channel bottom width

input-channel-bank-slope = Bank slope

input-channel-current-kn = Current

input-water-density-t-per-m3 = Water density

input-squat-safety-margin-m = Squat safety margin

input-hull-steel-rate = Hull steel rate

input-tank-shell-rate = Tank shell rate

input-tank-supports-rate = Tank supports rate

input-insulation-rate = Insulation rate

input-piping-rate = Piping and machinery rate

input-outfitting-rate = Outfitting rate

input-cargo-system-lump = Cargo system

input-paint-anodes-misc = Paint, anodes and miscellaneous

input-battery-rate-per-kwh = Battery rate

input-propulsion-rate-per-kw = Propulsion rate

input-wheelhouse-accom-lump = Wheelhouse and accommodation

input-tug-outfitting-lump = Tug outfitting

input-yard-margin-fraction = Yard margin

input-delivery-lump = Delivery
