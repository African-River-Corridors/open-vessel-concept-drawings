# The app's own text (spec 0005). English is the source language of the Text Library.
# Ids are kebab case with the prefix app-. Numbers arrive as text, already rounded.

## Header

app-new = New
app-open = Open
app-save = Save
app-save-as = Save as
app-about = About
app-web-version = Web version
app-download-desktop = Download the desktop app
app-input-panel = Input panel
app-input-panel-show = Show the Input panel
app-input-panel-hide = Hide the Input panel
app-outputs-panel = Outputs panel
app-outputs-panel-show = Show the Outputs panel
app-outputs-panel-hide = Hide the Outputs panel
app-file-unsaved = { $file } (unsaved)
app-header-profile = Defaults: { $profile }
app-engine-version = Engine { $version }
app-language = Language
app-theme = Theme
app-theme-auto = Auto
app-theme-light = Light
app-theme-dark = Dark

## Banners and common words

app-dismiss = Dismiss
app-msg-bar = Messages
app-msg-count = { $index } of { $count }
app-msg-previous = Previous message
app-msg-next = Next message
app-msg-show-all = All
app-msg-show-more = More
app-msg-show-less = Less
app-hash-changed = Results changed since this file was saved. Options affected: { $options }.
app-ok = OK
app-cancel = Cancel
app-none = none
app-yes = yes
app-no = no
app-parameter-count = Parameters: { $count }
app-offset-station-count = Stations: { $count }
app-opening-count = Openings: { $count }
app-import-offsets = Import CSV
app-pass = PASS
app-fail = FAIL
app-out-of-date = out of date
app-range-to = { $min } to { $max }
app-unsaved-discard = The Design has unsaved changes. Discard them?
app-file-name = File name

## The first-run notice

app-notice-title = Before you start
app-notice-version = Version
app-notice-licence = Licence
app-notice-scope = Concept design. Results are not verified.
app-notice-tool = OVCD is a concept design tool.
app-notice-unverified = Its results are not verified. Known engineering findings are under review.
app-notice-warranty = There is no warranty. OVCD is open source under the Apache License 2.0.
app-notice-accept = I understand

## Left panel: Options and the Defaults Profile

app-option = Option
app-option-active = Active Option
app-option-add = Add
app-option-duplicate = Duplicate
app-option-rename = Rename
app-option-delete = Delete
app-option-delete-last = A Design needs at least one Option
app-option-new-name = New Option name
app-option-delete-confirm = Delete "{ $option }"?
app-profile = Defaults Profile
app-profile-load = Load from file
app-profile-builtin = Built-in

## Left panel: one input

app-source-typed = typed
app-source-default = default
app-source-class = class
app-reset = reset
app-reset-title = Go back to the default
app-choice-not-typed = { $value } ({ $source })
app-range-and = { $first } and { $second }
app-help-default = Default: { $value }
app-help-default-by = Default by { $by }: { $values }
app-help-default-rule = Default when left out: the beam times a ratio by Cargo ({ $ratios }), to the millimetre
app-help-depth-warning = Warns when length over depth is above { $limit }
app-help-rule-basis = Rule basis: { $basis }
app-help-hard-range = Hard range: { $range }
app-help-soft-range = Soft range: { $min } to { $max }
app-help-basis = Basis: { $basis }
app-cargo-group-dry-bulk = dry bulk, container
app-cargo-group-liquid = liquid
app-cargo-group-gas = gas
app-convoy-abreast = Abreast
app-convoy-in-line = In line

## Left panel: Vessel Class

app-vessel-mode = How the vessel is defined
app-vessel-mode-typed = Typed dimensions
app-vessel-mode-class = Standard class
app-class-scheme = Class Scheme
app-class-scheme-choose = Choose a Class Scheme
app-class = Class
app-vessel-class = Vessel Class
app-class-choose = Choose a class
app-class-none-for-kind = No class of this scheme for a { $kind } Formation.
app-formation-self-propelled = self-propelled
app-formation-pushed-barge = pushed barge
app-formation-pushed-convoy = pushed convoy
app-formation-atb = articulated tug-barge (ATB)
app-formation-towed-barge = towed barge
app-class-sits-in = The vessel sits in { $class }
app-class-dimensions = Up to { $length } m long and { $beam } m wide.
app-class-draught = Draught { $draught }; air draft { $airDraft }.
app-class-decides = Length and beam decide the class; the draught and air-draft limits are filled from it.
app-stale-why = Not recalculated: fix the error under { $inputs }

## Views panel

app-views = Views
app-view-groups = View groups
app-views-of = Views of { $group }
app-group-drawings = Drawings
app-view-side = Side elevation
app-view-plan = Plan
app-view-midship = Cross-section
app-group-stability-propulsion = Stability & Propulsion
app-view-stability = Intact stability
app-view-ballast = Ballast tanks
app-view-convoy = Convoy totals
app-view-3d = 3D
app-view-speed-power = Power and speed curves
app-view-rules = Rule check calculations
app-group-boq-finance = BOQ & Finance
app-view-boq = BOQ
app-view-capital = Capital budget
app-no-results-draw = No results to draw.
app-no-results-chart = No results to chart.
app-no-results-show = No results to show.
app-boq-later = The bill of quantities comes in a later phase. See the roadmap.
app-zoom-view = Zoom { $view }
app-zoom-out = Zoom out
app-zoom-in = Zoom in
app-fit-panel = Fit to the panel
app-fit = Fit
app-zoom-fitted = fitted
app-zoom-zoomed = zoomed
app-work-in-development = Work in development
app-cargo-development = { $name } (work in development)

## Bottom panel

app-evaluating = Evaluating...
app-outputs = Outputs
app-checks = Checks
app-no-checks = No checks without results.
app-warnings = Warnings
app-tried = Arrangements tried
app-tried-depth = Depth (m)
app-tried-arrangement = Across × along
app-tried-tank-od = Tank OD (m)
app-tried-side-space = Side space (m)
app-tried-wing-tank = Wing tank (m)
app-tried-cargo-volume = Cargo (m³)
app-tried-outcome = Outcome
app-tried-chosen = Chosen
app-tried-less-cargo = Less cargo
app-tried-less-cargo-title = It passes the quick checks but carries less than the chosen arrangement, so its stability was not assessed.
app-tried-fails = Fails
app-tried-not-built = Not built
app-compare = Compare
app-make-active = Make this the active Option
app-status = Status
app-results-hash = Results hash
app-class-reference-dwt = Class reference deadweight ({ $class })
app-class-reference-dwt-title = The class's published figure, for comparison. It is never an input.
app-not-applicable = not applicable
app-metric-cargo = Cargo
app-metric-governs = Governed by
app-metric-cargo-volume = Cargo volume
app-metric-lightship = Lightship
app-metric-displacement = Displacement
app-metric-deadweight = Deadweight
app-metric-draught = Draught
app-metric-draught-max = Maximum draught
app-metric-light-draught = Light draught
app-metric-freeboard = Freeboard
app-load-basis-note = Draught and freeboard are for the full load: the cargo and { $consumables } t of stores and fuel. The freeboard checks take the cargo alone, so their draught is a little less.
app-check-basis-cargo-only = cargo alone, no stores or fuel
app-metric-air-draft-loaded = Air draft, loaded
app-metric-air-draft-light = Air draft, light
app-metric-depth-to-draught = Depth to draught (h/T)
app-metric-tanks-across = Tanks across
app-metric-tanks-along = Tanks along
app-metric-tank-od = Tank outer diameter
app-metric-side-clearance = Side clearance
app-metric-ruleset = Ruleset
app-metric-scope = Scope of the check
app-unverified-count = { $count } unverified
app-metric-fill-ratio = Fill ratio
app-metric-depth-optimised = Depth optimised
app-metric-installed-power = Installed power
app-metric-bollard-pull = Bollard pull
app-metric-service-speed-deep = Service speed, deep water
app-metric-service-speed-channel = Service speed, channel
app-metric-battery-energy = Battery energy
app-metric-block-coefficient = Block coefficient (Cb)
app-metric-midship-coefficient = Midship coefficient (Cm)
app-metric-prismatic-coefficient = Prismatic coefficient (Cp)
app-metric-waterplane-coefficient = Waterplane coefficient (Cwp)
app-metric-waterline-length = Waterline length
app-metric-kb = Centre of buoyancy above keel (KB)
app-metric-bm-transverse = Transverse metacentric radius (BM)
app-metric-km-transverse = Transverse metacentre above keel (KM)
app-metric-lcb = Centre of buoyancy from aft end (LCB)
app-metric-lcf = Centre of flotation from aft end (LCF)
app-metric-waterplane-area = Waterplane area
app-metric-wetted-surface = Wetted surface
app-metric-tpc = Tonnes per centimetre immersion
app-metric-mct = Moment to change trim 1 cm

## Rule checks view

app-rules-none = The engine reported no checks for this Option.
app-rules-trace-count = { $failing } of { $total ->
        [one] { $total } rule check
       *[other] { $total } rule checks
    } { $failing ->
        [one] fails
       *[other] fail
    }
app-rules-col-result = Result
app-rules-col-rule = Rule
app-rules-col-object = Object
app-rules-col-value = Value
app-rules-col-limit = Limit
app-rules-col-margin = Margin
app-rules-col-citation = Citation
app-rules-col-status = Status
app-rules-warn = WARN
app-rules-note = NOTE
app-rules-not-evaluated = NOT EVALUATED
app-rules-not-applicable = NOT APPLICABLE
app-rules-yes = yes
app-rules-no = no
app-rules-governing = Governing
app-rules-governing-hint = The strictest Rule of its requirement: it decides whether the requirement is met.
app-rules-sizes = Sizes the arrangement
app-rules-finding = Finding { $finding }
app-rules-ovcd-check = OVCD Check
app-rules-verified = Verified
app-rules-unverified = Unverified
app-rules-method-unverified = Method unverified

## Speed and power view

app-sp-speed-axis = Speed (kn)
app-sp-power-axis = Power (kW)
app-sp-fuel-axis = Fuel (l/h)
app-sp-squat-series = Squat-limited (kW)
app-sp-power = Power
app-sp-fuel = Fuel
app-sp-limit-line = limit { $speed } kn
app-sp-head-title = limiting speed { $speed }, blockage { $blockage }
app-sp-limiting-speed = limiting speed { $speed } kn
app-sp-no-limit = no speed limit from the channel
app-sp-blockage = · blockage { $blockage }
app-sp-squat-points = · { $count ->
        [one] { $count } squat-limited point
       *[other] { $count } squat-limited points
    }
app-sp-selected = Selected point
app-sp-r-speed = speed
app-sp-r-power = power
app-sp-r-fuel = fuel
app-sp-r-squat = squat
app-sp-r-squat-limited = squat-limited
app-sp-r-limiting-speed = limiting speed

## Stability view

app-stab-wip = Stability is computed for gas cargo only. Dry-bulk stability is work in development.
app-stab-heel-axis = Heel angle (deg)
app-stab-gz-axis = GZ (m)
app-stab-governs = governs
app-stab-legend = GM { $gm } m · max GZ { $gz } m at { $angle }°
app-stab-zoom = Zoom the GZ curves
app-stab-governing = Governing condition:
app-stab-condition = Condition
app-stab-col-cargo = Cargo
app-stab-col-fill = Fill
app-stab-col-displacement = Displacement
app-stab-col-draught = Draught
app-stab-col-kg = KG
app-stab-col-gm-solid = GM solid
app-stab-col-free-surface = Free surface
app-stab-col-gm-fluid = GM fluid
app-stab-col-max-gz = Max GZ
app-stab-col-max-gz-angle = Angle of max GZ
app-stab-col-deck-edge = Deck-edge angle
app-stab-criteria-of = { $condition }: criteria
app-stab-needs-at-least = needs at least
app-stab-limitations = Limitations
app-stab-limit-downflooding = The 0–40° and 30–40° areas run to 40° unless you type an angle of down-flooding below 40° (the input Angle of down-flooding). IS Code 2008 Part A 2.2.1 stops them at that angle when it is less than 40°. List the deck openings (the input Openings for down-flooding) and the model works the angle out for each loading condition. With neither typed nor listed, a hull whose openings immerse sooner can pass when it should fail (findings #282, #592).

## Capital budget view

app-cost-wip = The capital budget is computed for gas cargo and crude oil only. A dry-bulk budget is work in development.
app-cost-none = There is no capital budget for this Option, because the hull holds no tank arrangement to cost.
app-cost-currency = All amounts are in { $currency }.
app-cost-prices-current = Current estimates.
app-cost-prices-as-of = Prices as of { $date }.
app-cost-basis-example = Example rates — replace with yours.
app-cost-basis-user = Your rates.
app-cost-barge = Barge
app-cost-tug = Tug
app-cost-barge-subtotal = Barge subtotal
app-cost-tug-subtotal = Tug subtotal
app-cost-unit = Unit
app-cost-yard-margin = Yard margin ({ $percent }%)
app-cost-delivery = Delivery
app-cost-unit-total = Unit total
app-cost-from-design = Mass from this design: { $masses }.
app-cost-core-defaults = Core defaults, not from this design: { $masses }.
app-cost-check-failed = Check failed: { $check }.
app-cost-note = Note: { $check }.
app-cost-item-hull-steel = Hull steel
app-cost-steel-delta = Priced: { $bottom_up } t bottom-up. The lightship uses { $top_down } t top-down, a difference of { $delta } t.
app-cost-item-tank-coating = Tank coating
app-cost-item-cargo-pumps = Cargo pumps
app-cost-item-cargo-piping = Cargo piping
app-cost-item-cargo-heating = Cargo heating
app-cost-item-vents-gauging = Vents and gauging
app-cost-item-type-c-tanks = Type C tanks
app-cost-item-tank-supports = Tank supports
app-cost-item-insulation = Insulation
app-cost-item-cargo-system = Cargo system
app-cost-item-piping-machinery = Piping and machinery
app-cost-item-outfitting = Outfitting
app-cost-item-paint-anodes-misc = Paint, anodes and miscellaneous
app-cost-item-battery = Battery
app-cost-item-propulsion = Propulsion
app-cost-item-wheelhouse-accommodation = Wheelhouse and accommodation

## The splash screen

app-splash-starting = OVCD is starting
app-splash-tagline = An open instrument for the { $design } of inland and coastal vessels.
app-splash-tagline-design = concept and preliminary design
app-splash-state = State
app-splash-starting-engine = Starting the engine
app-splash-notice = Its results are not verified. Known engineering findings are under review. There is no warranty.
app-cost-internals = Tank internals: { $percent }% of the tank structure surplus is priced at the tank shell rate. This share is an assumption (an Estimate, no source read); the rest is priced at the supports rate.

# The ballast view (issue #426)
app-ballast-none = No ballast tanks to show. They are listed for a Design that is checked in the IS Code loading conditions.
app-ballast-count = { $count } ballast tanks
app-ballast-capacity = Capacity { $m3 } m³, { $t } t of water
app-ballast-of = { $condition }: { $mass } t of ballast
app-ballast-no-plan = No ballast plan meets every criterion in this condition, even with every ballast tank full.
app-ballast-col-tank = Tank
app-ballast-col-kind = Kind
app-ballast-col-side = Side
app-ballast-col-capacity-m3 = Capacity
app-ballast-col-capacity-t = Capacity
app-ballast-col-fill = Fill
app-ballast-col-mass = Mass
app-ballast-col-lcg = LCG
app-ballast-col-tcg = TCG
app-ballast-col-vcg = VCG
app-ballast-col-free-surface = Free surface moment
app-ballast-kind-double-bottom = Double bottom
app-ballast-kind-wing = Wing
app-ballast-kind-peak = Peak
app-ballast-side-port = Port
app-ballast-side-starboard = Starboard
app-ballast-side-centre = Centre

# The convoy totals (issue #551)
app-convoy-formation = { $abreast } abreast, { $in_line } in line: { $barges } barges
app-convoy-quantity = Quantity
app-convoy-one-barge = One barge
app-convoy-total = Convoy
app-convoy-cargo = Cargo
app-convoy-lightship = Lightship
app-convoy-deadweight = Deadweight
app-convoy-displacement = Displacement
app-convoy-cargo-volume = Cargo volume
app-convoy-overall-length = Overall length
app-convoy-overall-beam = Overall beam
app-convoy-pusher-note = The pusher is not in these totals. It joins them when the pusher is sized.

# The 3D view (issue #458)
app-3d-loading = Loading the 3D model.
app-3d-failed = The 3D model could not be built: { $why }
app-3d-no-webgl = This browser or machine cannot draw 3D (WebGL is not available). The Spaces are listed below instead.
app-3d-canvas-label = 3D model of the hull. Drag to orbit, scroll to zoom, arrow keys to orbit.
app-3d-hint = Drag to orbit, scroll to zoom, right-drag to pan, arrow keys to orbit.
app-3d-reset = Reset view
app-3d-view-profile = Profile
app-3d-view-plan = Plan
app-3d-view-body = Body plan
app-3d-layers = Layers
app-3d-layer-hull = Hull
app-3d-layer-deck = Decks
app-3d-layer-bulkhead = Bulkheads
app-3d-layer-structure = Structure
app-3d-structure-later = from phase 3
app-3d-spaces = Spaces
app-3d-no-spaces = This Option has no Spaces to show.
app-3d-pick-hint = Click a Space, or pick one in the list, to read its volume and centroid.
app-3d-col-name = Name
app-3d-col-kind = Kind
app-3d-col-volume = Volume
app-3d-col-centroid = Centroid (x, y, z)
app-3d-kind-cargo-tank = Cargo tank
app-3d-kind-slop = Slop tank
app-3d-kind-cargo-hold = Cargo hold
app-3d-kind-hold-around-tanks = Hold space (around tanks)
app-3d-layer-dome = Tank domes
app-3d-layer-weather-shield = Weather shield
app-3d-layer-deck-item = Deck Items
app-3d-presets = View
app-3d-preset-exterior = Exterior
app-3d-preset-cargo = Cargo
app-3d-preset-arrangement = Arrangement
app-3d-preset-custom = Custom
app-3d-cut-half = Half model
app-3d-cut-deck = Lift the deck off
app-3d-wheelhouse = Wheelhouse
app-3d-wheelhouse-lowered = Lowered
app-3d-wheelhouse-raised = Raised
app-3d-wheelhouse-both = Both
app-3d-lift-needed = lift needed { $needed } m, allowed { $allowed } m
app-3d-lift-used = lift used { $used } m, allowed { $allowed } m
app-3d-air-draft-light = light air draft { $lowered } m lowered, { $raised } m raised
app-3d-item-accommodation-unit = Accommodation unit
app-3d-item-battery-unit = Battery unit
app-3d-item-wheelhouse-lowered = Wheelhouse, lowered
app-3d-item-wheelhouse-raised = Wheelhouse, raised
app-3d-item-lift-column = Lift column
app-3d-item-size = Size (length × width × height)
app-3d-item-base = Base above the keel
app-3d-item-top = Top above the keel
app-3d-item-lift = Lift needed / used / allowed
app-3d-layer-propulsor = Propulsors
app-3d-part-propulsor = Propulsor
app-3d-part-nozzle = Nozzle of propulsor
app-3d-part-leg = Leg of azimuth unit
app-3d-part-rudder = Rudder
app-3d-part-bow-thruster = Bow thruster
app-3d-propulsor-default-thrusters = Azimuth thruster, ducted (default stern block)
app-3d-propulsor-type = Propulsor type
app-3d-propulsor-diameter = Propeller diameter
app-3d-propulsor-axis = Axis height above the keel
app-3d-propulsor-power = Installed Power per unit
app-3d-propulsor-bow-power = Bow thruster power
app-3d-propulsor-tunnel-diameter = Tunnel diameter
app-3d-propulsor-rudder-size = Rudder span × chord
app-3d-propulsor-mass-centre = Mass item centre (x, z)
app-3d-tank-od = Tank outer diameter
app-3d-tank-length = Tank length (heads included)
app-3d-tank-head = Tank head
app-3d-head-hemisphere = Hemispherical
app-3d-head-ellipsoidal-2to1 = 2:1 ellipsoidal
app-3d-kind-cofferdam = Cofferdam
app-3d-kind-void = Void
app-3d-kind-peak-void = Peak void
app-3d-kind-pump-room = Pump room
app-3d-kind-machinery = Machinery
app-3d-kind-double-bottom-ballast = Double-bottom ballast
app-3d-kind-wing-ballast = Wing ballast
app-3d-kind-peak-ballast = Peak ballast
app-3d-side-port = port
app-3d-side-starboard = starboard
app-3d-side-centre = centre
app-3d-section = Section plane
app-3d-section-x = Position of the section plane along the hull
app-3d-section-at = at x = { $x } m
app-3d-waterline = Waterline
app-3d-waterline-none = None
app-3d-waterline-draughts = draught { $mean } m (aft { $aft } m, forward { $fwd } m), trim { $trim } m
