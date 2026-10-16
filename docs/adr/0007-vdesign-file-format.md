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
  The Vessel Class change (issue #94) also changes the input table before the next release, so it
  reuses version 2 and does not bump again: a third source marker `class` (alongside `typed` and
  `default`) marks an input that a Vessel Class gave, and two inputs, `vessel_scheme` and `vessel_class`,
  are held only by an Option defined by a standard class (an Option with typed dimensions has neither,
  so a version 1 file needs no change). The class is derived: after every read it is the smallest class
  of the scheme whose length and beam maxima hold the vessel, so length and beam decide it; a stored
  class that disagrees is replaced, with a warning. Picking a class sets length, beam and Formation
  (and a convoy's counts), marked `class`; its draught (top of the range) and air draft (bottom of the
  range) fill the two limits, marked `class`, unless a limit was typed. Depth stays typed.
- **Inputs added after the first files** (issue #4: the dry-bulk hold, the bow rake over depth, the stern void of a pushed barge, the end bulkhead allowance): a file that lacks one of these
  inputs, in version 1 or in a version 2 file from a test build, is read with the fixed value the engine
  used before the input existed (a table in `vdesign.rs`, not the Defaults Profile), marked `default`, with
  one warning per Option. Results cannot change. Every other left-out input is still refused. A Defaults
  Profile copy that lacks an entry for such an input takes the built-in entry; `profile_version` is unchanged.
  An input can also apply to some Formations only (the stern void is for a pushed barge or convoy): the
  input schema lists them in `applies_to_formation`, and a session drops the input when the Formation changes
  and brings a typed value back when it changes back. In a session (a new Option, a reset input, a changed
  Cargo Containment or Formation) such an input takes the Defaults Profile value; only a file read from disk
  gets the fixed value.

## Consequences

- A saved file gives the same results when opened again, on Windows and macOS.
- Moulded depth had to be typed until issue #120. Now a file may leave it out, and `ovcd fill` writes
  the rule's value in. This changes no file that has a depth and no format version: a depth that is
  stored is used as it is. `ovcd new` still writes an example depth (typed), for the starter hull.
- A convoy Formation is valid in the file, but its results are not yet available.
