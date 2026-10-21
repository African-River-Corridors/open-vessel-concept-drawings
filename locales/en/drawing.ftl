# The engine's messages: drawing labels and notes (English only for now: the Drawing Format
# Standard allows ASCII only) (spec 0005, ADR 0015).
# Written by tools/i18n_extract.py from the Rust source. Do not edit: change the English in the
# `text!` call and run the tool again. A number argument arrives as text, already rounded.


## crates/ovcd-draw/src/furniture.rs

draw-status-draft = DRAFT – NOT FOR CONSTRUCTION
draw-status-concept = CONCEPT – NOT FOR CONSTRUCTION
draw-earlier-revisions = { $count } EARLIER REVISIONS NOT SHOWN
draw-scale-as-shown = AS SHOWN
draw-heading-notes = NOTES
draw-heading-references = REFERENCES

## crates/ovcd-format/src/ga.rs

draw-double-bottom = DOUBLE BOTTOM
draw-wbt-both-sides = WBT (BOTH SIDES)
draw-dome = DOME
draw-cofferdam = COFFERDAM
draw-fwd-equip = FWD EQUIP.
draw-thrusters = THRUSTERS ({ $count })
draw-wheelhouse-block-top = WHEELHOUSE, BLOCK TOP { $height }
draw-containers-block-top = CONTAINERS ({ $abreast } x { $rows }), BLOCK TOP { $height }
draw-wheelhouse = WHEELHOUSE
draw-containers = CONTAINERS ({ $abreast } x { $rows })
draw-loa = LOA { $length }
draw-depth = DEPTH { $depth }
draw-freeboard = FREEBOARD { $freeboard }
draw-draught = DRAUGHT { $draught }
draw-light-draught = LIGHT DRAUGHT { $draught }
draw-beam = BEAM { $beam }
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
draw-note-dimensions = { $view }. DIMENSIONS IN METRES.
draw-view-side-elevation = SIDE ELEVATION
draw-view-plan = PLAN
draw-view-midship-section = MIDSHIP SECTION
