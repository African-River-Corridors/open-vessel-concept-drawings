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
  The `rule_trace` object (spec 0004, the Rule Trace) is left out too: every number it compares is
  already hashed in its own field, and the Rule Set versions and hashes cover the rules. So a rule
  change is reported as a rule change, not as "results differ". `UNHASHED_KEYS` in
  `crates/ovcd-format/src/run.rs` lists every top-level key the hash leaves out.

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
- **3** (issue #347, spec 0007; no release has shipped it, so later tickets add to it: #274's
  `service_area` and `rule_sets`, the ADR 0012 hull block, the tug and Formation inputs of spec 0007): the stern
  and propulsion inputs of a self-propelled hull, in a new group "Stern and propulsion": `stern_form`,
  `stern_run_length_m`, `stern_transom_height_m`, `tunnel_crown_height_m`, `propulsor_type`,
  `propulsor_count`, `propeller_diameter_m`, `power_source`, `installed_power_kw`, `design_speed_kn`,
  `installed_power_margin`, `bow_thruster`, `bow_thruster_power_kw`, `battery_energy_kwh`,
  `battery_unit_energy_kwh`, `wheelhouse_type`, `sightline_method` and `stern_block_mass_included`. A file of
  version 1 or 2 that lacks them gets the value that reproduces today's stern block (a table in `vdesign.rs`,
  marked `default`, one warning per Option): a square transom stern, an unspecified propulsor (the fixed
  efficiency and today's thruster layout), battery-electric, no bow thruster, an elevating wheelhouse with the
  margin sightline, no stern block mass in the lightship. So its results do not change. A new Option takes the
  Defaults Profile values: `stern_form`, `propulsor_type` and `power_source` by Cargo phase, and
  `wheelhouse_eye_height_m` 1.65 m (ES-TRIN 7.02; a file keeps its own, and a file without it 1.50 m). An input
  can now apply only when an earlier input holds some values (`applies_when` in the schema):
  `thruster_count` and `propulsive_efficiency` only with an `unspecified` propulsor, the count and diameter
  only with a specified one, the tunnel crown only with a tunnel stern, the bow thruster power only with a
  bow thruster, the battery energy only with a battery Power Source. The engine reads none of the new inputs
  yet: later tickets of spec 0007 give them effect. A profile entry may say `status` (`sourced` with its
  `source`, or `estimate`) beside its `basis`; the schema shows it as `default_status`. Saving writes
  version 3. Issue #350: `block_coefficient` may now be none, its new Defaults Profile value: the engine
  then takes the rule, the volume of the raked box with its bow rake and Stern Form at the design draught
  over L x B x T, or 0.85 when the hull has no Stern Form (a barge, or a square transom). A file stores the
  value as used, so a file with 0.85 keeps 0.85 and its results; a typed value is used as typed.
  Issue #351 adds two inputs to the group, held only with a specified Propulsor type:
  `propulsor_efficiency_factor` (kappa) and `bollard_pull_factor` (kappa_b), with Defaults Profile values
  by Propulsor type. A file without them reads 1.0 for each; it never holds a specified Propulsor, so
  neither applies to it. Issue #353 adds `trip_length_km`, held only with a battery Power Source:
  none (no route) keeps the battery unit count; a length sizes the battery on the energy per trip. A
  file without it reads none. Issue #357 adds the Formation kinds `atb` and `towed_barge`, a "Tug" group
  of the tug inputs of a barge Formation (`tug_included`, `tug_preset`, the tug's size, its stern and
  propulsion inputs prefixed `tug_`, `formation_resistance_allowance`, and for an ATB `notch_depth_m` and
  `notch_width_m`), with defaults by Formation kind, and a fourth source marker, `reference`, for a value
  a tug preset gave. A file without them reads `tug_included: false`, the barge alone, as before.
  Issue #446 (spec 0008, ADR 0012) adds the hull block, a "Hull Form" group: `hull_form` (`box`,
  `raked_box`, `v_bottom`, `chine`, `shipd`, `offsets`), `bow_rake_length_m`, `bow_rake_rise_m`,
  `bilge_radius_m`, `sheer_aft_m`, `sheer_fwd_m`, `camber_m`, `deadrise_deg`, `keel_half_width_m`,
  `chine_half_breadth_m`, `chine_height_m`, and two object-valued inputs: `shipd_parameters` (an object of
  named numbers) and `hull_offsets` (a table of rows `station_x_m`, `y_m`, `z_m`). The schema gives the
  shape of each object-valued input. Each parameter applies only to the Hull Forms that use it
  (`applies_when`). A file of version 1 or 2 gets `hull_form = raked_box`, its own bow rake (none: the
  bow rake over depth rule, rising to the deck), a square bilge, no sheer and no camber, with one warning
  per Option, so its results do not change. Until a Hull Form's generator ticket lands, the engine refuses
  any `hull_form` but `raked_box` with a "not built yet" message. `shipd_parameters` defaults to none
  until the ShipD ticket (#452) brings the dataset rows.
  Issue #417 (typed openings, a trial) adds an "Openings" group for a gas Option: `deckhouse_front_opening`,
  `deckhouse_side_opening_distance_m`, `entrance_distance_m`, `engine_room_vent_inlet_distance_m` and
  `funnel_outlet_distance_m`. A file without them reads false, 5 m, 2 m, 2 m and 2 m, marked `default`, so
  its results do not change.
  Issue #274 (spec 0004) adds the rules block, a "Rules" group: `service_area` (`river-only` or
  `coastal-and-river`) and `rule_sets`, the list of Rule Sets the Option is checked against, each
  `{id, version, hash, source}` with `source` `default` (from the Service Area and the Cargo) or `typed`
  (added by the user). The `ruleset` input is removed: its one value, `IGC_2G`, is the per-tank rule the
  gas search always used, so a file that names it is read with the name dropped (a version 1 file keeps its
  notes about `ADN_G` and the stepped reading). A file of version 1 or 2, or a version 3 file saved before
  this block, gets `service_area = river-only` and the default stack of its Cargo Containment, each set at
  its highest built-in version with its content hash, marked `default`, with one warning per Option, so its
  results do not change (the `1.0.0` sets of the spec are not the stack the engine checked: `ovcd` is at
  2.1.0 and `lr-inland-gas` at 1.1.0). Opening a stored set: a stored version that is built in is used; a
  newer built-in version with other content gives a warning, never a swap; a stored hash that differs from
  the built-in hash of that version gives a warning that the rules differ from the file; a stored version
  the engine lacks, with no `--rule-set` supplying it, refuses the Option with a message naming the set.
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

- **Design liquid pressure** (issue #374): a new input `tank_liquid_pressure` (`static`, the Defaults
  Profile value; `igc_4_28_2`; `none`) sets the liquid pressure the wall formula adds to the design
  pressure (IGC 4.23.2.2, 4.28.1.2). A file that lacks it gets `none` (a table in `vdesign.rs`, marked
  `default`): the vapour pressure only, so its results do not change. The format version is not changed.

- **Tank head type** (issue #375): a new input `tank_head_type` (`hemisphere`, `ellipsoidal_2to1`), with a
  default table by Cargo and carriage condition in the Defaults Profile. A file that lacks it gets
  `hemisphere` (a table in `vdesign.rs`, marked `default`), so its results do not change. The format version
  is not changed.

## Consequences

- A saved file gives the same results when opened again, on Windows and macOS.
- Moulded depth had to be typed until issue #120. Now a file may leave it out, and `ovcd fill` writes
  the rule's value in. This changes no file that has a depth and no format version: a depth that is
  stored is used as it is. `ovcd new` still writes an example depth (typed), for the starter hull.
- A convoy Formation is valid in the file, but its results are not yet available.
