//! The Rule Trace of an Option (spec 0004, "Rule Trace"): the Measures of one computed Option, the
//! default stack of Rule Sets, and the trace as JSON. The evaluator is `ovcd_core::rule_trace`.

use crate::profile::Profile;
use crate::rule_set::{Library, Selection};
use crate::simulate::Measured;
use crate::vdesign::{DesignOption, INPUTS, InputKind, Source, StoredInput};
use arc_rules::model::Compare;
use ovcd_core::gas_arrangement::RuleInputs;
use ovcd_core::rule_trace::{
    ArgDef, MeasureInfo, MeasureSource, MeasureValue, MethodStatus, ObjectType, Outcome, Trace,
    TraceRow, ValueKind, evaluate, measure_def,
};
use ovcd_core::rules::{ArgValue, MeasureRef};
use serde_json::{Value, json};

/// The Service Area of an Option that stores none (spec 0004, "Service Area and default stacks").
pub const DEFAULT_SERVICE_AREA: &str = "river-only";

/// The registry entry of a Measure: a computed Measure of the core, or `input.<name>` for an input
/// of the schema that is a number, a yes/no or a choice. The unit of an input Measure is its unit in
/// the built-in Defaults Profile (`-` for none).
pub fn measure_info(id: &str) -> Option<MeasureInfo> {
    if let Some(def) = measure_def(id) {
        return Some(def.info());
    }
    let name = id.strip_prefix("input.")?;
    let spec = INPUTS.iter().find(|s| s.name == name)?;
    let value = match spec.kind {
        InputKind::Number { .. } => ValueKind::Number,
        InputKind::Bool => ValueKind::Bool,
        InputKind::Choice(_) | InputKind::Cargo | InputKind::Text { .. } => ValueKind::Text,
        InputKind::Formation
        | InputKind::VesselScheme
        | InputKind::VesselClass
        | InputKind::ParameterSet
        | InputKind::OffsetsTable
        | InputKind::OpeningsList
        | InputKind::RuleSets => return None,
    };
    // Parsed once: a run asks this for every input.
    static PROFILE: std::sync::OnceLock<Profile> = std::sync::OnceLock::new();
    let unit = match PROFILE.get_or_init(Profile::built_in).unit(name) {
        "" => "-".to_string(),
        u => u.to_string(),
    };
    Some(MeasureInfo {
        unit,
        object: ObjectType::Option.key().to_string(),
        value,
        args: Vec::new(),
        // An input Measure echoes the value used; there is no method to check.
        method_status: MethodStatus::Verified,
    })
}

/// The default stack of Rule Set ids for a Cargo Containment (spec 0004, "Service Area and default
/// stacks"). Both Service Areas have the same stacks until more sets land. Only the ids the library
/// holds are used.
pub fn default_stack_ids(containment: Option<&str>) -> Vec<&'static str> {
    if containment == Some("type_c_tanks") {
        // `igc-2g` includes `is2008-a` (IGC 2.2.2); naming it here too keeps the IS Code rows in the
        // stack whatever the gas sets hold. A set selected twice is checked once.
        vec!["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]
    } else {
        vec!["is2008-a", "ovcd"]
    }
}

/// The `rule_sets` a file saved before the input existed is read with: the stack the engine checked
/// every Option against then (issue #274), whatever the profile's default stacks are now.
pub fn legacy_rule_sets(containment: Option<&str>) -> Value {
    let library = crate::rule_set::built_in();
    let sets: Vec<Value> = default_stack_ids(containment)
        .into_iter()
        .filter_map(|id| {
            // The IS Code and IGC sets held every light condition to the angle of max GZ until
            // issue #498; a file saved before keeps that version, so its results do not move.
            match legacy_version(id) {
                Some(version) => library.find(id, version),
                None => library.latest(id),
            }
        })
        .map(|set| {
            json!({
                "id": set.id,
                "version": set.version,
                "hash": set.hash,
                "source": "default",
            })
        })
        .collect();
    Value::Array(sets)
}

/// The version of a set a file saved before `rule_sets` existed is read with, when it is not the
/// highest.
fn legacy_version(id: &str) -> Option<&'static str> {
    match id {
        "is2008-a" => Some("1.0.0"),
        "igc-2g" => Some("1.1.0"),
        _ => None,
    }
}

/// The default `rule_sets` input for a Service Area and a Cargo Containment, from the profile's
/// default stacks (the built-in stack when the profile gives none): each set at its highest built-in
/// version, marked `default`, with the content hash of that version.
pub fn default_rule_sets(
    profile: &Profile,
    service_area: &str,
    containment: Option<&str>,
) -> Value {
    let gas = containment == Some("type_c_tanks");
    // Issue #339: a liquid bulk Cargo Containment has its own stack, with the liquid Rule Sets.
    let found = if containment == Some("integral_tanks") {
        profile.default_liquid_stack(service_area)
    } else {
        profile.default_stack(service_area, gas)
    };
    let ids: Vec<String> = found.unwrap_or_else(|| {
        default_stack_ids(containment)
            .into_iter()
            .map(String::from)
            .collect()
    });
    rule_sets_of(ids)
}

/// The `rule_sets` entries for set ids: each at its highest built-in version, with its hash, marked
/// `default`.
fn rule_sets_of(ids: Vec<String>) -> Value {
    let library = crate::rule_set::built_in();
    let sets: Vec<Value> = ids
        .iter()
        .filter_map(|id| library.latest(id))
        .map(|set| {
            json!({
                "id": set.id,
                "version": set.version,
                "hash": set.hash,
                "source": "default",
            })
        })
        .collect();
    Value::Array(sets)
}

/// The entries of an Option's `rule_sets` input (empty when it has none).
fn rule_set_entries(option: &DesignOption) -> Vec<Value> {
    option
        .input("rule_sets")
        .and_then(|i| i.value.as_array().cloned())
        .unwrap_or_default()
}

/// Write the Option's `rule_sets`. The input is marked `typed` when any entry is `typed`, else
/// `default`. The stored results hash is dropped: the stack changed.
fn write_rule_sets(option: &mut DesignOption, list: Vec<Value>) {
    let typed = list.iter().any(|e| e["source"] == "typed");
    let stored = StoredInput {
        value: Value::Array(list),
        source: if typed {
            Source::Typed
        } else {
            Source::Default
        },
    };
    match option.inputs.iter_mut().find(|(n, _)| n == "rule_sets") {
        Some((_, slot)) => *slot = stored,
        None => option.inputs.push(("rule_sets".into(), stored)),
    }
    option.results_hash = None;
}

/// Reset the `default` entries of the Option's `rule_sets` to the default stack of its Service Area
/// and Cargo Containment, from `profile`, and keep the `typed` ones (a typed entry replaces the
/// default one with its id). Issue #275: the Service Area or the Cargo changed.
pub fn refresh_default_rule_sets(option: &mut DesignOption, profile: &Profile) {
    if option.input("rule_sets").is_none() {
        return;
    }
    let containment = option
        .input("cargo_containment")
        .and_then(|i| i.value.as_str())
        .map(String::from);
    let mut list = default_rule_sets(profile, service_area_of(option), containment.as_deref())
        .as_array()
        .cloned()
        .unwrap_or_default();
    for typed in rule_set_entries(option)
        .into_iter()
        .filter(|e| e["source"] == "typed")
    {
        match list.iter().position(|e| e["id"] == typed["id"]) {
            Some(at) => list[at] = typed,
            None => list.push(typed),
        }
    }
    write_rule_sets(option, list);
}

/// Add a set to the Option's `rule_sets`, as `typed`, at `id@version`. Another version of the same
/// id is replaced. Refused when the library lacks the set, or the selection that results is refused
/// (an include cycle, two versions of one set, one rule id with two meanings).
pub fn add_rule_set(
    option: &mut DesignOption,
    library: &Library,
    spec: &str,
) -> Result<(), String> {
    let set = library
        .lookup(spec)
        .ok_or_else(|| format!("Rule Set `{spec}` is not in this engine"))?;
    let mut list = rule_set_entries(option);
    let entry = json!({
        "id": set.id, "version": set.version, "hash": set.hash, "source": "typed",
    });
    match list.iter().position(|e| e["id"] == json!(set.id)) {
        Some(at) => list[at] = entry,
        None => list.push(entry),
    }
    check_selection(library, &list)?;
    write_rule_sets(option, list);
    Ok(())
}

/// Remove a set from the Option's `rule_sets`. The `ovcd` set cannot be removed, and a set that a
/// selected set `includes` cannot be removed while that set is selected.
pub fn remove_rule_set(
    option: &mut DesignOption,
    library: &Library,
    id: &str,
) -> Result<(), String> {
    if id == "ovcd" {
        return Err("the `ovcd` Rule Set cannot be removed".to_string());
    }
    let mut list = rule_set_entries(option);
    let at = list
        .iter()
        .position(|e| e["id"] == json!(id))
        .ok_or_else(|| format!("Rule Set `{id}` is not selected"))?;
    for other in &list {
        let (oid, over) = (other["id"].as_str(), other["version"].as_str());
        if let (Some(oid), Some(over)) = (oid, over)
            && oid != id
            && library
                .find(oid, over)
                .is_some_and(|s| s.includes.iter().any(|i| i.id == id))
        {
            return Err(format!(
                "Rule Set `{id}` is required by {oid}@{over}, which is selected"
            ));
        }
    }
    list.remove(at);
    write_rule_sets(option, list);
    Ok(())
}

/// The selection of the stored sets must be accepted by the library.
fn check_selection(library: &Library, list: &[Value]) -> Result<(), String> {
    let pairs: Vec<(&str, &str)> = list
        .iter()
        .filter_map(|e| Some((e["id"].as_str()?, e["version"].as_str()?)))
        .collect();
    library.select(&pairs).map(|_| ())
}

/// The Service Area of an Option: its input, or the default.
pub fn service_area_of(option: &DesignOption) -> &str {
    option
        .input("service_area")
        .and_then(|i| i.value.as_str())
        .unwrap_or(DEFAULT_SERVICE_AREA)
}

/// The sets an Option stores: `(id, version, hash)` of each entry of its `rule_sets` input. `None`
/// when the Option has none (an Option built by hand, not read from a file).
fn stored_rule_sets(option: &DesignOption) -> Option<Vec<(String, String, String)>> {
    let list = option.input("rule_sets")?.value.as_array()?;
    Some(
        list.iter()
            .filter_map(|e| {
                let text = |k: &str| e.get(k).and_then(Value::as_str).map(str::to_string);
                Some((text("id")?, text("version")?, text("hash")?))
            })
            .collect(),
    )
}

/// The sets a run of one Option checks: the sets the Option stores (`rule_sets`, at the stored
/// versions), then each extra set (loaded with `--rule-set`). An extra set with the id of a set in
/// the stack replaces it for the run.
///
/// A stored version the library lacks is refused with a message naming the set. A stored hash that
/// differs from the hash of that version in the library, and a newer version with other content,
/// each give a warning, never a swap.
pub fn stack_for<'a>(
    library: &'a Library,
    option: &DesignOption,
    extra: &[(String, String)],
) -> Result<Selection<'a>, String> {
    let mut chosen: Vec<(String, String)> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    match stored_rule_sets(option) {
        Some(stored) => {
            for (id, version, hash) in stored {
                let replaced = extra.iter().any(|(i, _)| *i == id);
                match library.find(&id, &version) {
                    Some(set) => {
                        if !replaced && set.hash != hash {
                            warnings.push(format!(
                                "the file stores Rule Set {id}@{version} with hash {}, but the rules \
                                 of that version in this engine have hash {}: the rules differ from \
                                 the file",
                                short_hash(&hash),
                                short_hash(&set.hash)
                            ));
                        }
                    }
                    None if replaced => {}
                    None => {
                        return Err(format!(
                            "Rule Set {id}@{version} is not in this engine: pass the file with \
                             --rule-set, or pick another version"
                        ));
                    }
                }
                if let Some(newer) = library.latest(&id)
                    && crate::rule_set::is_newer(&newer.version, &version)
                    && newer.hash != hash
                {
                    warnings.push(format!(
                        "Rule Set {id}@{version} is used as the file stores it; {} is also built in \
                         with other content",
                        newer.version
                    ));
                }
                chosen.push((id, version));
            }
        }
        None => {
            for id in default_stack_ids(
                option
                    .input("cargo_containment")
                    .and_then(|i| i.value.as_str()),
            ) {
                if let Some(set) = library.latest(id) {
                    chosen.push((set.id.clone(), set.version.clone()));
                }
            }
        }
    }
    for (id, version) in extra {
        chosen.retain(|(i, _)| i != id);
        chosen.push((id.clone(), version.clone()));
    }
    let pairs: Vec<(&str, &str)> = chosen
        .iter()
        .map(|(i, v)| (i.as_str(), v.as_str()))
        .collect();
    let mut selection = library.select(&pairs)?;
    warnings.extend(std::mem::take(&mut selection.warnings));
    selection.warnings = warnings;
    Ok(selection)
}

fn short_hash(hash: &str) -> &str {
    &hash[..hash.len().min(12)]
}

/// The derived, read-only inputs an Applicability can name (spec 0004 additions of spec 0006
/// §5.2), with their values. The engine works them out; a user never types them.
pub const DERIVED_INPUTS: &[(&str, &[&str])] = &[
    ("cargo_class", &["gas", "dry_bulk", "liquid_bulk"]),
    // Issue #355: the Formation's kind, read from the `formation` input.
    ("formation_kind", crate::vdesign::FORMATION_KINDS),
    ("deadweight_band", DEADWEIGHT_BANDS),
    // Issue #338: the ADN Tank Vessel Type of the Cargo, from its Crude Grade (ADN 2025 3.2.3.3).
    ("adn_vessel_type", &["N_open", "N_closed", "C"]),
    // Issue #543: the length overall, in the bands of SOLAS V/22.1 and ES-TRIN 28.02.
    ("length_band", LENGTH_BANDS),
    // Issue #282: whether the Option gives an angle of down-flooding.
    (
        "downflooding_angle",
        ovcd_core::measured::DOWNFLOODING_ANGLES,
    ),
];

pub use ovcd_core::measured::{DEADWEIGHT_BANDS, LENGTH_BANDS, deadweight_band, length_band};

/// The ADN Tank Vessel Type of a Cargo, from its Crude Grade; `None` for a Cargo with no grade.
pub fn adn_vessel_type_of(
    cargo: &str,
) -> Option<ovcd_core::liquid_bulk::adn_vessel_type::AdnVesselType> {
    use ovcd_core::liquid_bulk::{adn_vessel_type as adn, crude_grade};
    crude_grade::grade_of(cargo).map(|g| adn::derive(&adn::AdnFlags::of(g)).vessel_type)
}

/// The `cargo_class` value of a Cargo Class.
pub fn cargo_class_value(class: ovcd_core::cargo::CargoClass) -> &'static str {
    match class {
        ovcd_core::cargo::CargoClass::Gas => "gas",
        ovcd_core::cargo::CargoClass::DryBulk => "dry_bulk",
        ovcd_core::cargo::CargoClass::LiquidBulk => "liquid_bulk",
    }
}

/// The values an Applicability may give for an input: a derived input, a choice input of the
/// schema, or `cargo` (the Cargo keys). `None` for an input that cannot be named.
pub fn applicability_values(input: &str) -> Option<Vec<String>> {
    if let Some((_, values)) = DERIVED_INPUTS.iter().find(|(n, _)| *n == input) {
        return Some(values.iter().map(|v| v.to_string()).collect());
    }
    let spec = INPUTS.iter().find(|s| s.name == input)?;
    match spec.kind {
        InputKind::Choice(values) => Some(values.iter().map(|v| v.to_string()).collect()),
        InputKind::Cargo => Some(
            ovcd_core::cargo::CARGO_DATABASE
                .iter()
                .map(|(k, _)| k.to_string())
                .collect(),
        ),
        _ => None,
    }
}

/// The Measures of one computed Option.
pub struct OptionMeasures<'a> {
    pub option: &'a DesignOption,
    pub measured: &'a Measured,
}

impl OptionMeasures<'_> {
    /// Whether ES-TRIN 28.04(1)(a) can be read from the inputs: a self-propelled Option with a
    /// specified Propulsor.
    fn redundancy_applies(&self) -> bool {
        let kind = self
            .option
            .input("formation")
            .and_then(|i| i.value.get("kind"))
            .and_then(Value::as_str);
        let propulsor = self
            .option
            .input("propulsor_type")
            .and_then(|i| i.value.as_str());
        kind == Some("self_propelled") && propulsor.is_some_and(|p| p != "unspecified")
    }

    /// ES-TRIN 28.04(1)(a), from the inputs: a craft of 110 m or less is not covered; a longer one
    /// needs a multi-propeller system or a single propeller, each with a bow thruster. Equal power,
    /// independence and a thruster that works unladen are assumed.
    fn redundancy(&self) -> Result<bool, String> {
        let number = |n: &str| self.option.input(n).and_then(|i| i.value.as_f64());
        let loa = number("length_overall_m").ok_or("the Option has no length overall")?;
        if loa <= 110.0 {
            return Ok(true);
        }
        let thruster = self
            .option
            .input("bow_thruster")
            .and_then(|i| i.value.as_str())
            .is_some_and(|t| t != "none");
        // One propulsor or more, with a bow thruster that is controlled from the wheelhouse: the
        // same condition in this model.
        Ok(thruster && number("propulsor_count").unwrap_or(1.0) >= 1.0)
    }
}

impl MeasureSource for OptionMeasures<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }

    fn objects(&self, object: &str) -> Vec<String> {
        self.measured.objects_by_key(object, &self.option.name)
    }

    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        if let Some(name) = m.id.strip_prefix("input.") {
            let stored = self
                .option
                .input(name)
                .ok_or_else(|| format!("the Option has no input `{name}`"))?;
            return match &stored.value {
                Value::Number(n) => n
                    .as_f64()
                    .map(MeasureValue::Number)
                    .ok_or_else(|| format!("`{name}` is not a number")),
                Value::Bool(b) => Ok(MeasureValue::Bool(*b)),
                Value::String(s) => Ok(MeasureValue::Text(s.clone())),
                _ => Err(format!("`{name}` has no value")),
            };
        }
        if m.id == "propulsion.redundancy_28_04" {
            return self.redundancy().map(MeasureValue::Bool);
        }
        if m.id == "option.adn_vessel_type" {
            // Issue #338: the type the Crude Grade of the Cargo gives, and the branch that decided it.
            let cargo = self
                .option
                .input("cargo")
                .and_then(|i| i.value.as_str())
                .ok_or_else(|| "the Option has no Cargo".to_string())?;
            return adn_vessel_type_of(cargo)
                .map(|t| MeasureValue::Text(t.key().to_string()))
                .ok_or_else(|| "the Cargo has no ADN Tank Vessel Type".to_string());
        }
        if m.id == "openings.front_clear" {
            // Issue #417: read from the typed input; no opening on the front means it is clear.
            return match self
                .option
                .input("deckhouse_front_opening")
                .map(|i| &i.value)
            {
                Some(Value::Bool(open)) => Ok(MeasureValue::Bool(!open)),
                _ => Err("the Option has no `deckhouse_front_opening`".to_string()),
            };
        }
        self.measured.value(m, index)
    }

    fn has(&self, measure: &MeasureRef) -> bool {
        match measure.id.as_str() {
            "propulsion.redundancy_28_04" => self.redundancy_applies(),
            // Rules on these are `not evaluated`, with their gaps: the Option has them to name.
            "wheelhouse.field_of_view"
            | "wheelhouse.raised_stability"
            | "formation.separate_vessels"
            | "formation.notch_wing_walls" => true,
            _ => self.measured.has(measure),
        }
    }

    fn missing(&self, object: &str) -> Option<String> {
        let object = ObjectType::from_key(object)?;
        let gas = self
            .option
            .input("cargo_containment")
            .and_then(|i| i.value.as_str())
            == Some("type_c_tanks");
        if object == ObjectType::Tank && gas && self.measured.tank.is_none() {
            return Some("no tank fits the hull, so there is no tank to check".to_string());
        }
        if object == ObjectType::OilOutflow && self.measured.oil_outflow.is_none() {
            return Some(
                "the oil outflow was not computed: the layout has no cargo volume or no deadweight"
                    .to_string(),
            );
        }
        if (object == ObjectType::DamageCase && self.measured.damage.is_empty())
            || (object == ObjectType::MarpolDamageCase && self.measured.marpol_damage.is_empty())
        {
            return Some(
                "the damage cases were not computed: the laden departure condition has no floating position"
                    .to_string(),
            );
        }
        if object != ObjectType::LoadingCondition || !self.measured.conditions.is_empty() {
            return None;
        }
        Some(if gas {
            "no stability loading conditions: the barge carries no cargo".to_string()
        } else {
            "intact stability is not computed for this Cargo yet (a first-order GM only)"
                .to_string()
        })
    }

    fn object_input(&self, object: &str, index: usize, name: &str) -> Option<String> {
        self.measured
            .object_input(object, index, name)
            .or_else(|| self.input(name))
    }

    fn input(&self, name: &str) -> Option<String> {
        match name {
            "formation_kind" => {
                return self
                    .option
                    .input("formation")?
                    .value
                    .get("kind")?
                    .as_str()
                    .map(str::to_string);
            }
            "cargo_class" => {
                let cargo = self.option.input("cargo")?.value.as_str()?;
                let props = ovcd_core::cargo::get_cargo(cargo).ok()?;
                return Some(cargo_class_value(props.phase).to_string());
            }
            "downflooding_angle" => {
                return Some(
                    ovcd_core::measured::downflooding_angle_value(
                        self.option
                            .input("downflooding_angle_deg")
                            .and_then(|i| i.value.as_f64()),
                        self.option
                            .input("downflooding_openings")
                            .and_then(|i| i.value.as_array())
                            .is_some_and(|rows| !rows.is_empty()),
                    )
                    .to_string(),
                );
            }
            "length_band" => {
                return self
                    .option
                    .input("length_overall_m")?
                    .value
                    .as_f64()
                    .map(|l| length_band(l).to_string());
            }
            "deadweight_band" => {
                return self
                    .measured
                    .deadweight_t
                    .map(|dw| deadweight_band(dw).to_string());
            }
            "adn_vessel_type" => {
                let cargo = self.option.input("cargo")?.value.as_str()?;
                return adn_vessel_type_of(cargo).map(|t| t.key().to_string());
            }
            _ => {}
        }
        self.option
            .input(name)
            .and_then(|i| i.value.as_str())
            .map(str::to_string)
            .or_else(|| (name == "service_area").then(|| DEFAULT_SERVICE_AREA.to_string()))
    }
}

/// The Option's inputs as the arrangement search reads them for its Rule Sets (#277): every input
/// with a text value, the derived `service_area` and `cargo_class` (the search works out
/// `deadweight_band` per candidate), and every input Measure with its registry entry and value.
pub fn rule_inputs(option: &DesignOption) -> RuleInputs {
    let source = OptionMeasures {
        option,
        measured: &Measured::default(),
    };
    let mut applicability: Vec<(String, String)> = Vec::new();
    for spec in INPUTS {
        if let Some(v) = source.input(spec.name) {
            applicability.push((spec.name.to_string(), v));
        }
    }
    for name in [
        "cargo_class",
        "formation_kind",
        "length_band",
        "downflooding_angle",
        "adn_vessel_type",
    ] {
        if let Some(v) = source.input(name) {
            applicability.push((name.to_string(), v));
        }
    }
    let mut measures = Vec::new();
    for spec in INPUTS {
        let id = format!("input.{}", spec.name);
        if let (Some(info), Ok(value)) = (measure_info(&id), source.value(&bare(&id), 0)) {
            measures.push((id, info, value));
        }
    }
    // Issue #417: the Measures the Option's typed inputs give, which are not `input.<name>`.
    let front = "openings.front_clear";
    if let (Some(info), Ok(value)) = (measure_info(front), source.value(&bare(front), 0)) {
        measures.push((front.to_string(), info, value));
    }
    RuleInputs {
        applicability,
        measures,
    }
}

fn bare(id: &str) -> MeasureRef {
    MeasureRef {
        id: id.to_string(),
        args: Vec::new(),
    }
}

/// The Rule Trace of a computed Option.
pub fn trace_of(selection: &Selection, option: &DesignOption, measured: &Measured) -> Trace {
    let source = OptionMeasures { option, measured };
    evaluate(&selection.sets, &source)
}

fn value_json(v: &MeasureValue) -> Value {
    match v {
        MeasureValue::Number(x) => json!(x),
        MeasureValue::Bool(b) => json!(b),
        MeasureValue::Text(t) => json!(t),
    }
}

fn measure_json(m: &MeasureRef) -> Value {
    let args: serde_json::Map<String, Value> = m
        .args
        .iter()
        .map(|(k, v)| {
            let v = match v {
                ArgValue::Number(x) => json!(x),
                ArgValue::Text(t) => json!(t),
            };
            (k.clone(), v)
        })
        .collect();
    json!({"id": m.id, "args": args})
}

fn row_json(r: &TraceRow) -> Value {
    json!({
        "rule": r.rule,
        "rule_sets": r.rule_sets,
        "title": r.title,
        "citation": r.citation,
        "origin": r.origin.map(|o| o.key()),
        "rule_status": if r.verified { "verified" } else { "unverified" },
        "method_status": r.method_status.map(MethodStatus::key),
        "measure": r.measure.as_ref().map(measure_json),
        "object_type": r.object_type,
        "object": r.object,
        "inputs": r.inputs.iter().map(|u| json!({"measure": u.measure, "value": value_json(&u.value)})).collect::<Vec<_>>(),
        "value": r.value.as_ref().map(value_json),
        "unit": r.unit,
        "compare": r.compare.map(|c| c.key()),
        "limit": r.limit,
        "tolerance": r.tolerance,
        "margin": r.margin,
        "consequence": r.consequence.map(|c| c.key()),
        "outcome": r.outcome.key(),
        "governing": r.governing,
        "sizes": r.sizes,
        "gap": r.gap,
        "finding": r.finding,
    })
}

/// The trace as JSON: the selected sets, every row, and the verdict. A pass with blocking Rules
/// not evaluated names them; nothing here says "meets every rule" while one is not evaluated.
pub fn trace_json(t: &Trace, service_area: &str, warnings: &[String]) -> Value {
    let not_evaluated: Vec<Value> = t
        .blocking_not_evaluated()
        .iter()
        .map(|r| json!({"rule": r.rule, "title": r.title}))
        .collect();
    let failing = t.rows.iter().filter(|r| r.outcome == Outcome::Fail).count();
    let summary = if !t.passes() {
        format!("fails {failing} blocking Rule check(s)")
    } else if not_evaluated.is_empty() {
        "meets every evaluated blocking Rule; no blocking Rule is left not evaluated".to_string()
    } else {
        let names: Vec<String> = t
            .blocking_not_evaluated()
            .iter()
            .map(|r| r.title.clone())
            .collect();
        format!(
            "meets every evaluated blocking Rule; {} blocking Rule(s) not evaluated: {}",
            names.len(),
            names.join(", ")
        )
    };
    // The scope of the result (spec 0004, "Service Area and default stacks"; ADR 0011): the Service
    // Area, the sets, what was not checked, and the verdict with its not evaluated Rules.
    let not_checked = if service_area == "river-only" {
        "no statutory technical code was checked, only these Rule Sets"
    } else {
        "only these Rule Sets were checked, not a statutory technical code"
    };
    let scope = format!(
        "Service Area {service_area}; Rule Sets {}; {not_checked}. {summary}",
        t.sets.join(", ")
    );
    json!({
        "scope": scope,
        "rule_sets": t.sets,
        "service_area": service_area,
        "warnings": warnings,
        "rows": t.rows.iter().map(row_json).collect::<Vec<_>>(),
        "verdict": {
            "passes": t.passes(),
            "failing_rows": failing,
            "blocking_not_evaluated": not_evaluated,
            "summary": summary,
        },
    })
}

/// The arguments a Measure takes, for `ovcd rules --measures`.
pub fn args_text(args: &[ArgDef]) -> String {
    let names: Vec<&str> = args.iter().map(|a| a.name).collect();
    names.join(", ")
}

/// The Measure registry as text, for `ovcd rules --measures`.
pub fn measures_text() -> String {
    let mut out = String::from("Computed Measures:\n");
    for m in ovcd_core::rule_trace::MEASURES {
        out.push_str(&format!(
            "{}  {}  {}  args [{}]  method {}\n  {}\n",
            m.id,
            m.object.key(),
            m.unit,
            args_text(m.args),
            m.method_status.key(),
            m.method
        ));
    }
    out.push_str("\nInput Measures (object option, method verified):\n");
    for spec in INPUTS {
        let id = format!("input.{}", spec.name);
        if let Some(info) = measure_info(&id) {
            let kind = match info.value {
                ValueKind::Number => "number",
                ValueKind::Bool => "yes/no",
                ValueKind::Text => "text",
            };
            out.push_str(&format!("{id}  {}  {kind}\n", info.unit));
        }
    }
    out
}

/// The Rule each check of a result reports (spec 0004, "Each check gains the id of the Rule it
/// reports"). The `checks` list is in the results hash, so the id is not written into each check;
/// this table holds the link instead, and a test keeps it complete. A name that starts with
/// `Stability — ` is the stability check of one loading condition.
pub const CHECK_RULES: &[(&str, &str)] = &[
    ("Cargo > 0", "ovcd/capacity/cargo-positive"),
    ("feasible geometry", "ovcd/capacity/feasible-geometry"),
    ("cargo zone positive", "ovcd/capacity/cargo-zone-positive"),
    (
        "air draft within limit",
        "ovcd/capacity/air-draft-within-limit",
    ),
    ("intact gm at reported cargo", "ovcd/capacity/intact-gm"),
    // The per-condition checks of the IS Code loading conditions (issue #426, part 2; #628). Each
    // is in the results only when a condition fails it.
    ("Air draft", "ovcd/loading/air-draft"),
    ("Ballast plan", "ovcd/loading/ballast-plan"),
    (
        "Feasible arrangement",
        "ovcd/gas-arrangement/feasible-arrangement",
    ),
    (
        "tank length over diameter",
        "ovcd/gas-arrangement/tank-l-over-d",
    ),
    ("thrusters fit beam", "ovcd/aft-block/thrusters-fit-beam"),
    (
        "requested thruster count fits",
        "ovcd/aft-block/thruster-count",
    ),
    ("containers fit beam", "ovcd/aft-block/containers-fit-beam"),
    ("units fit in one row", "ovcd/aft-block/units-in-one-row"),
    (
        "wheelhouse fits beam",
        "ovcd/aft-block/wheelhouse-fits-beam",
    ),
    (
        "accom beside wheelhouse",
        "ovcd/aft-block/accom-beside-wheelhouse",
    ),
    ("gensets fit beam", "ovcd/aft-block/gensets-fit-beam"),
    (
        "gensets fit under deck",
        "ovcd/aft-block/gensets-under-deck",
    ),
    (
        "gensets carry load with one in reserve",
        "ovcd/aft-block/gensets-reserve",
    ),
    (
        "wheelhouse sees over cargo",
        "ovcd/aft-block/wheelhouse-sightline",
    ),
    ("aft end bulkhead space", "lr-inland/3.5.3/aft-end-space"),
    ("opening setback", "lr-inland/3.3.7/opening-setback"),
    ("fwd end bulkhead space", "lr-inland/3.5.3/end-space"),
    // The checks only the arrangement search makes (`fit`, #277).
    ("not overloaded", "ovcd/capacity/not-overloaded"),
    ("freeboard ok", "ovcd/capacity/freeboard"),
    ("capacity is volume bound", "ovcd/journey/volume-bound"),
    ("floats", "ovcd/journey/floats"),
    ("ukc consistent", "ovcd/journey/ukc-consistent"),
    ("utilisation reasonable", "ovcd/journey/utilisation"),
    (
        "freeboard at payload",
        "ovcd/gas-arrangement/freeboard-at-payload",
    ),
    ("Stability \u{2014} not assessed", "ovcd/stability/assessed"),
    // The resistance method's checks at each service speed (issue #420).
    ("keel clears bed", "ovcd/squat/keel-clears-bed"),
    ("keel clears bed with margin", "ovcd/squat/keel-margin"),
    ("froude depth subcritical", "ovcd/squat/froude-depth"),
    ("blockage within barrass range", "ovcd/squat/blockage"),
    ("methods agree within 50pct", "ovcd/squat/methods-agree"),
    (
        "stawave1 short wave validity",
        "ovcd/waves/short-wave-validity",
    ),
    (
        "wave height within screening range",
        "ovcd/waves/wave-height",
    ),
    ("bow length supplied", "ovcd/waves/bow-length-supplied"),
    (
        "service speed below limiting",
        "ovcd/resistance/service-below-limiting",
    ),
    ("blockage in range", "ovcd/resistance/blockage-in-range"),
    (
        "service speed clears squat",
        "ovcd/resistance/service-clears-squat",
    ),
    ("lackenby fit subcritical", "ovcd/resistance/lackenby-fit"),
    (
        "coefficients are first order",
        "ovcd/resistance/first-order-coefficients",
    ),
];

/// The check name without its environment: `keel clears bed[Channel ...]` is `keel clears bed`
/// (issue #420: the resistance method's checks come once per environment).
pub fn check_base_name(name: &str) -> &str {
    match name.split_once('[') {
        Some((base, rest)) if rest.ends_with(']') => base,
        _ => name,
    }
}

/// The checks as the Rule Trace reads them: each check of the result, then one verdict per check
/// that comes once per environment, under its name without the environment. It passes when it
/// passes in every environment, so a Rule on it fails when any environment fails.
pub fn with_check_verdicts(checks: Vec<(String, bool)>) -> Vec<(String, bool)> {
    let mut folded: Vec<(String, bool)> = Vec::new();
    for (name, ok) in &checks {
        let base = check_base_name(name);
        if base == name || checks.iter().any(|(n, _)| n == base) {
            continue;
        }
        match folded.iter_mut().find(|(n, _)| n == base) {
            Some((_, all)) => *all = *all && *ok,
            None => folded.push((base.to_string(), *ok)),
        }
    }
    let mut out = checks;
    out.extend(folded);
    out
}

/// The id of the Rule a check reports, or `None` for a check no Rule reports. A check that comes
/// once per environment reports the Rule of its name without the environment.
pub fn rule_of_check(name: &str) -> Option<&'static str> {
    let name = check_base_name(name);
    if name.starts_with("Stability \u{2014} ") && name != "Stability \u{2014} not assessed" {
        return Some("is2008/A.2.2/condition");
    }
    CHECK_RULES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, id)| *id)
}

/// What a box section gives the Rule Trace (the evaluation harness, issue #278): one loading
/// condition and its stability check, nothing else. It has no inputs, so a set or Rule that applies
/// only for some input value does not apply.
struct BoxMeasures<'a> {
    measured: &'a Measured,
}

impl MeasureSource for BoxMeasures<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }

    fn objects(&self, object: &str) -> Vec<String> {
        self.measured.objects_by_key(object, "Box")
    }

    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        self.measured.value(m, index)
    }

    fn input(&self, _name: &str) -> Option<String> {
        None
    }

    fn object_input(&self, object: &str, index: usize, name: &str) -> Option<String> {
        self.measured.object_input(object, index, name)
    }

    fn has(&self, measure: &MeasureRef) -> bool {
        self.measured.has(measure)
    }
}

/// The Rule Trace of one box loading condition under the sets `chosen` (`(id, version)`, in
/// selection order) of `library`, as `trace_json` writes it. The condition is named "Box". The
/// harness compares its rows with a case's expected verdicts, keyed by rule id.
pub fn box_trace(
    mut condition: ovcd_core::gas_stability::Condition,
    library: &Library,
    chosen: &[(&str, &str)],
) -> Result<Value, String> {
    condition.name = "Box".to_string();
    let check = ovcd_core::gas_arrangement::stability_check(&condition);
    let measured = Measured {
        checks: vec![(check.name, check.ok)],
        conditions: vec![condition],
        ..Measured::default()
    };
    let selection = library.select(chosen)?;
    let trace = evaluate(
        &selection.sets,
        &BoxMeasures {
            measured: &measured,
        },
    );
    Ok(trace_json(&trace, DEFAULT_SERVICE_AREA, &[]))
}

/// The Rule Trace of a self-propelled Option whose measured values the case gives (the evaluation
/// harness, issue #355). The Option is filled as a new one from `inputs`; the Measures that a run
/// reads from the results (the Blind Distance, the speed, the propeller ratios) come from `measured`
/// instead, so a case sets each to either side of a limit without a hull that happens to give it.
/// `measured` keys: `blind_distance_light_m`, `blind_distance_laden_m`, `speed_deep_water_kmh`,
/// `design_speed_kn`, `diameter_over_max`, `immersion_over_limit`, `blade_load_over_limit`.
pub fn stern_trace(
    inputs: &serde_json::Map<String, Value>,
    measured_values: &serde_json::Map<String, Value>,
    library: &Library,
    chosen: &[(&str, &str)],
) -> Result<Value, String> {
    // A harness run asks for a few hundred cases that differ only in the measured values: the filled
    // Option is made once for each set of typed inputs.
    static OPTIONS: std::sync::Mutex<Vec<(String, DesignOption)>> =
        std::sync::Mutex::new(Vec::new());
    static PROFILE: std::sync::OnceLock<Profile> = std::sync::OnceLock::new();
    let key = Value::Object(inputs.clone()).to_string();
    let cached = OPTIONS
        .lock()
        .map_err(|_| "the harness cache is poisoned".to_string())?
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, o)| o.clone());
    let option = match cached {
        Some(o) => o,
        None => {
            let mut typed = serde_json::Map::new();
            for (k, v) in inputs {
                typed.insert(k.clone(), json!({ "value": v, "source": "typed" }));
            }
            let file = json!({
                "format": "vdesign", "format_version": 3, "engine_version": crate::vdesign::ENGINE_VERSION,
                "units": "SI", "options": [{ "name": "Option 1", "inputs": typed }]
            });
            let filled = crate::vdesign::fill_as_new(
                &file.to_string(),
                PROFILE.get_or_init(Profile::built_in),
            );
            if !filled.report.errors.is_empty() {
                return Err(filled.report.errors.join("; "));
            }
            let o = filled.design.ok_or("the case has no Design")?.options[0].clone();
            OPTIONS
                .lock()
                .map_err(|_| "the harness cache is poisoned".to_string())?
                .push((key, o.clone()));
            o
        }
    };
    let option = &option;
    let number = |k: &str| measured_values.get(k).and_then(Value::as_f64);
    let mut measured = Measured {
        blind_distance_light_m: number("blind_distance_light_m"),
        blind_distance_laden_m: number("blind_distance_laden_m"),
        design_speed_kn: number("design_speed_kn"),
        diameter_over_max: number("diameter_over_max"),
        immersion_over_limit: number("immersion_over_limit"),
        blade_load_over_limit: number("blade_load_over_limit"),
        ..Measured::default()
    };
    if let Some(kmh) = number("speed_deep_water_kmh") {
        measured
            .speeds_kn
            .push(("Deep water".to_string(), kmh / 1.852));
    }
    let selection = library.select(chosen)?;
    let trace = trace_of(&selection, option, &measured);
    Ok(trace_json(&trace, service_area_of(option), &[]))
}

/// What a liquid bulk layout gives the Rule Trace (the evaluation harness, issue #338): the layout, the
/// deadweight, and the few inputs the liquid Rules read. It has no Option, so the inputs are the case's.
struct LiquidMeasures<'a> {
    measured: &'a Measured,
    /// The numeric inputs (`beam_m`, `length_overall_m`, `cofferdam_m`), by the name of the `input.<name>`
    /// Measure that reads them.
    numbers: &'a [(&'static str, f64)],
    service_area: &'a str,
    adn_vessel_type: &'a str,
}

impl MeasureSource for LiquidMeasures<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }

    fn objects(&self, object: &str) -> Vec<String> {
        self.measured.objects_by_key(object, "Option")
    }

    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        if let Some(name) = m.id.strip_prefix("input.") {
            return self
                .numbers
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| MeasureValue::Number(*v))
                .ok_or_else(|| format!("the layout case has no input `{name}`"));
        }
        if m.id == "option.adn_vessel_type" {
            return Ok(MeasureValue::Text(self.adn_vessel_type.to_string()));
        }
        self.measured.value(m, index)
    }

    fn input(&self, name: &str) -> Option<String> {
        match name {
            "cargo_class" => Some("liquid_bulk".to_string()),
            "service_area" => Some(self.service_area.to_string()),
            "adn_vessel_type" => Some(self.adn_vessel_type.to_string()),
            "deadweight_band" => self
                .measured
                .deadweight_t
                .map(|dw| deadweight_band(dw).to_string()),
            _ => None,
        }
    }

    fn has(&self, measure: &MeasureRef) -> bool {
        self.measured.has(measure)
    }

    fn missing(&self, object: &str) -> Option<String> {
        matches!(
            ObjectType::from_key(object),
            Some(ObjectType::DamageCase | ObjectType::MarpolDamageCase | ObjectType::OilOutflow)
        )
        .then(|| "the layout case has no loading condition, so no damage is computed".to_string())
    }
}

/// The inputs of [`liquid_trace`]: a hull, a cargo zone and a tank layout, as the evaluation harness
/// writes a case.
#[derive(Debug, Clone)]
pub struct LiquidTraceCase {
    pub loa_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
    pub zone_aft_m: f64,
    pub zone_fwd_m: f64,
    pub layout: ovcd_core::liquid_bulk::LiquidLayout,
    pub cargo_density_t_m3: f64,
    pub deadweight_t: f64,
    pub service_area: String,
    pub adn_vessel_type: String,
}

/// The Rule Trace of a liquid bulk layout, with the layout's result: the part [`liquid_trace`] and
/// [`liquid_fit`] share.
fn liquid_trace_raw(
    case: &LiquidTraceCase,
    library: &Library,
    chosen: &[(&str, &str)],
) -> Result<(Trace, ovcd_core::liquid_bulk::LiquidBulkResult), String> {
    let inputs = ovcd_core::liquid_bulk::LiquidBulkInputs::of(
        (case.loa_m, case.beam_m, case.depth_m),
        (case.zone_aft_m, case.zone_fwd_m),
        case.cargo_density_t_m3,
        &case.layout,
    );
    let liquid = ovcd_core::liquid_bulk::compute(&inputs).map_err(|e| e.message)?;
    let measured = Measured {
        liquid: Some(liquid.clone()),
        deadweight_t: Some(case.deadweight_t),
        ..Measured::default()
    };
    let selection = library.select(chosen)?;
    let numbers = [
        ("beam_m", case.beam_m),
        ("length_overall_m", case.loa_m),
        ("cofferdam_m", case.layout.cofferdam_m),
    ];
    let trace = evaluate(
        &selection.sets,
        &LiquidMeasures {
            measured: &measured,
            numbers: &numbers,
            service_area: &case.service_area,
            adn_vessel_type: &case.adn_vessel_type,
        },
    );
    Ok((trace, liquid))
}

/// The Rule Trace of a liquid bulk layout under the sets `chosen` of `library`, as `trace_json` writes
/// it. The harness compares its rows with a case's expected verdicts, keyed by rule id.
pub fn liquid_trace(
    case: &LiquidTraceCase,
    library: &Library,
    chosen: &[(&str, &str)],
) -> Result<Value, String> {
    let (trace, _) = liquid_trace_raw(case, library, chosen)?;
    Ok(trace_json(&trace, &case.service_area, &[]))
}

/// The JSON of a [`LiquidFit`], as the `liquid_fit` entry point of the evaluation API gives it.
pub fn liquid_fit_json(f: &LiquidFit) -> Value {
    let rows = |list: &[LiquidFitRow]| -> Value {
        Value::Array(
            list.iter()
                .map(|r| {
                    json!({
                        "rule": r.rule,
                        "measure": r.measure,
                        "object": r.object,
                        "value": r.value,
                        "limit": r.limit,
                    })
                })
                .collect(),
        )
    };
    let l = &f.layout;
    json!({
        "feasible": f.feasible,
        "sized": f.sized,
        "layout": {
            "double_side_m": l.double_side_m,
            "double_bottom_m": l.double_bottom_m,
            "tanks_in_line": l.tanks_in_line,
            "centreline_bulkhead": l.centreline_bulkhead,
            "cofferdam_m": l.cofferdam_m,
        },
        "cargo_volume_m3": f.cargo_volume_m3,
        "candidates_tried": f.candidates_tried,
        "fails": rows(&f.fails),
        "sizing_warnings": rows(&f.sizing_warnings),
        "not_evaluated": f.not_evaluated,
    })
}

// ---- `fit` for liquid bulk (issue #340, spec 0006 section 5) ---------------------------------------

/// The layout inputs a liquid bulk `fit` may choose. An input the designer typed is kept as it is.
#[derive(Debug, Clone, Copy, Default)]
pub struct LiquidTyped {
    pub double_side_m: bool,
    pub double_bottom_m: bool,
    pub tanks_in_line: bool,
    pub centreline_bulkhead: bool,
    pub cofferdam_m: bool,
}

/// What `fit` is asked: the hull and the layout. The layout holds the inputs `fit` does not choose (the
/// pump arrangement, the slop share, the fill ratio, the heating, k) and the value of each one it may
/// choose but that is `typed`.
#[derive(Debug, Clone)]
pub struct LiquidFitCase {
    pub case: LiquidTraceCase,
    pub typed: LiquidTyped,
}

/// A Rule row that a candidate did not meet.
#[derive(Debug, Clone, PartialEq)]
pub struct LiquidFitRow {
    pub rule: String,
    pub measure: String,
    pub object: Option<String>,
    pub value: Option<f64>,
    pub limit: Option<f64>,
}

/// The answer of [`liquid_fit`].
#[derive(Debug, Clone)]
pub struct LiquidFit {
    /// No row of the Rule Trace of `layout` fails.
    pub feasible: bool,
    /// Every `sizes` Rule of the Trace is met too (a `sizes` Rule that only warns is not a failure).
    pub sized: bool,
    pub layout: ovcd_core::liquid_bulk::LiquidLayout,
    pub cargo_volume_m3: f64,
    pub candidates_tried: usize,
    /// The rows that fail, for the best candidate. Empty when `feasible`.
    pub fails: Vec<LiquidFitRow>,
    /// The `sizes` rows that warn: a typed input, or a limit that no layout meets.
    pub sizing_warnings: Vec<LiquidFitRow>,
    /// The blocking Rules that were not evaluated, each once.
    pub not_evaluated: Vec<String>,
}

/// The most tank positions a liquid bulk `fit` tries. A tank is at least 1 m long, so the cargo zone
/// ends the search first on any real hull.
const FIT_MAX_TANKS: u32 = 30;
/// The widest a tank may be, over the beam, before a centreline bulkhead is needed (ADN 9.3.x.13 and
/// BV NR217, spec 0006 section 3.2).
const CENTRELINE_BREADTH_OVER_BEAM: f64 = 0.70;

fn fit_row(r: &TraceRow) -> LiquidFitRow {
    LiquidFitRow {
        rule: r.rule.clone().unwrap_or_default(),
        measure: r.measure.as_ref().map(|m| m.id.clone()).unwrap_or_default(),
        object: r.object.clone(),
        value: match &r.value {
            Some(MeasureValue::Number(v)) => Some(*v),
            _ => None,
        },
        limit: r.limit,
    }
}

/// A limit rounded up to a whole centimetre, so that a width at it meets the Rule ("greater than" too).
fn round_up_cm(limit: f64, strict: bool) -> f64 {
    let up = (limit * 100.0 - 1e-9).ceil() / 100.0;
    if strict && up <= limit { up + 0.01 } else { up }
}

/// The layout the `sizes` Rules of a Trace ask for, for the inputs that are not typed: the double side,
/// the double bottom and the cofferdam take the governing lower limit (the greatest of the limits of the
/// applicable `sizes` Rules on that Measure).
fn sized_layout(
    trace: &Trace,
    typed: &LiquidTyped,
    mut layout: ovcd_core::liquid_bulk::LiquidLayout,
) -> ovcd_core::liquid_bulk::LiquidLayout {
    let governing = |measure: &str| -> Option<f64> {
        trace
            .rows
            .iter()
            .filter(|r| r.sizes && r.measure.as_ref().is_some_and(|m| m.id == measure))
            .filter(|r| !matches!(r.outcome, Outcome::NotApplicable | Outcome::NotEvaluated))
            .filter_map(|r| {
                let limit = r.limit?;
                match r.compare? {
                    Compare::Ge => Some(round_up_cm(limit, false)),
                    Compare::Gt => Some(round_up_cm(limit, true)),
                    Compare::Le | Compare::Lt => None,
                }
            })
            .reduce(f64::max)
    };
    if !typed.double_side_m
        && let Some(v) = governing("hull.double_side_width")
    {
        layout.double_side_m = v;
    }
    if !typed.double_bottom_m
        && let Some(v) = governing("hull.double_bottom_height")
    {
        layout.double_bottom_m = v;
    }
    if !typed.cofferdam_m
        && let Some(v) = governing("input.cofferdam_m")
    {
        layout.cofferdam_m = v;
    }
    layout
}

/// `fit` for a liquid bulk hull: the double side and bottom from the governing `sizes` Rules, the
/// cofferdam likewise, a centreline bulkhead where a tank would be wider than 0.70 B, and the smallest
/// count of tanks in line that meets the tank volume and length limits. It tries every count, keeps only
/// layouts that no Rule fails, and among those prefers the layouts that meet every `sizes` Rule, then the
/// most cargo, then the fewest tanks. When no layout passes, the answer is not feasible and carries the
/// failing rows of the closest one. The Rules are those of `chosen`; a `warn` Rule never removes a
/// layout. Typed inputs are kept (`case.typed`), and a Rule they break shows in the answer.
pub fn liquid_fit(
    fit: &LiquidFitCase,
    library: &Library,
    chosen: &[(&str, &str)],
) -> Result<LiquidFit, String> {
    let mut best: Option<(LiquidFit, (usize, usize, f64, u32))> = None;
    let mut tried = 0;
    let counts: Vec<u32> = if fit.typed.tanks_in_line {
        vec![fit.case.layout.tanks_in_line]
    } else {
        (1..=FIT_MAX_TANKS).collect()
    };
    for n in counts {
        let mut case = fit.case.clone();
        case.layout.tanks_in_line = n;
        let mut result = None;
        // The limits of the Rules can depend on the layout (a tank's volume, the cofferdam): size, then
        // read the Trace again, until the layout stops moving.
        for _ in 0..4 {
            if !fit.typed.centreline_bulkhead {
                let inner = case.beam_m - 2.0 * case.layout.double_side_m;
                case.layout.centreline_bulkhead =
                    inner > CENTRELINE_BREADTH_OVER_BEAM * case.beam_m;
            }
            // A layout that does not fit the zone has no Trace: the count is skipped.
            let Ok((trace, liquid)) = liquid_trace_raw(&case, library, chosen) else {
                result = None;
                break;
            };
            let next = sized_layout(&trace, &fit.typed, case.layout);
            let settled = next.double_side_m == case.layout.double_side_m
                && next.double_bottom_m == case.layout.double_bottom_m
                && next.cofferdam_m == case.layout.cofferdam_m;
            result = Some((trace, liquid));
            if settled {
                break;
            }
            case.layout = next;
        }
        let Some((trace, liquid)) = result else {
            continue;
        };
        tried += 1;
        let fails: Vec<LiquidFitRow> = trace
            .rows
            .iter()
            .filter(|r| r.outcome == Outcome::Fail)
            .map(fit_row)
            .collect();
        let sizing_warnings: Vec<LiquidFitRow> = trace
            .rows
            .iter()
            .filter(|r| r.sizes && r.outcome == Outcome::Warn)
            .map(fit_row)
            .collect();
        let candidate = LiquidFit {
            feasible: fails.is_empty(),
            sized: fails.is_empty() && sizing_warnings.is_empty(),
            layout: case.layout,
            cargo_volume_m3: liquid.cargo_volume_m3,
            candidates_tried: 0,
            not_evaluated: trace
                .blocking_not_evaluated()
                .iter()
                .filter_map(|r| r.rule.clone())
                .collect(),
            fails,
            sizing_warnings,
        };
        // Lower is better: failing rows, sizing warnings, then more cargo, then fewer tanks.
        let key = (
            candidate.fails.len(),
            candidate.sizing_warnings.len(),
            -candidate.cargo_volume_m3,
            n,
        );
        let better = best.as_ref().is_none_or(|(_, k)| {
            (key.0, key.1) < (k.0, k.1)
                || ((key.0, key.1) == (k.0, k.1)
                    && (key.2 < k.2 - 1e-9 || (key.2 - k.2).abs() <= 1e-9 && key.3 < k.3))
        });
        if better {
            best = Some((candidate, key));
        }
    }
    let (mut answer, _) = best.ok_or_else(|| {
        "no layout fits the cargo zone: the zone is too short for its cofferdams and one tank"
            .to_string()
    })?;
    answer.candidates_tried = tried;
    Ok(answer)
}
