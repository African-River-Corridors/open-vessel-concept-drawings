# Stern, propulsion and tug are design choices on a parametric stern

Today a self-propelled hull gets one stern block: azimuth thrusters, deck containers and a lifting
wheelhouse. Its size comes from layout values, never from power, and it has no mass. One propulsive
efficiency (0.45) serves every vessel. A pushed convoy has no results, and a tug exists only as three
drawing numbers. The maintainer wants real choices in the stern and the tug, and stern blocks that look
and weigh like built ones, in 0.2.x, for self-propelled vessels, pushers and ATBs alike.

## Decision

- **Four choices describe the aft end.** **Stern Form** (transom, pram, tunnel; and the ATB notch on a
  barge), **Propulsor** (azimuth thruster open or ducted, shaft with open propeller or in a nozzle, Voith
  Schneider, pump jet), **Power Source** (battery-electric, gas-fuelled gensets, diesel-mechanical,
  diesel-electric, battery-hybrid) and, for a Formation, the **Tug** that drives it (pusher, ATB tug,
  towing tug). Each is an input with parameters. The engine derives the stern block, the machinery, its
  mass and centres, and the power from them.
- **The stern is parametric now.** A Stern Form is a few numbers (run length, transom height, tunnel
  crown, notch depth and width). The core corrects displacement, LCB and wetted surface in closed form
  and the drawings show the shape. When the sections table of ADR 0012 lands, its generator writes
  sections from the same parameters. There is no second input set and no second format bump: the new
  inputs go into `.vdesign` format version 3, which #274 and the ADR 0012 hull block share.
- **Power comes from a propulsor model.** An actuator disc gives the ideal efficiency from the thrust
  loading of the propeller disc, using the diameter the stern allows. One factor per Propulsor type,
  calibrated on published bollard pull per kW, gives the losses. Bollard pull comes from the same model.
  The fixed propulsive efficiency stays only for files that name no Propulsor.
- **A tug is a Hull of its own.** OVCD sizes it from the power its Formation needs, with published
  ratios; a Reference Vessel tug can be picked as a preset; every value can be typed. It has its own
  stern block, machinery and mass.
- **A Formation is powered as one equivalent hull.** The convoy, tow or ATB runs through the existing
  resistance method as one hull of its overall length, beam and draught, with allowances, marked
  `unverified` and calibrated on published pushers and ATBs. A shallow-water convoy method (Zhang et al.,
  2023) is a later, separate change.
- **The stern block has mass and centres.** Each item (engines, drives, gensets, batteries,
  accommodation, wheelhouse, lift column) is a lightship item with a mass, an LCG and a KG. Results gain
  trim and aft and forward draughts.
- **Old files do not move.** A file that lacks the new inputs is read with values that reproduce
  today's arrangement exactly (transom with no rake, the fixed efficiency, battery-electric, no stern
  mass). A new Option takes its defaults by Cargo: gas Cargo gets gas-fuelled gensets and ducted azimuth
  thrusters; dry bulk gets diesel-mechanical twin shafts in nozzles under a tunnel stern.

## Why

- Users choose the stern and the drive early; a concept tool that offers one arrangement cannot compare
  them.
- Parametric now and sections later gives realistic numbers and drawings in 0.2.x without waiting for
  ADR 0012, and without a second set of inputs to migrate.
- The actuator disc is the simplest model in which a smaller propeller (shallow draught) costs
  efficiency and a larger one gives more pull, as the published tugs show. A fixed efficiency per
  type cannot show this; open-water charts need wake data that inland ships do not publish.
- A sized tug with presets covers every Formation, where a library alone leaves gaps between entries.

## Consequences

- The `.vdesign` input table grows (spec 0007); the inputs land in format version 3.
- `aft_arrangement` becomes the layout of the deck and accommodation items. The machinery, the
  propulsor and the stern geometry move into their own modules, each with a small interface.
- New Rules enter the rules engine (spec 0004): ES-TRIN 2025/1 wheelhouse visibility, forward speed
  and propulsion redundancy; SOLAS V/22 visibility for the coastal-and-river Service Area; OVCD Checks
  for propeller fit, immersion and power.
- Calibration gains propulsion cases: bollard pull of tugs and pushers, installed power of inland
  ships, and convoy and ATB power.
- Every value without a published source is marked `estimate` in the Defaults Profile, with its basis.
