//! Rule Sets as plain typed data (spec 0004, ADR 0013). The loader ([`crate::load`]) reads a Rule
//! Set file and builds these structs; the evaluator ([`crate::eval`]) reads them. Nothing here
//! knows a domain: object types, Measures and inputs are names the caller declares.

use alloc::string::String;
use alloc::vec::Vec;

/// Where a Rule comes from. Only a published origin (`Regulation`, `Class`, `Guideline`,
/// `NationalPractice`) can ever be shown as a sourced rule; the other three are the tool's own
/// checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Regulation,
    Class,
    MethodValidity,
    Physical,
    ToolLimit,
    /// A design guideline that says it is not a standard (schema 1.1, additive).
    Guideline,
    /// A national standard known through another text, such as a guideline's survey (schema 1.1,
    /// additive).
    NationalPractice,
}

impl Origin {
    pub const ALL: [Origin; 7] = [
        Origin::Regulation,
        Origin::Class,
        Origin::MethodValidity,
        Origin::Physical,
        Origin::ToolLimit,
        Origin::Guideline,
        Origin::NationalPractice,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Origin::Regulation => "regulation",
            Origin::Class => "class",
            Origin::MethodValidity => "method-validity",
            Origin::Physical => "physical",
            Origin::ToolLimit => "tool-limit",
            Origin::Guideline => "guideline",
            Origin::NationalPractice => "national-practice",
        }
    }

    /// A published text (a statutory code, a class rule book, a guideline, a national standard):
    /// the Rule must cite a paragraph, and it is sourced once `verified` is filled.
    pub const fn is_published(self) -> bool {
        matches!(
            self,
            Origin::Regulation | Origin::Class | Origin::Guideline | Origin::NationalPractice
        )
    }
}

/// The form of a Rule's limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A constant.
    Limit,
    /// A row of one of the set's tables.
    Table,
    /// A formula shape the caller registers, with coefficients as data.
    Formula,
    /// A yes/no Measure; no comparison and no limit.
    Check,
    /// A cited value shown in the trace, with no test.
    Note,
}

impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::Limit,
        Kind::Table,
        Kind::Formula,
        Kind::Check,
        Kind::Note,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Kind::Limit => "limit",
            Kind::Table => "table",
            Kind::Formula => "formula",
            Kind::Check => "check",
            Kind::Note => "note",
        }
    }
}

/// The comparison of a Criterion. `Ge` and `Gt` are lower bounds, `Le` and `Lt` upper bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compare {
    Ge,
    Gt,
    Le,
    Lt,
}

impl Compare {
    pub const ALL: [Compare; 4] = [Compare::Ge, Compare::Gt, Compare::Le, Compare::Lt];

    pub const fn key(self) -> &'static str {
        match self {
            Compare::Ge => "ge",
            Compare::Gt => "gt",
            Compare::Le => "le",
            Compare::Lt => "lt",
        }
    }

    /// `true` for `Ge` and `Gt`.
    pub const fn is_lower_bound(self) -> bool {
        matches!(self, Compare::Ge | Compare::Gt)
    }

    /// `true` for `Gt` and `Lt`.
    pub const fn is_strict(self) -> bool {
        matches!(self, Compare::Gt | Compare::Lt)
    }
}

/// What happens when a Rule is not met.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consequence {
    Block,
    Warn,
    Note,
}

impl Consequence {
    pub const ALL: [Consequence; 3] = [Consequence::Block, Consequence::Warn, Consequence::Note];

    pub const fn key(self) -> &'static str {
        match self {
            Consequence::Block => "block",
            Consequence::Warn => "warn",
            Consequence::Note => "note",
        }
    }
}

/// An Applicability: a conjunction of `input ∈ {values}`. Empty means always. The conditions are
/// sorted by input name, each value list as written.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Applicability {
    pub conditions: Vec<(String, Vec<String>)>,
    /// The inputs of `conditions` that the caller's vocabulary gives per object
    /// ([`crate::domain::InputScope::Object`]), sorted. The evaluator reads them for each object of
    /// the Rule's type; the others once for the design. Filled by the loader; not in the content
    /// hash, as the file does not say it.
    pub object_scoped: Vec<String>,
}

impl Applicability {
    /// Whether some Option could meet both: for every input both name, their value lists share a
    /// value.
    pub fn overlaps(&self, other: &Applicability) -> bool {
        self.conditions.iter().all(|(input, values)| {
            match other.conditions.iter().find(|(i, _)| i == input) {
                Some((_, theirs)) => values.iter().any(|v| theirs.contains(v)),
                None => true,
            }
        })
    }
}

/// A Measure argument: a number (an angle) or a text (a material class).
#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    Number(f64),
    Text(String),
}

/// A Measure instance: a Measure id with its arguments bound, sorted by name.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRef {
    pub id: String,
    pub args: Vec<(String, ArgValue)>,
}

/// An operand of a formula limit: another Measure's value, or a table row picked by a Measure value.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Measure(String),
    TableAt { table: String, at: String },
}

/// The limit of a Rule.
#[derive(Debug, Clone, PartialEq)]
pub enum Limit {
    /// `kind: limit`.
    Constant { value: f64, unit: String },
    /// `kind: table`: the row of `table` picked by the value of `at` (a Measure id).
    Table {
        table: String,
        at: String,
        unit: String,
    },
    /// `kind: formula`: a shape the caller registers ([`crate::domain::FormulaShape`]), with its
    /// coefficients and operands, each sorted by name.
    Formula {
        shape: String,
        unit: String,
        coefficients: Vec<(String, f64)>,
        operands: Vec<(String, Operand)>,
    },
}

/// Who wrote a Rule record. `by` is a role, never a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Codified {
    pub by: String,
    pub date: String,
    pub review: String,
}

/// A second reading against the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    pub by: String,
    pub date: String,
    pub edition_held: String,
    pub locator_read: String,
    pub review: String,
}

/// The citation of a Rule.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Citation {
    pub document: String,
    pub edition: String,
    /// The paragraph, as printed. Empty for a tool check with no source.
    pub locator: String,
    /// Other paragraphs the requirement reads with.
    pub also: Vec<String>,
    pub page: String,
    /// `free`, `paywalled` or `sign-up`; empty for a tool check.
    pub access: String,
}

/// One Rule (spec 0004, "Rule (data)").
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub origin: Origin,
    pub source: Citation,
    pub codified: Codified,
    /// `None` until a second reader verifies the Rule.
    pub verified: Option<Verification>,
    pub applies_when: Applicability,
    pub kind: Kind,
    pub measure: MeasureRef,
    /// `None` for `check` and `note` Rules.
    pub compare: Option<Compare>,
    /// `None` for `check` and `note` Rules.
    pub limit: Option<Limit>,
    pub tolerance: f64,
    pub consequence: Consequence,
    pub sizes: bool,
    pub evaluated: bool,
    /// What the engine lacks, for a Rule that is not evaluated.
    pub gap: Option<String>,
    pub finding: Option<String>,
    pub since: String,
}

impl Rule {
    /// The family segment of the id.
    pub fn family(&self) -> &str {
        self.id.split('/').next().unwrap_or("")
    }

    /// The paragraph segment of the id.
    pub fn paragraph(&self) -> &str {
        self.id.split('/').nth(1).unwrap_or("")
    }
}

/// A bound of a table row: `value` compared with the table's input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bound {
    pub compare: Compare,
    pub value: f64,
}

/// When a table row applies.
#[derive(Debug, Clone, PartialEq)]
pub enum RowWhen {
    /// Numeric bounds on the input, all of which hold.
    Bounds(Vec<Bound>),
    /// The input is one of these texts (a material class).
    OneOf(Vec<String>),
}

/// One row of a table: when it applies, and its value as a shape with coefficients.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub when: RowWhen,
    pub shape: String,
    pub coefficients: Vec<(String, f64)>,
}

/// A step or piecewise table that a Rule's limit names.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub id: String,
    pub source: Citation,
    pub input: String,
    /// The unit of the input; empty for a text input.
    pub input_unit: String,
    pub unit: String,
    pub rows: Vec<TableRow>,
}

/// A set this one requires, at a pinned version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Include {
    pub id: String,
    pub version: String,
}

/// Two Rules of a set that no value can meet together, declared with a reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownConflict {
    pub rules: [String; 2],
    pub reason: String,
}

/// A source text a set was codified from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub document: String,
    pub edition: String,
    pub url: String,
    pub access: String,
    pub sha256: String,
}

/// `draft` or `stable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetStatus {
    Draft,
    Stable,
}

impl SetStatus {
    pub const fn key(self) -> &'static str {
        match self {
            SetStatus::Draft => "draft",
            SetStatus::Stable => "stable",
        }
    }
}

/// One version of a Rule Set (spec 0004, "Rule Set (data)").
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSet {
    pub id: String,
    pub family: String,
    pub title: String,
    pub version: String,
    /// The source edition, as named fields (document, edition, amendments, in force).
    pub edition: Vec<(String, String)>,
    pub sources: Vec<Source>,
    pub status: SetStatus,
    pub applies_when: Applicability,
    pub includes: Vec<Include>,
    pub known_conflicts: Vec<KnownConflict>,
    pub tables: Vec<Table>,
    pub rules: Vec<Rule>,
    /// The content hash (spec 0004, "Content hash"), filled in by the loader.
    pub hash: String,
}

impl RuleSet {
    /// `id@version`.
    pub fn key(&self) -> String {
        alloc::format!("{}@{}", self.id, self.version)
    }

    pub fn rule(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }

    pub fn table(&self, id: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.id == id)
    }
}
