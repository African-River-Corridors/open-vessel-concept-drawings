# The `.vdesign` file format

A Design is saved as one file with the extension `.vdesign`.

## Decisions

- Versioned JSON, in SI units **except speeds, which are in knots** (the inland-waterway convention):
  `max_speed_kn` and `channel_current_kn`. The `units` field holds `"SI"` and means this. Nothing else
  is non-SI. Storing speeds in m/s would be a format change (a new `format_version` with a
  migration); it is not planned. Unit switching is only for display and input, in the app.
- A Design holds a named list of one or more Options, the engine version, and a results hash for each
  Option.
- An Option holds the typed `simulate` inputs, using glossary names, plus `hull_form` and `formation`.
- Required inputs: Cargo, Hull Form, length, beam and Formation. Every other input comes from the
  Defaults Profile. Moulded depth (keel to main deck at the side) is one of them: left out, it is the
  depth-to-beam ratio of the Cargo times the beam, a rule in the Defaults Profile (issue #120), stored
  as used and marked `default` like any other.
- The file stores every input **as used**, and marks which ones came from a default. A later change to
  the Defaults Profile never changes the results of a saved file.
- Each validation range is hard (the input is refused) or soft (the input is accepted with a warning).
  Validation lives only in `ovcd-format`. The core adds no checks.
- The results hash is the SHA-256 of the canonical JSON of an Option's results. If a file opens on an
  engine version that gives a different hash, the tool warns that the results differ from the hash
  saved in the file. It cannot tell an engine change from a hand edit, so it does not blame either.
  The top-level `drawing` object of the results holds drawing geometry from the core (double
  bottom, wing tanks, separation, aft block). It is not a result, and the hash leaves it out.

## Format versions

`format_version` is an integer. A tool reads every version from `OLDEST_FORMAT_VERSION` to
`FORMAT_VERSION` (`crates/ovcd-format/src/vdesign.rs`) and writes the newest. An older file is
changed to the current inputs as it is read, and each change is a warning (in `ovcd validate`,
`ovcd fill`, `ovcd run`, the Design session and the app). The file on disk is not touched; the next
save writes the current version.

- **1**: the first version.
- **2** (issue #175): the input table changed. `max_tank_od_m` is new. `adn_tank_cap_m3` and the
  `ADN_G` ruleset are gone, because the engine applies IGC rules only to gas tanks. A version 1 file
  that names `ADN_G` is read as `IGC_2G`, its `adn_tank_cap_m3` is dropped, and a gas Option with no
  `max_tank_od_m` gets the Defaults Profile value (16 m), marked `default`, with a warning that
  version 1 used a fixed 12 m cap, so results may change.
  Issue #237 (the one gas rule set) also reuses version 2, as version 2 is unreleased: `IGC_2G` is now
  the per-tank d(Vc) of IGC 2.4.1.1 (measured plate to plate, 2.4.2, dome included in Vc), and
  `IGC_2G_EXACT` is no longer offered. A file of any version that names `IGC_2G_EXACT` is read as
  `IGC_2G`, with the warning "IGC_2G_EXACT is now IGC_2G: the same per-tank rule of IGC 2.4.1.1". A
  version 1 file that names `IGC_2G` meant the old stepped reading, so it is read as `IGC_2G` with a
  warning that the per-tank rule applies now and results may change (a version 2 file needs no such
  note: this build wrote it). The version 1 `ADN_G` mapping to `IGC_2G` is kept. A file that lacks the
  new input `tank_dome_diameter_m` (2.0 m, marked `default`) gets a warning that results may change,
  not the "results are unchanged" note of the other inputs added after the first files, because the
  engine had no dome in Vc before.
  The Vessel Class change (issue #94) also changes the input table before the next release, so it
  reuses version 2 and does not bump again: a third source marker `class` (alongside `typed` and
  `default`) marks an input that a Vessel Class gave, and two inputs, `vessel_scheme` and `vessel_class`,
  are held only by an Option defined by a standard class (an Option with typed dimensions has neither,
  so a version 1 file needs no change). The class is derived: after every read it is the smallest class
  of the scheme whose length and beam maxima hold the vessel, so length and beam decide it; a stored
  class that disagrees is replaced, with a warning. Picking a class sets length, beam and Formation
  (and a convoy's counts), marked `class`; its draught (top of the range) and air draft (bottom of the
  range) fill the two limits, marked `class`, unless a limit was typed. Depth stays typed.
- **Inputs added after the first files** (issue #4: the dry-bulk hold, the bow rake over depth, the stern void of a pushed barge, the end bulkhead allowance, the gas tank arrangement values, the water density, the squat safety margin, the 14 cost rates, the 17 aft block values of a self-propelled hull): a file that lacks one of these
  inputs, in version 1 or in a version 2 file from a test build, is read with the fixed value the engine
  used before the input existed (a table in `vdesign.rs`, not the Defaults Profile), marked `default`, with
  one warning per Option. Results cannot change. Every other left-out input is still refused. A Defaults
  Profile copy that lacks an entry for such an input takes the built-in entry; `profile_version` is unchanged.
  An input can also apply to some Formations only (the stern void is for a pushed barge or convoy): the
  input schema lists them in `applies_to_formation`, and a session drops the input when the Formation changes
  and brings a typed value back when it changes back. In a session (a new Option, a reset input, a changed
  Cargo Containment or Formation) such an input takes the Defaults Profile value; only a file read from disk
  gets the fixed value.
  The aft block values apply to a self-propelled Formation only, for every Cargo Containment. Two of them, the
  battery and accommodation unit counts, have a default by Cargo phase (a keyed default, like the Cargo
  Containment), and so two rows each in the table of fixed values: an old file gets the value of its Cargo
  Containment (gas 1 and 2, dry bulk 2 and 1).

- **Type C tank defaults by Cargo** (issue #292): the gas tank design values are inputs and depend on the
  Cargo Containment, the Cargo and, for an LPG Cargo, its carriage condition (`lpg_carriage_condition`:
  `fully_pressurised`, `semi_pressurised` or `fully_refrigerated`, an input of the LPG Cargos only). New
  inputs: `lpg_carriage_condition`, `tank_design_pressure_mpa`, `tank_design_temperature_c`,
  `tank_material`, `tank_allowable_stress_mpa` (none: the material's), `tank_insulation_type`,
  `tank_insulation_thickness_mm` and `tank_structure_factor`; `fill_ratio` now takes its default from the
  same table (it was keyed by ruleset), and `tank_wall_thickness_mm` is an optional override (none: the
  pressure-vessel formula gives the thickness; a file that stores 25 keeps 25). The Defaults Profile holds
  one table for each of these inputs, `default_by_table`: rows of `when` (the keys it fits), `value`,
  `status` (`sourced`, with its `source`, or `estimate`) and `basis`; the first row that fits wins, and the
  profile must have a row for every combination the schema can make. An older profile copy that lacks the
  entries takes the built-in ones; its own `fill_ratio` default (one value for every Cargo) is kept as an
  estimate on every row. A file that lacks the new inputs gets the values the engine used before (a table in
  `vdesign.rs`, marked `default`), so its results do not change; an LPG file that names no carriage
  condition is read as fully pressurised, with its own warning that says so. A Design made now takes the
  profile values (`ovcd fill --as-new` does this for a hand-written file). The format version is not
  changed (version 2 is the version of this build).
  The tank outer diameter is measured to the outside of the insulation. The clearances (IGC 2.4.2 gives d
  plate to plate, so the insulation is taken off it), the tank top and the drawing use that face; the wall
  thickness formula and the volume start from the steel diameter, the outer diameter less twice the
  insulation. The results show both (`drawing.tank`).

- **Allowable stress basis** (issue #313): a new input `tank_allowable_stress_basis` says where the
  material's allowable stress comes from when no stress is typed. `igc_4_23_3_1` (the Defaults Profile value)
  is the IGC 4.23.3.1 limit, the lesser of Rm/A and Re/B; `published_design` is the stress the engine used
  before that check. Only S690QL1 differs (256.7 MPa against 285 MPa). A file that lacks the input gets
  `published_design` (a table in `vdesign.rs`, marked `default`), so its results do not change; a stress
  above the IGC limit is a soft warning. The format version is not changed.

- **LPG density and loading limit** (issue #315): two new inputs of the LPG Cargos only.
  `lpg_loading_limit_basis` is `igc_15_1_3` (the Defaults Profile value), `igc_15_5_2` or
  `refrigerated_density`; `lpg_loading_temperature_c` takes its default from a table by Cargo and carriage
  condition. On an IGC basis the cargo density is the saturated liquid density at the loading temperature and
  the tank is loaded to the loading limit of IGC 15.5.1; `fill_ratio` is the filling limit. A file that lacks
  the inputs gets `refrigerated_density` and 15 C (tables in `vdesign.rs`, marked `default`): the cargo data's
  density and the fill ratio as loaded, so its results do not change. The format version is not changed.

- **Dome rule** (issue #316): `tank_dome_diameter_m` and `tank_dome_height_m` are optional overrides. The
  Defaults Profile value is none (an `estimate` row): the dome rule of each tank, diameter = the larger of
  1.2 m and 0.17 x the steel outer diameter, height = the diameter in force. A file that stores a size keeps
  it; a file that lacks the inputs gets the fixed 2.0 m, as before. The dome in force is in `drawing.tank`.
  The format version is not changed.

## Consequences

- A saved file gives the same results when opened again, on Windows and macOS.
- Moulded depth had to be typed until issue #120. Now a file may leave it out, and `ovcd fill` writes
  the rule's value in. This changes no file that has a depth and no format version: a depth that is
  stored is used as it is. `ovcd new` still writes an example depth (typed), for the starter hull.
- A convoy Formation is valid in the file, but its results are not yet available.
