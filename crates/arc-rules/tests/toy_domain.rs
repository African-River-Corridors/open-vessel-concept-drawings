//! Label: sourced - hand calculation of the toy Rule Set below (invented numbers, no real rule book)
//!
//! The minimal non-vessel example of the README: a toy domain of stairs in a building. It shows
//! each extension point: object keys (`building`, `stair`), a Measure registry, an applicability
//! vocabulary with a design-wide input (`use`) and an object-scoped one (`location`), and one
//! formula shape the caller registers (`scaled`).

use arc_rules::domain::{
    ArgDef, Domain, FormulaShape, InputDef, InputScope, MeasureInfo, MethodStatus, ValueKind,
    coefficient,
};
use arc_rules::eval::{MeasureSource, MeasureValue, Outcome, evaluate};
use arc_rules::load::{Library, load};
use arc_rules::model::{MeasureRef, Origin};

/// The toy domain: a building and its stairs.
struct Stairs;

fn info(object: &str, unit: &str) -> MeasureInfo {
    MeasureInfo {
        unit: unit.to_string(),
        object: object.to_string(),
        value: ValueKind::Number,
        args: Vec::<ArgDef>::new(),
        method_status: MethodStatus::Verified,
    }
}

fn scaled(
    shape: &str,
    coefficients: &[(String, f64)],
    operand: &mut dyn FnMut(&str) -> Result<f64, String>,
) -> Result<f64, String> {
    // `x * a`: a limit that grows with another Measure.
    Ok(operand("x")? * coefficient(shape, coefficients, "a")?)
}

static SHAPES: &[FormulaShape] = &[FormulaShape {
    id: "scaled",
    operands: &["x"],
    coefficients: &["a"],
    free_unit_operands: &["x"],
    value: scaled,
}];

impl Domain for Stairs {
    fn measure(&self, id: &str) -> Option<MeasureInfo> {
        match id {
            "stair.riser" => Some(info("stair", "mm")),
            "stair.going" => Some(info("stair", "mm")),
            "building.storeys" => Some(info("building", "-")),
            _ => None,
        }
    }

    fn input(&self, name: &str) -> Option<InputDef> {
        let (values, scope): (&[&str], InputScope) = match name {
            "use" => (&["home", "public"], InputScope::Design),
            "location" => (&["inside", "outside"], InputScope::Object),
            _ => return None,
        };
        Some(InputDef {
            values: values.iter().map(|v| v.to_string()).collect(),
            scope,
        })
    }

    fn shapes(&self) -> &[FormulaShape] {
        SHAPES
    }

    fn design_object(&self) -> &str {
        "building"
    }
}

/// A public building with two stairs: one inside, one outside.
struct Building;

impl MeasureSource for Building {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        Stairs.measure(id)
    }

    fn objects(&self, object: &str) -> Vec<String> {
        match object {
            "building" => vec!["Library".into()],
            "stair" => vec!["Main stair".into(), "Garden steps".into()],
            _ => Vec::new(),
        }
    }

    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        let v = match (m.id.as_str(), index) {
            ("stair.riser", 0) => 175.0,
            ("stair.riser", 1) => 160.0,
            ("stair.going", 0) => 260.0,
            ("stair.going", 1) => 300.0,
            ("building.storeys", _) => 3.0,
            _ => return Err(format!("no `{}` on object {index}", m.id)),
        };
        Ok(MeasureValue::Number(v))
    }

    fn input(&self, name: &str) -> Option<String> {
        (name == "use").then(|| "public".to_string())
    }

    fn object_input(&self, object: &str, index: usize, name: &str) -> Option<String> {
        match (object, name) {
            ("stair", "location") => Some(["inside", "outside"][index].to_string()),
            _ => self.input(name),
        }
    }
}

const SET: &str = r#"{
  "id": "toy-stairs",
  "family": "toy",
  "title": "Toy stair rules (invented numbers, for the example only)",
  "version": "1.0.0",
  "edition": {"document": "Toy guideline, invented for this example", "edition": "none"},
  "sources": [],
  "status": "draft",
  "applies_when": {"use": ["public"]},
  "rules": [
    {
      "id": "toy/1.1/riser",
      "title": "Riser height at most 180 mm",
      "origin": "guideline",
      "source": {"document": "Toy guideline", "edition": "none", "locator": "1.1"},
      "codified": {"by": "agent", "date": "2026-10-21", "review": "example"},
      "verified": false,
      "kind": "limit",
      "measure": {"id": "stair.riser"},
      "compare": "le",
      "limit": {"value": 180, "unit": "mm"},
      "consequence": "block",
      "evaluated": true,
      "since": "1.0.0"
    },
    {
      "id": "toy/1.2/riser-outside",
      "title": "Outside, riser height at most 150 mm",
      "origin": "national-practice",
      "source": {"document": "Toy guideline", "edition": "none", "locator": "1.2"},
      "codified": {"by": "agent", "date": "2026-10-21", "review": "example"},
      "verified": false,
      "applies_when": {"location": ["outside"]},
      "kind": "limit",
      "measure": {"id": "stair.riser"},
      "compare": "le",
      "limit": {"value": 150, "unit": "mm"},
      "consequence": "block",
      "evaluated": true,
      "since": "1.0.0"
    },
    {
      "id": "toy/2.1/going",
      "title": "Going at least 1.5 times the riser",
      "origin": "physical",
      "source": {},
      "codified": {"by": "agent", "date": "2026-10-21", "review": "example"},
      "verified": false,
      "kind": "formula",
      "measure": {"id": "stair.going"},
      "compare": "ge",
      "limit": {"shape": "scaled", "unit": "mm", "x": "stair.riser", "a": 1.5},
      "consequence": "warn",
      "evaluated": true,
      "since": "1.0.0"
    }
  ]
}"#;

#[test]
fn the_toy_domain_loads_and_evaluates() {
    let loaded = load(SET, &Stairs).expect("the toy set loads");
    let set = &loaded.set;
    assert_eq!(set.rules[0].origin, Origin::Guideline);
    assert_eq!(set.rules[1].origin, Origin::NationalPractice);
    assert_eq!(set.rules[1].applies_when.object_scoped, vec!["location"]);
    assert_eq!(set.hash.len(), 64);

    let trace = evaluate(&[set], SHAPES, &Building);
    let rows: Vec<(&str, &str, Outcome)> = trace
        .rows
        .iter()
        .map(|r| {
            (
                r.rule.as_deref().unwrap(),
                r.object.as_deref().unwrap(),
                r.outcome,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("toy/1.1/riser", "Main stair", Outcome::Pass),
            ("toy/1.1/riser", "Garden steps", Outcome::Pass),
            // The outside Rule does not reach the inside stair.
            (
                "toy/1.2/riser-outside",
                "Main stair",
                Outcome::NotApplicable
            ),
            // 160 mm > 150 mm.
            ("toy/1.2/riser-outside", "Garden steps", Outcome::Fail),
            // 260 mm < 1.5 x 175 mm = 262.5 mm: a warning.
            ("toy/2.1/going", "Main stair", Outcome::Warn),
            ("toy/2.1/going", "Garden steps", Outcome::Pass),
        ]
    );
    assert_eq!(trace.rows[4].limit, Some(262.5));
    assert_eq!(trace.rows[0].object_type.as_deref(), Some("stair"));
    // On the garden steps the outside Rule is stricter, so it governs.
    assert!(trace.rows[3].governing && !trace.rows[1].governing);
    assert!(!trace.passes());
}

#[test]
fn the_loader_refuses_names_the_domain_does_not_declare() {
    let typo = SET.replace(r#""location": ["outside"]"#, r#""location": ["outdoors"]"#);
    let e = load(&typo, &Stairs).unwrap_err();
    assert!(e.contains("`applies_when.location` has `outdoors`"), "{e}");

    let unknown = SET.replace(r#""id": "stair.going""#, r#""id": "stair.width""#);
    let e = load(&unknown, &Stairs).unwrap_err();
    assert!(e.contains("unknown Measure `stair.width`"), "{e}");

    // A set applies to a whole design: no object-scoped input in its own Applicability.
    let set_level = SET.replace(r#""use": ["public"]"#, r#""location": ["inside"]"#);
    let e = load(&set_level, &Stairs).unwrap_err();
    assert!(e.contains("a set applies to a whole design"), "{e}");

    // A guideline Rule cites its paragraph, as a regulation Rule does.
    let uncited = SET.replace(r#""locator": "1.1""#, r#""locator": """#);
    let e = load(&uncited, &Stairs).unwrap_err();
    assert!(e.contains("must cite its paragraph"), "{e}");
}

#[test]
fn a_design_the_set_does_not_apply_to_gets_one_row() {
    struct Home;
    impl MeasureSource for Home {
        fn def(&self, id: &str) -> Option<MeasureInfo> {
            Stairs.measure(id)
        }
        fn objects(&self, object: &str) -> Vec<String> {
            Building.objects(object)
        }
        fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
            Building.value(m, index)
        }
        fn input(&self, name: &str) -> Option<String> {
            (name == "use").then(|| "home".to_string())
        }
    }
    let mut lib = Library::default();
    lib.add_text(SET, &Stairs).unwrap();
    let selection = lib.select(&[("toy-stairs", "1.0.0")]).unwrap();
    let trace = evaluate(&selection.sets, SHAPES, &Home);
    assert_eq!(trace.rows.len(), 1);
    assert_eq!(trace.rows[0].rule, None);
    assert_eq!(trace.rows[0].outcome, Outcome::NotApplicable);
}
