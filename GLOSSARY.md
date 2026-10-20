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
such as block coefficient and rake lengths.
_Avoid_: hull type, hull shape

**Formation**:
An arrangement of Hulls that moves as one — a pushed convoy, a tow, a coupled unit, an ATB — described
by its Hulls, its Tug and how they are placed (abreast, in line). It has no General Arrangement of its own, only an
arrangement plan.
_Avoid_: convoy (when the general term is meant), fleet

**Waterway Limit**:
A dimension a waterway imposes on any Formation using it, such as a lock chamber's length and width.
It belongs to the waterway, not to a vessel.
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

**Design**:
One study saved as a single file: one or more Options, compared side by side.
_Avoid_: project, model

**Option**:
One complete set of inputs within a Design, giving one set of results.
_Avoid_: variant, scenario, case

**Moulded Depth**:
Keel to main deck at the side (not hold depth). Left out, it is the depth-to-beam ratio of the Cargo
times the beam, a rule in the Defaults Profile.
_Avoid_: hold depth, draught (a result, not an input)

**Defaults Profile**:
The named set of default input values and allowed input ranges used when an Option leaves an input
out. Each range is hard (the input is refused) or soft (the input is accepted with a warning). A
default may depend on another input (the Cargo Containment, the Cargo, the LPG Carriage Condition); each
value of such a table says whether it is sourced from a published tank or is an estimate, and why.
_Avoid_: settings, config (on its own)

**Moulded Depth**:
The depth of a Hull from the top of the keel to the main deck at the side, at mid-length. Not the hold
depth, which is less by the double bottom.
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
Rule or a Rule Set applies. A Rule that does not apply is listed as not applicable, never dropped.
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
governs, and no Rule loosens another.
_Avoid_: controlling rule, binding rule

**OVCD Check**:
A Rule that OVCD makes but no published rule book requires: physical feasibility (the draught is below
the depth), a method's validity range, or a limit of the tool. Shown in the Rule Trace with its origin,
never as a published Rule.
_Avoid_: design rule

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
