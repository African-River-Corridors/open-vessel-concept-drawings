# The engine's messages: drawing labels and notes (English only for now: the Drawing Format
# Standard allows ASCII only) (spec 0005, ADR 0015).
# Written by tools/i18n_extract.py from the Rust source. Do not edit: change the English in the
# `text!` call and run the tool again. A number argument arrives as text, already rounded.


## crates/ovd-format/src/ga/arrangement.rs

draw-coupling-points = COUPLING POINTS ({ $count })
draw-overall-length = OVERALL LENGTH { $length }
draw-overall-beam = OVERALL BEAM { $beam }
draw-wheelhouse-raised = RAISED
draw-sightline-light = SIGHTLINE, LIGHT
draw-light-waterline = LIGHT WATERLINE
draw-view-arrangement-plan = ARRANGEMENT PLAN
draw-note-arrangement = { $view }. DIMENSIONS IN METRES. PROFILE IN THE LIGHT CONDITION.
draw-note-barges = { $abreast } ABREAST BY { $in_line } IN LINE.
draw-note-blind-distance = BLIND DISTANCE AHEAD OF THE BOW: LIGHT { $light } M, LADEN { $laden } M, LIMIT { $limit } M.
draw-note-scope = FOR CONCEPT AND PRELIMINARY DESIGN ONLY, NOT FOR APPROVAL.

## crates/ovd-format/src/ga/tug.rs

draw-tug-atb = ATB TUG
draw-tug-towing = TOWING TUG
draw-tug-pusher = PUSHER
draw-wl = WL
draw-push-knees = PUSH KNEES
draw-propeller = PROPELLER
draw-rudder = RUDDER
draw-flanking-rudders = FLANKING RUDDERS
draw-wheelhouse = WHEELHOUSE
draw-loa = LOA { $length }
draw-draught = DRAUGHT { $draught }
draw-engines = ENGINES ({ $count })
draw-beam = BEAM { $beam }
draw-view-tug-ga = { $tug } GA
draw-note-dimensions = { $view }. DIMENSIONS IN METRES.
draw-note-tug = { $tug } { $length } X { $beam } X { $draught } M; { $count } X { $propulsor }, DIAMETER { $diameter } M.
draw-note-tug-positions = PROPELLERS, RUDDERS AND PUSH KNEES ARE DRAWN TO SHOW THE ARRANGEMENT, NOT CALCULATED.

## crates/ovd-format/src/ga.rs

draw-double-bottom = DOUBLE BOTTOM
draw-wbt-both-sides = WBT (BOTH SIDES)
draw-weather-shield = WEATHER SHIELD
draw-dome = DOME
draw-cofferdam = COFFERDAM
draw-service-space = SERVICE SPACE
draw-fwd-separation = FWD SEPARATION
draw-fwd-equip = FWD EQUIP.
draw-holds = HOLDS ({ $count })
draw-tank = TANK { $number }
draw-slop-tank = SLOP TANK
draw-pump-room = PUMP ROOM
draw-cargo-tank-pair = CARGO TANKS (P AND S)
draw-cargo-tank = CARGO TANK
draw-centreline-bulkhead = CENTRELINE BULKHEAD
draw-cargo-heating = CARGO HEATING FITTED
draw-thrusters = THRUSTERS ({ $count })
draw-wheelhouse-block-top = WHEELHOUSE, BLOCK TOP { $height }
draw-containers-block-top = CONTAINERS ({ $abreast } x { $rows }), BLOCK TOP { $height }
draw-containers = CONTAINERS ({ $abreast } x { $rows })
draw-air-draft = AIR DRAFT { $draught }
draw-depth = DEPTH { $depth }
draw-freeboard = FREEBOARD { $freeboard }
draw-light-draught = LIGHT DRAUGHT { $draught }
draw-ballast-draught = BALLAST DRAUGHT { $draught }
draw-void-both-sides = VOID (BOTH SIDES)
draw-cradle = CRADLE
draw-inner-bottom = INNER BOTTOM { $height }
draw-coaming = COAMING { $height }
draw-side-clearance = SIDE CLEARANCE { $clearance }
draw-bottom-clearance = BOTTOM CLEARANCE { $clearance }
draw-tank-bottom = TANK BOTTOM { $height } (CLEARANCE { $clearance })
draw-fails-one = DESIGN FAILS { $count } CHECK: { $checks }
draw-fails = DESIGN FAILS { $count } CHECKS: { $checks }
draw-fails-more = DESIGN FAILS { $count } CHECKS: { $checks } AND { $more } MORE
draw-fails-one-count = DESIGN FAILS { $count } CHECK
draw-fails-count = DESIGN FAILS { $count } CHECKS
draw-owner = OPEN VESSEL CONCEPT DESIGN
draw-note-rule-sets = RULE SETS: { $sets }.
draw-note-rule-sets-count = { $count } RULE SETS.
draw-view-side-elevation = SIDE ELEVATION
draw-view-plan = PLAN
draw-view-midship-section = TYPICAL SECTION

## otd/src/furniture.rs

draw-status-draft = DRAFT – NOT FOR CONSTRUCTION
draw-status-concept = CONCEPT – NOT FOR CONSTRUCTION
draw-earlier-revisions = { $count } EARLIER REVISIONS NOT SHOWN
draw-scale-as-shown = AS SHOWN
draw-heading-notes = NOTES
draw-heading-references = REFERENCES
