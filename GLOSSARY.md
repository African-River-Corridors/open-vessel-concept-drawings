# vessel_designer

Concept design of inland and coastal cargo vessels for feasibility studies: from a set of inputs, produce
concept General Arrangements, characteristics, tonnage, power and cost — checked against real vessels.

## Language

### Catalogue

**Vessel Class**:
A standard size envelope from a Class Scheme or a recognised size band (e.g. CEMT Va, US jumbo hopper).
It sets maximum length and beam, and where the scheme gives them, draught, air draft and deadweight. A
vessel sits in the smallest class of its scheme whose length and beam maxima it fits.
_Avoid_: type, category, standard (on its own)

**Class Scheme**:
A published family of Vessel Classes ordered by size (e.g. CEMT, GB 38030). A vessel is classed within
one scheme at a time; growing or shrinking moves it between classes of that scheme, never to another
scheme.
_Avoid_: standard (on its own), class system

**Reference Vessel**:
A real, built vessel with published particulars, filed under one or more Vessel Classes.
_Avoid_: example ship, benchmark

**Hull**:
A Vessel Class that is a single floating body and so gets a General Arrangement of its own
(motor vessel, push barge, pusher, towboat).
_Avoid_: ship type

**Hull Form**:
The shape family of a Hull (e.g. flat-bottom raked box, V-bottom) and the parameters that set it,
such as rake lengths, bilge radius or deadrise. The block coefficient is a result of the shape, not a
parameter.
_Avoid_: hull type, hull shape

**Sections Table**:
The master hull geometry (ADR 0012): closed section polygons at Stations along the length. A Hull
Form's Generator writes it; the Offsets Hull Form is one typed in.
_Avoid_: lines plan, offsets (when the derived table is meant)

**Station**:
A position x along the hull where the Sections Table holds a section.

**Generator**:
The code of a Hull Form that writes a Sections Table from the Hull Form's parameters.

**Vessel Model**:
The hull, arrangement and structure layers of a Hull (ADR 0018), from which every analysis and every
view reads its geometry.
_Avoid_: digital twin, 3D model (when the engineering model is meant)

**Waterplane**:
The plane of the water surface relative to the hull, set by the draught, the heel and the trim.

**Formation**:
An arrangement of Hulls that moves as one — a pushed convoy, a tow, a coupled unit, an ATB — described
by its Hulls, its Tug and how they are placed (abreast, in line). It has no General Arrangement of its own, only an
arrangement plan.
_Avoid_: convoy (when the general term is meant), fleet

**Waterway Limit**:
A dimension a waterway imposes on any Formation using it: a lock chamber's length and width, a
largest Draught, a largest Air Draft. It belongs to the waterway, not to a vessel.
_Avoid_: lock class

### Design

**Cargo**:
What the vessel carries (e.g. LNG, bauxite, containers). An input to every design, never a property of a
Vessel Class.

**Cargo Containment**:
How a Hull holds its Cargo: a box hold, a sloped hopper hold, or IMO Type C tanks.
_Avoid_: vessel type, barge type

**LPG Carriage Condition**:
How an LPG Cargo is carried in Type C tanks: fully pressurised (ambient temperature, the vapour
pressure of the cargo), semi-pressurised or semi-refrigerated (cooled, at a middle pressure), or fully
refrigerated or cryogenic (cooled, near atmospheric pressure). An input for LPG Cargos only. The tank
design defaults follow the Cargo Containment, the Cargo and this condition.
_Avoid_: LPG type, LPG mode

**Allowable Stress Basis**:
Where the allowable stress of a tank steel comes from when no stress is typed. The IGC basis is the
limit of IGC 4.23.3.1 for Type C tanks: the lesser of Rm/A and Re/B, with the steel's specified minimum
tensile strength Rm and yield stress Re, and A and B by steel class (3 and 1.5 for nickel and
carbon-manganese steels, 3.5 and 1.5 for austenitic steels). The old basis keeps the stress a file was
saved with (S690QL1 at 285 MPa, an inland basis above the IGC limit).
_Avoid_: safety factor (on its own), design stress

**Filling Limit**:
The largest share of a cargo tank's volume that liquid may fill when the cargo has reached the
reference temperature (IGC 15.1.1). 98 % by default (IGC 15.3). The fill ratio input holds it.
_Avoid_: fill level, ullage

**Loading Limit**:
The largest share of a cargo tank's volume to which it may be loaded (IGC 15.1.2): the Filling Limit
times the cargo density at the reference temperature over the density at the loading temperature
(IGC 15.5.1). The reference temperature is the one at which the vapour pressure equals the relief
valve setting when the cargo has no temperature control, and the loading temperature when it has. An
LPG Cargo is loaded to it; it is never above the Filling Limit.
_Avoid_: fill ratio (it is the Filling Limit), loading level

**Volume-bound Cargo**:
Cargo limited by the space that holds it, before the Draught limit is reached.
_Avoid_: cubed out

**Weight-bound Cargo**:
Cargo limited by the Draught (the displacement left), before the space is full.
_Avoid_: draft-bound, deadweighted out

**Stowage Factor**:
The volume one tonne of a dry cargo takes in a hold, in m³ per t.
_Avoid_: density (its inverse, for liquids)

**Dome Rule**:
The default size of a Type C tank's dome: diameter the larger of 1.2 m and 0.17 times the steel outer
diameter, height above the tank equal to the diameter. An estimate on two published tanks. The dome
counts in the tank's gross volume Vc (IGC 2.4.1.1) and raises the tank top and the air draft. A typed
diameter or height replaces it.
_Avoid_: dome height default, standard dome

**Tank Outer Diameter**:
The diameter of a Type C tank measured to the outside of its insulation. The steel outer diameter is
that less twice the insulation thickness. The side and bottom clearances, the tank top and the drawing
use the insulated face; the wall thickness and the volume start from the steel diameter.
_Avoid_: tank diameter (on its own), OD (on its own)

**Steel Outer Diameter**:
The outside diameter of a cargo tank's steel shell, without its insulation. Less than the Tank Outer
Diameter by twice the insulation thickness.
_Avoid_: shell diameter, OD (when the insulated one could be meant)

**Tank Head**:
The closed end of a cylindrical cargo tank: hemispherical, 2:1 ellipsoidal or torispherical.
_Avoid_: end cap, dished end (on its own)

**Design Vapour Pressure**:
The highest gauge pressure in the vapour space of a cargo tank that the tank is designed for (P0).
_Avoid_: tank pressure, MARVS (a different limit)

**Tank Structure Factor**:
A multiplier on a cargo tank's shell mass that stands for its stiffeners, ring frames and supports.
_Avoid_: structure allowance

**Design**:
One study saved as a single file: one or more Options, compared side by side.
_Avoid_: project, model

**Option**:
One complete set of inputs within a Design, giving one set of results.
_Avoid_: variant, scenario, case

**Defaults Profile**:
The named set of default input values and allowed input ranges used when an Option leaves an input
out. Each range is hard (the input is refused) or soft (the input is accepted with a warning). A
default may depend on another input (the Cargo Containment, the Cargo, the LPG Carriage Condition); each
value of such a table says whether it is sourced from a published tank or is an estimate, and why.
_Avoid_: settings, config (on its own)

**Moulded Depth**:
The depth of a Hull from the top of the keel to the main deck at the side, at mid-length. Not the hold
depth, which is less by the double bottom, and not the Draught.
_Avoid_: depth (on its own), hold depth, hull height

**Calibration**:
Running the design calculations from a Reference Vessel's inputs and comparing the results with its
published particulars, to measure and correct the calculation logic.
_Avoid_: validation, tuning

**Calibration Band**:
The accepted range around a Reference Vessel's published figure (e.g. lightship ±20 %). A Calibration
result outside its band is flagged with a warning, then either accepted with a reason or logged as a bug.
_Avoid_: tolerance, error margin

**General Arrangement (GA)**:
A concept drawing set for one Hull — plan, elevation and cross-section — for feasibility studies, not
for construction or class approval.
_Avoid_: layout, drawing (on its own)

### Hull and stability

**Length Overall (LOA)**:
The length of a Hull from its aftmost to its foremost point.
_Avoid_: length (on its own)

**Length of Waterline**:
The length of a Hull at the waterline of a given Draught.
_Avoid_: LWL alone in prose, waterline length

**Draught**:
The depth of the Hull below the waterline, from the keel. A result of a Loading Condition, not an input.
_Avoid_: draft (in prose), depth

**Freeboard**:
The height of the main deck at the side above the waterline: Moulded Depth less Draught.
_Avoid_: dry height

**Minimum Freeboard**:
The least Freeboard a Design accepts; it caps the Draught a Hull may load to.
_Avoid_: freeboard margin

**Block Coefficient (Cb)**:
The displaced volume over the box of Length of Waterline × beam × Draught that holds it.
_Avoid_: fullness, Cb without its length basis

**Lightship**:
The mass of the vessel ready for service, without cargo, consumables, stores or water ballast.
_Avoid_: light weight, empty weight

**Deadweight**:
What a vessel carries on top of its Lightship: cargo, consumables and stores, and water ballast.
_Avoid_: payload (use cargo for the cargo part), DWT in prose

**Consumables**:
The fuel, fresh water and stores a vessel carries for a voyage; part of the Deadweight, never the Lightship.
_Avoid_: provisions, stores (for the whole)

**Loading Condition**:
One state of loading checked for stability: a cargo, consumables and water ballast at stated amounts,
for example full load departure or ballast arrival.
_Avoid_: load case, governing condition

**Damage Case**:
One assumed damage of a liquid bulk hull (a side or a bottom damage of stated extent at a stated place along the
hull) and the compartments it floods, checked on the laden departure Loading Condition: the floating position of
the damaged hull, then its righting lever. The result is a verdict for one Rule object.
_Avoid_: damage scenario, damage condition (a Loading Condition is intact)

**Water Ballast**:
Water taken into ballast tanks to bring a vessel to a safe Draught, trim or stability in a Loading Condition.
_Avoid_: ballast (when the solid kind could be meant)

**GZ Curve**:
The righting lever of a vessel against heel angle, in one Loading Condition.
_Avoid_: stability curve, righting arm curve

**Free Surface Correction**:
The loss of GM from liquid that can move in a part-filled tank, added to the height of the centre of gravity.
_Avoid_: slack tank effect, FSE

**Down-flooding Angle**:
The heel angle at which water first enters the hull through an opening that cannot be closed weathertight.
_Avoid_: flooding angle

### Gas cargo arrangement

**Cargo Zone**:
The length of a Hull between its end spaces that holds the cargo tanks or holds.
_Avoid_: cargo area, tank zone

**End Space**:
A space at one end of the Cargo Zone: the Stern Block aft, a bow void forward, and the Cofferdams beside them.
_Avoid_: end void (for the whole)

**Cofferdam**:
An empty space between two other spaces, there to keep them apart.
_Avoid_: void (when a cofferdam is meant), separation space

**Protective Location**:
The least distance a gas cargo tank must keep from the hull's side shell and bottom, which grows with the tank's volume.
_Avoid_: protective distance, clearance rule

**Side Clearance**:
The clear distance from the moulded side of the Hull to the outer face of a cargo tank's insulation; the
bottom clearance is the same, measured down to the inner bottom.
_Avoid_: clear gap, side gap

**Wing Ballast Tank**:
A ballast tank along the side of the Cargo Zone, between the cargo tanks and the side shell.
_Avoid_: side tank, wing tank (without "ballast")

**Saddle**:
The seat that carries a cylindrical cargo tank on the Hull's bottom structure.
_Avoid_: cradle (in prose; a drawing may say cradle), stool

**Candidate**:
One arrangement of cargo tanks the arrangement search builds and judges on a Hull.
_Avoid_: trial, variant

### Waterway and powering

**Environment**:
One waterway section, with its depth and width, in which resistance, squat and speed are worked out.
_Avoid_: Service Area (a different thing), scenario

**Air Draft**:
The height of the highest fixed point of a vessel above the waterline. The light Air Draft is the largest.
_Avoid_: air clearance, governing air draft

**Under-Keel Clearance**:
The water depth under the keel: static at rest, dynamic with Squat at speed.
_Avoid_: UKC in prose, keel clearance

**Squat**:
The sinkage of a moving vessel in shallow or confined water, which reduces the Under-Keel Clearance.
_Avoid_: sinkage (on its own)

**Blockage Ratio**:
The vessel's midship section area over the waterway's cross-section area.
_Avoid_: blockage (on its own), m (in prose)

**Limiting Speed**:
The highest speed a vessel can reach in a confined waterway by displacement, from the Blockage Ratio and the water depth.
_Avoid_: critical speed (in prose), Schijf speed

**Service Speed**:
The speed a vessel holds in an Environment, as the power, the Limiting Speed, Squat or the speed range allow.
_Avoid_: cruising speed, operating speed

**Journey De-rating**:
The cut in cargo so that the loaded Draught fits the shallowest point of a journey.
_Avoid_: light loading, derating (unhyphenated)

### Cost

**Capital Budget**:
The estimated cost of building one Unit, from the masses of the Design and the cost rates.
_Avoid_: price, quotation, build cost

**Bill of Quantities (BOQ)**:
The list of what a Capital Budget prices, each line a quantity and its rate.
_Avoid_: cost breakdown, takeoff

**Cost Rate**:
The price of one unit of a quantity, such as hull steel per tonne, an input of the Capital Budget.
_Avoid_: unit cost, price

**Lump Sum**:
A fixed amount in the Capital Budget that does not scale with the Design.
_Avoid_: allowance, provisional sum

**Yard Margin**:
A fraction of the subtotal added for engineering, class, trials and the yard's margin.
_Avoid_: markup, contingency

**Unit**:
What one Capital Budget prices: one Hull, with its Tug where its Formation has one.
_Avoid_: ATB unit (unless it is one), set

### Drawings

**Side Elevation**:
The view of a Hull from the side, along its length.
_Avoid_: profile (on its own), side view

**Plan**:
The view of a Hull from above.
_Avoid_: top view, deck plan (unless only the deck is shown)

**Midship Section**:
The cross-section of a Hull at mid-length, or a typical section of the Cargo Zone when so labelled.
_Avoid_: cross-section (on its own), section (on its own)

**Light Waterline**:
The waterline of the vessel at its Lightship with no cargo and no ballast.
_Avoid_: lightship line

### Evidence

**Calibration Case**:
One Reference Vessel run through the calculations and compared with its published particulars.
_Avoid_: test case, benchmark

**Held-out Case**:
A Calibration Case kept out of the fit of a factor, so that it tests the factor; an in-sample case was used to fit it.
_Avoid_: test point, validation point

**Evaluation Harness**:
The set of cases with known pass or fail verdicts that checks the Rule results.
_Avoid_: test suite (on its own), validation set

**False Pass**:
A case the Evaluation Harness expects to fail that OVCD passes. The build allows none.
_Avoid_: missed failure

**Design-impact Check**:
The comparison of a fixed set of Designs run before and after a change, which lists every result that moves.
_Avoid_: regression test, golden test

**Results Hash**:
The fingerprint of an Option's results, saved in its file, so a later run shows whether they moved.
_Avoid_: checksum (on its own)

### Stern, propulsion and tugs

**Stern Block**:
The aft part of a self-propelled Hull or a Tug that holds its propulsion, machinery, accommodation and
wheelhouse. Its length comes off the cargo zone.
_Avoid_: aft block, aft arrangement (in user text), engine block

**Stern Form**:
The shape family of the aft body of a Hull (transom, pram, tunnel, or a barge stern with a Notch) and
the few parameters that set it, such as the run length and the transom height.
_Avoid_: stern type, stern shape, aft body (on its own)

**Propulsor**:
The device that turns power into thrust: a propeller on a shaft (open or in a nozzle), an azimuth
thruster (open or ducted), a Voith Schneider unit or a pump jet. A Hull has one type of Propulsor, one
or more of them.
_Avoid_: thruster (for every type), drive, screw (on its own)

**Power Source**:
What makes the power the Propulsors use: diesel-mechanical, diesel-electric, battery-electric,
battery-hybrid or gas-fuelled generating sets.
_Avoid_: propulsion (on its own), engine type, fuel type

**Installed Power**:
The total rated power of the prime movers (or, for battery-electric, the drive motors) that drive the
Propulsors, in kW.
_Avoid_: engine power (on its own), MCR (in user text), rated power

**Design Speed**:
The speed through still deep water at the laden draught that the Installed Power is sized for.
_Avoid_: service speed (that is a result per waterway), trial speed

**Bollard Pull**:
The thrust a Hull gives at zero speed with its full Installed Power, in tonnes.
_Avoid_: static thrust, towing force

**Tug**:
The Hull that propels a Formation: a pusher (pushed barges and convoys), an ATB tug, or a towing tug
(barges on a hawser).
_Avoid_: towboat (except as the US name for a pusher), push boat, tractor

**Articulated Tug-Barge (ATB)**:
A Formation of one barge and one Tug whose bow sits in a Notch in the barge stern, joined by a coupler
that lets the Tug pitch relative to the barge.
_Avoid_: ITB (that is a rigid connection), coupled unit (that is a motor vessel and a pushed barge)

**Notch**:
The cut in the stern of an ATB barge that the Tug's bow sits in, set by its depth and width.
_Avoid_: slot, recess, dock

**Equivalent Hull**:
One hull with the overall length, beam and draught of a Formation, used to estimate the Formation's
resistance.
_Avoid_: combined hull, convoy hull

**Blind Distance**:
How far ahead of the bow (of the Formation's lead craft, for a pusher) the water cannot be seen from the
steering position, in a stated loading condition.
_Avoid_: blind zone (on its own), dead zone, obstructed view

**Fixed Height**:
The height from the waterline to the highest point of a Hull with the wheelhouse and every other mobile
part lowered. It sets the air draft.
_Avoid_: air draft (that is the waterway's limit compared with it), clearance height

### Rules

**Rule**:
One requirement about one Measure: its Criterion, the objects and Options it applies to, what happens
if it is not met, and where it comes from. A Rule from a published rule book is sourced only when it is
verified; an OVCD Check is a Rule with OVCD's own origin.
_Avoid_: check (on its own), regulation

**Criterion**:
The test part of a Rule: a comparison (at least, more than, at most, less than) and a limit, which is a
constant, a table value or a formula of the Design's inputs. Two Criteria in the same direction can
always be ranked: one is at least as strict as the other.
_Avoid_: rule (for the test part only), condition

**Requirement**:
The Rules that test the same Measure on the same object in the same direction, from any selected Rule
Set. The Design meets a Requirement only if it meets every Rule in it.
_Avoid_: constraint, clause

**Applicability**:
The condition on an Option's inputs (Cargo, Cargo Containment, Service Area, and similar) under which a
Rule or a Rule Set applies. It may also name a derived input the engine works out: the Cargo Class or
the deadweight band. A Rule that does not apply is listed as not applicable, never dropped.
_Avoid_: scope, filter

**Blocking Rule**:
A Rule whose failure makes the Option fail. Other Rules only warn or inform.
_Avoid_: hard rule, error check

**Verified**:
The status of a Rule that a second reader, not the one who wrote it, has checked against the text of
the source edition it cites, at the paragraph it cites. Every other Rule is unverified.
_Avoid_: checked, approved, validated

**Rule Set**:
A named, versioned group of Rules from one source edition (e.g. the IS Code 2008 Part A, or the IGC
Code as amended for ship type 2G). A Design selects the Rule Sets it is checked against. Its version is
OVCD's encoding of the edition, so one edition can have several versions; every released version is
kept.
_Avoid_: ruleset (one word), criteria set, code (on its own)

**Measure**:
A quantity the engine computes from a Design for one object, such as the GZ area from 0° to 30° of a
loading condition or the clear side gap of a tank. A design input the engine uses, such as the weld
factor, is also a Measure when a Rule bounds it. Rules compare Measures against limits.
_Avoid_: metric, parameter

**Service Area**:
Where an Option is meant to operate: river only, or coastal and river. It sets the default Rule Sets of
the Option.
_Avoid_: route, zone, trade

**Rule Trace**:
The list, in every result, of each Rule of each selected Rule Set: its citation, the inputs used, the
value, the limit, the Margin, the outcome and the verification status. A Rule the engine cannot evaluate
yet is listed as not evaluated, never left out.
_Avoid_: log, audit (on its own)

**Margin**:
How far a value is from its Rule's limit, signed so that a positive Margin passes.
_Avoid_: tolerance, slack

**Governing Rule**:
Of the Rules in one Requirement, the one with the strictest Criterion. All of them apply; the strictest
governs, and no Rule loosens another. "Governing" is kept for this sense only.
_Avoid_: controlling rule, binding rule; "governing condition" (say lowest-GM Loading Condition);
"governing air draft" (say light Air Draft)

The result key that names the lowest-GM Loading Condition is `stability.lowest_gm_condition` (it was
`stability.governing`, renamed in #534). The check "Air draft (empty, governing)" was removed in #421; the light
Air Draft is checked by `air draft within limit`.

**Rules Kernel**:
The domain-free part of the rules engine, the `arc-rules` crate: the Rule Set format and loader, the
content hash, Applicability and the evaluator. It knows no vessel. OVCD declares its object types,
Measures, inputs and formula shapes to it; another tool can declare its own.
_Avoid_: rules library, rule core

**Object-Scoped Input**:
An input with one value per object, not one per Design (for a channel, the kind of each reach). An
Applicability condition on it is read for each object a Rule tests. Every OVCD input is one per Option.
_Avoid_: local input, per-object flag

**OVCD Check**:
A Rule that OVCD makes but no published rule book requires: physical feasibility (the draught is below
the depth), a method's validity range, or a limit of the tool. Shown in the Rule Trace with its origin,
never as a published Rule.
_Avoid_: design rule

**Origin**:
Where a Rule comes from: a regulation, a class rule, a physical limit, a limit of the tool, or the
validity range of a method. It tells compliance apart from engineering judgement.
_Avoid_: source (which is the citation), type

**Consequence**:
What a failed Rule does: block (the Option does not pass), warn (shown, not blocking) or note (a value
reported with its citation).
_Avoid_: severity (in prose about Rules), level

**Sizing Rule**:
A Rule whose limit sets a dimension of the design, such as a wall thickness, not only checks it.
_Avoid_: design rule

**Not Evaluated**:
The outcome of a Rule the engine cannot check yet. It is never a pass.
_Avoid_: skipped, n/a, unknown

**Set Version**:
The version number of one Rule Set's content, changed with every change of a Rule. Not the Edition.
_Avoid_: release (of a Rule Set)

**Edition**:
The edition of the published document a Rule cites, such as a year or an amendment.
_Avoid_: version (of a regulation)

**Selection Order**:
The order of the Rule Sets an Option selects; on equal strictness the first in this order governs.
_Avoid_: priority

### Languages

**Display Language**:
The language a user reads OVCD in. It is a choice of the person, never of the Design: it changes no
input, no result and no saved file.
_Avoid_: locale (in user text), Design language

**Text Library**:
The set of message files, one folder per language, that holds every string a user reads. English is
the source; each other language holds translations of English messages.
_Avoid_: catalogue (that is the vessel Catalogue), string table, dictionary

**Message Id**:
The stable name of one message in the Text Library, with named arguments for its numbers and texts.
The engine gives a Message Id beside its English; the app formats it in the Display Language.
_Avoid_: key, string id, translation key

**Translation Status**:
The state of one translated message: draft (written by a person or a machine, not checked) or
reviewed (checked by a named native speaker against the English it was written for).
_Avoid_: approved, verified (those words are for engineering results)

**Term List**:
The controlled list of naval-architecture terms and the one word each language uses for each. Where a
language has no established term, the English term is kept and marked for a native reviewer.
_Avoid_: glossary (that is this file), vocabulary
