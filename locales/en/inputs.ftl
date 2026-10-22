# The input schema's labels, choices and group names (spec 0005).
# Written by crates/ovcd-format/tests/i18n_inputs.rs from the schema. Do not edit: change the
# label in the Rust source and run `OVCD_WRITE_TEXT=1 cargo test -p ovcd-format --test i18n_inputs`.

## Groups

group-required = Required
group-vessel-class = Vessel Class
group-hull-and-containment = Hull and containment
group-hull-form = Hull Form
group-gas-tanks = Gas tanks
group-note-gas-tanks = The tank outer diameter (the cap below and the result) is measured to the outside of the insulation; the clearances and the drawing use that face. The steel diameter is that less twice the insulation thickness. The wall thickness is worked out from the pressure-vessel formula unless a thickness is typed. The design values below depend on the Cargo and, for LPG, on the carriage condition; each shows its source or marks an estimate.
group-dry-bulk-hold = Dry-bulk hold
group-liquid-cargo-tanks = Liquid cargo tanks
group-note-liquid-cargo-tanks = Rectangular cargo tanks inside a double hull, between the cofferdams at the ends of the cargo zone. The double hull widths and the cofferdam length are the ADN minimums for a type C vessel until the Rule Sets for liquid cargo exist; each value shows its source or marks an estimate. Crude oil is work in development.
group-aft-block = Aft block
group-openings = Openings
group-stern-and-propulsion = Stern and propulsion
group-tug = Tug
group-waterway-limits = Waterway Limits
group-rules = Rules
group-loading-conditions = Loading conditions
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
choice-cargo-crude-oil = Crude oil
choice-cargo-diesel-en590 = Diesel fuel (EN 590)

input-hull-form = Hull Form
choice-hull-form-box = Box
choice-hull-form-raked-box = Raked box
choice-hull-form-v-bottom = V-bottom
choice-hull-form-chine = Chine
choice-hull-form-shipd = ShipD-style
choice-hull-form-offsets = Offsets table

input-formation = Formation
choice-formation-self-propelled = Self-propelled
choice-formation-pushed-barge = Pushed barge
choice-formation-pushed-convoy = Pushed convoy
choice-formation-atb = Articulated tug-barge (ATB)
choice-formation-towed-barge = Towed barge

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
choice-cargo-containment-integral-tanks = Integral tanks

input-bow-rake-length-m = Bow rake length

input-bow-rake-rise-m = Bow rake rise

input-bilge-radius-m = Bilge radius

input-sheer-aft-m = Sheer at the aft end

input-sheer-fwd-m = Sheer at the forward end

input-camber-m = Deck camber

input-deadrise-deg = Deadrise

input-keel-half-width-m = Keel half-width

input-chine-half-breadth-m = Chine half-breadth

input-chine-height-m = Chine height above base

input-shipd-parameters = ShipD parameters

input-hull-offsets = Offsets table

input-min-freeboard-m = Minimum freeboard (safety clearance)

input-bow-rake-length-to-depth-ratio = Bow rake length over depth

input-pushed-stern-void-m = Stern void of a pushed barge

input-lightship-factor-t-per-m3 = Lightship factor
input-lightship-factor-t-per-m3-default-by = cargo_containment

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

input-tank-head-type = Tank head type
input-tank-head-type-default-by = Cargo, LPG carriage condition
choice-tank-head-type-hemisphere = Hemispherical
choice-tank-head-type-ellipsoidal-2to1 = 2:1 semi-ellipsoidal

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

input-weather-shield-thickness-mm = Weather shield plate thickness

input-weather-shield-clearance-m = Weather shield clearance over the tanks

input-tank-forming-allowance-pct = Tank plate forming allowance

input-tank-dome-wall-thickness-mm = Tank dome wall thickness

input-aft-end-space-m = Aft end-bulkhead space

input-fwd-end-space-m = Forward end-bulkhead space

input-opening-setback-basis = Opening setback measured from
choice-opening-setback-basis-tank-end = From the tank end
choice-opening-setback-basis-cargo-area = From the cargo area

input-tank-wall-thickness-mm = Tank wall thickness (override)

input-optimise-depth = Optimise depth

input-hold-wing-void-width-m = Wing void width

input-hold-double-bottom-m = Hold double bottom

input-hopper-slope-h-per-v = Hopper slope (horizontal over vertical)

input-hatch-coaming-height-m = Hatch coaming height

input-hold-covering = Hold covering
choice-hold-covering-covered = Covered
choice-hold-covering-open = Open

input-n-holds = Number of holds

input-bulk-end-bulkhead-m = End bulkhead allowance

input-hold-fill-ratio = Hold fill ratio

input-inner-bottom-plate-mm = Inner bottom plate thickness

input-hopper-plate-mm = Hopper plate thickness

input-hold-bulkhead-plate-mm = Hold bulkhead plate thickness

input-hatch-coaming-plate-mm = Hatch coaming plate thickness

input-double-side-m = Double side width

input-double-bottom-m = Double bottom height

input-liquid-tanks-in-line = Cargo tanks in line

input-centreline-bulkhead = Centreline bulkhead

input-cofferdam-m = Cofferdam length

input-pump-arrangement = Pump arrangement
choice-pump-arrangement-deepwell = Deepwell pumps
choice-pump-arrangement-pump-room = Pump room

input-pump-room-length-m = Pump room length

input-heating-kw-per-m3 = Cargo heating

input-liquid-fill-ratio = Cargo tank fill ratio

input-stern-form = Stern form
input-stern-form-default-by = cargo_phase
choice-stern-form-transom = Transom
choice-stern-form-pram = Pram
choice-stern-form-tunnel = Tunnel

input-stern-run-length-m = Stern run length

input-stern-transom-height-m = Transom height above base

input-tunnel-crown-height-m = Tunnel crown height above base

input-propulsor-type = Propulsor type
input-propulsor-type-default-by = cargo_phase
choice-propulsor-type-unspecified = Unspecified
choice-propulsor-type-azimuth-open = Azimuth thruster, open
choice-propulsor-type-azimuth-ducted = Azimuth thruster, ducted
choice-propulsor-type-shaft-open = Shaft and open propeller
choice-propulsor-type-shaft-nozzle = Shaft and propeller in a nozzle
choice-propulsor-type-voith-schneider = Voith Schneider
choice-propulsor-type-pump-jet = Pump jet

input-propulsor-count = Propulsor count

input-propeller-diameter-m = Propeller diameter

input-propulsor-efficiency-factor = Propulsor efficiency factor
input-propulsor-efficiency-factor-default-by = propulsor_type

input-bollard-pull-factor = Bollard pull factor
input-bollard-pull-factor-default-by = propulsor_type

input-power-source = Power source
input-power-source-default-by = cargo_phase
choice-power-source-battery-electric = Battery-electric
choice-power-source-gas-fuelled = Gas-fuelled
choice-power-source-diesel-mechanical = Diesel-mechanical
choice-power-source-diesel-electric = Diesel-electric
choice-power-source-battery-hybrid = Battery hybrid

input-installed-power-kw = Installed power

input-design-speed-kn = Design speed

input-installed-power-margin = Installed power margin

input-bow-thruster = Bow thruster
choice-bow-thruster-none = None
choice-bow-thruster-channel-jet = Channel jet
choice-bow-thruster-tunnel = Tunnel

input-bow-thruster-power-kw = Bow thruster power

input-battery-energy-kwh = Battery energy

input-trip-length-km = Trip length

input-battery-unit-energy-kwh = Battery energy per 40 ft unit

input-wheelhouse-type = Wheelhouse type
choice-wheelhouse-type-elevating = Elevating
choice-wheelhouse-type-fixed = Fixed

input-sightline-method = Sightline method
choice-sightline-method-margin = Margin over the obstruction
choice-sightline-method-blind-distance = Blind distance (ES-TRIN 7.02)

input-stern-block-mass-included = Stern block mass in the lightship

input-tug-included = Tug included

input-tug-preset = Tug preset
choice-tug-preset-none = None
choice-tug-preset-cds-4115 = CDS 4115 River Pusher
choice-tug-preset-sdp-4600 = SDP 4600 pusher
choice-tug-preset-rapide-4600-z3 = RApide 4600-Z3 pusher
choice-tug-preset-riverbuster-1909 = CDS Riverbuster 1909
choice-tug-preset-demi-k = Demi-K pusher tug
choice-tug-preset-bowling-green = Bowling Green towboat
choice-tug-preset-arne-christiansen = Arne Christiansen pusher
choice-tug-preset-stan-tug-1606 = Stan Tug 1606
choice-tug-preset-stan-tug-1907 = Stan Tug 1907
choice-tug-preset-asd-tug-2312 = Azimuth Stern Drive Tug 2312
choice-tug-preset-asd-tug-2811 = Azimuth Stern Drive Tug 2811
choice-tug-preset-asd-tug-3212 = Azimuth Stern Drive Tug 3212
choice-tug-preset-rsd-tug-2513-electric = Reversed Stern Drive Tug 2513 Electric
choice-tug-preset-bess = Bess (Voith tractor)
choice-tug-preset-baut = Baut (Voith tractor)
choice-tug-preset-aveogan = Aveogan (ATB tug)
choice-tug-preset-crowley-550-class = 550 Class ATB tug
choice-tug-preset-kim-m-bouchard = Kim M. Bouchard (ATB tug)

input-tug-length-m = Tug length

input-tug-beam-m = Tug beam

input-tug-draught-m = Tug draught

input-tug-stern-form = Tug stern form
input-tug-stern-form-default-by = formation
choice-tug-stern-form-transom = Transom
choice-tug-stern-form-pram = Pram
choice-tug-stern-form-tunnel = Tunnel

input-tug-stern-run-length-m = Tug stern run length

input-tug-stern-transom-height-m = Tug transom height

input-tug-tunnel-crown-height-m = Tug tunnel crown height

input-tug-propulsor-type = Tug propulsor type
input-tug-propulsor-type-default-by = formation
choice-tug-propulsor-type-azimuth-open = Azimuth thruster, open
choice-tug-propulsor-type-azimuth-ducted = Azimuth thruster, ducted
choice-tug-propulsor-type-shaft-open = Shaft and open propeller
choice-tug-propulsor-type-shaft-nozzle = Shaft and propeller in a nozzle
choice-tug-propulsor-type-voith-schneider = Voith Schneider
choice-tug-propulsor-type-pump-jet = Pump jet

input-tug-propulsor-count = Tug propulsor count

input-tug-propeller-diameter-m = Tug propeller diameter

input-tug-power-source = Tug power source
choice-tug-power-source-battery-electric = Battery-electric
choice-tug-power-source-gas-fuelled = Gas-fuelled
choice-tug-power-source-diesel-mechanical = Diesel-mechanical
choice-tug-power-source-diesel-electric = Diesel-electric
choice-tug-power-source-battery-hybrid = Battery hybrid

input-tug-installed-power-kw = Tug installed power

input-tug-design-speed-kn = Formation design speed

input-tug-installed-power-margin = Tug installed power margin

input-tug-bow-thruster = Tug bow thruster
choice-tug-bow-thruster-none = None
choice-tug-bow-thruster-channel-jet = Channel jet
choice-tug-bow-thruster-tunnel = Tunnel

input-tug-bow-thruster-power-kw = Tug bow thruster power

input-tug-battery-energy-kwh = Tug battery energy

input-tug-battery-unit-energy-kwh = Tug battery unit energy

input-tug-wheelhouse-type = Tug wheelhouse type
choice-tug-wheelhouse-type-elevating = Elevating
choice-tug-wheelhouse-type-fixed = Fixed

input-tug-sightline-method = Tug sightline method
choice-tug-sightline-method-margin = Margin over the obstruction
choice-tug-sightline-method-blind-distance = Blind distance (ES-TRIN 7.02)

input-formation-resistance-allowance = Formation resistance allowance
input-formation-resistance-allowance-default-by = formation

input-notch-depth-m = Notch depth

input-notch-width-m = Notch width

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

input-lock-length-m = Lock length

input-lock-width-m = Lock width

input-downflooding-angle-deg = Angle of down-flooding

input-downflooding-openings = Openings for down-flooding

input-channel-depth-m = Channel depth

input-channel-under-keel-clearance-m = Under-keel clearance

input-channel-bottom-width-m = Channel bottom width

input-channel-bank-slope = Bank slope

input-channel-current-kn = Current

input-water-density-t-per-m3 = Water density

input-squat-safety-margin-m = Squat safety margin

input-loading-conditions = Loading conditions
choice-loading-conditions-legacy = Laden, half laden and light (as before)
choice-loading-conditions-is-code-2008 = IS Code 2008 standard conditions, with ballast

input-may-sail-without-ballast = May sail without ballast
input-may-sail-without-ballast-default-by = formation

input-endurance-h = Endurance (running hours)

input-fixed-stores-t = Fresh water and stores

input-ballast-plan = Ballast plan
choice-ballast-plan-least-ballast = The least ballast that meets every criterion
choice-ballast-plan-user-set = Fills set by the user

input-ballast-departure-double-bottom-fill = Ballast departure: double bottom fill

input-ballast-departure-peak-fill = Ballast departure: peak tanks fill

input-ballast-departure-wing-fill = Ballast departure: wing tanks fill

input-ballast-arrival-double-bottom-fill = Ballast arrival: double bottom fill

input-ballast-arrival-peak-fill = Ballast arrival: peak tanks fill

input-ballast-arrival-wing-fill = Ballast arrival: wing tanks fill

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

input-tank-coating-rate = Tank coating rate

input-cargo-pump-lump = Cargo pump

input-cargo-piping-lump = Cargo piping

input-cargo-heating-rate-per-kw = Cargo heating rate

input-vents-gauging-lump = Vents and gauging

input-tank-internals-fraction = Tank internals share of the structure surplus

input-currency = Currency

input-price-base-date = Price base date

input-deckhouse-front-opening = Entrance, air inlet or opening on the deckhouse front

input-deckhouse-side-opening-distance-m = Distance of the first side opening from the deckhouse front

input-entrance-distance-m = Distance of the nearest entrance or openable window from the cargo area

input-engine-room-vent-inlet-distance-m = Distance of the engine room ventilation inlet from the cargo area

input-funnel-outlet-distance-m = Distance of the funnel outlet from the cargo area

input-service-area = Service Area
choice-service-area-river-only = River only
choice-service-area-coastal-and-river = Coastal and river

input-slop-share = Slop tank share
input-slop-share-default-by = service_area

input-rule-sets = Rule Sets
