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
An arrangement of Hulls that moves as one — a pushed convoy, a tow, a coupled unit — described by its
Hulls and how they are placed (abreast, in line). It has no General Arrangement of its own, only an
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

**Design**:
One study saved as a single file: one or more Options, compared side by side.
_Avoid_: project, model

**Option**:
One complete set of inputs within a Design, giving one set of results.
_Avoid_: variant, scenario, case

**Defaults Profile**:
The named set of default input values and allowed input ranges used when an Option leaves an input
out. Each range is hard (the input is refused) or soft (the input is accepted with a warning).
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
