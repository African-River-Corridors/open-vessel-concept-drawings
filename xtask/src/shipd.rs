//! The ShipD benchmark (spec 0008, "Benchmark against the dataset", issue #452).
//!
//! `shipd-sample` draws, with a fixed seed, 100 hulls from each of the Ship-D dataset's Constrained
//! Randomized Sets 2 and 3 (no bulbs) and Set 1 (a bulb at the bow, the stern or both) that pass
//! the constraints, and writes their input vectors and the dataset's measures at its ten draughts
//! to `calibration/shipd/`. `shipd-benchmark` builds each
//! sampled hull's Sections Table at L = 10 m, measures it at the same draughts and writes the
//! comparison to `calibration/shipd/report.md`.

use crate::{Result, root};
use ovd_core::shipd::{
    BENCHMARK, DESIGN, DRAUGHT_FRACTIONS, FilletRow, Measures, NAMES, Reading, Resolution,
    ShipdParameters, check, dataset_measures, measures, sections,
};
use std::fmt::Write as _;
use std::path::Path;

/// The sample's seed and size per set.
const SEED: u64 = 452;
const PER_SET: usize = 100;
const SETS: [&str; 3] = [
    "Constrained_Randomized_Set_2",
    "Constrained_Randomized_Set_3",
    "Constrained_Randomized_Set_1",
];

/// Set 1 is sampled among the hulls with a bulb; the others among the hulls without one.
fn with_bulb(set: &str) -> bool {
    set.ends_with("Set_1")
}
/// The dataset's measure files, in the order of the `Measures` fields.
const QUANTITIES: [&str; 7] = ["Volume", "Area_WP", "LCB", "LCF", "Ixx", "Iyy", "WL_Length"];

/// Pass limits: volume, waterplane area and waterline length 1 % (relative); LCB and LCF 0.5 % of
/// LOA; Ixx and Iyy 2 %.
const LIMITS: [(f64, bool); 7] = [
    (0.01, true),
    (0.01, true),
    (0.005, false),
    (0.005, false),
    (0.02, true),
    (0.02, true),
    (0.01, true),
];

fn folder() -> std::path::PathBuf {
    root().join("calibration").join("shipd")
}

fn read_csv(path: &Path) -> Result<Vec<Vec<String>>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|c| c.trim().to_string()).collect())
        .collect())
}

fn number(cell: &str) -> Result<f64> {
    cell.parse().map_err(|_| format!("not a number: {cell:?}"))
}

/// A small fixed-seed generator (xorshift64*), so the sample does not depend on a library.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

pub(crate) fn sample(args: &[String]) -> Result {
    let dataset = args
        .first()
        .ok_or("give the folder that holds Ship_D_Dataset's set folders")?;
    let mut sample = String::from("set,row");
    for n in NAMES {
        sample.push(',');
        sample.push_str(n);
    }
    sample.push('\n');
    let mut reference = String::from("set,row,quantity");
    for f in DRAUGHT_FRACTIONS {
        let _ = write!(reference, ",T/Dd = {f:.2}");
    }
    reference.push('\n');
    let mut rng = Rng(SEED);
    for set in SETS {
        let dir = Path::new(dataset).join(set);
        let inputs = read_csv(&dir.join("Input_Vectors.csv"))?;
        let header = &inputs[0];
        if header.len() != 45 || header.iter().zip(NAMES).any(|(a, b)| a != b) {
            return Err(format!("{set}: the input header is not the 45 ShipD names"));
        }
        let rows: Vec<&Vec<String>> = inputs[1..].iter().collect();
        let tables: Vec<Vec<Vec<String>>> = QUANTITIES
            .iter()
            .map(|q| read_csv(&dir.join(format!("{q}.csv"))))
            .collect::<Result<_>>()?;
        // A shuffled order of the rows; the first that pass, in that order.
        let mut order: Vec<usize> = (0..rows.len()).collect();
        for i in (1..order.len()).rev() {
            let j = (rng.next() % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        let mut taken = 0;
        let (mut seen, mut failed, mut only_vertex) = (0, 0, 0);
        for &r in &order {
            let v: Vec<f64> = rows[r].iter().map(|c| number(c)).collect::<Result<_>>()?;
            let p = ShipdParameters(v.clone().try_into().map_err(|_| "45 values")?);
            let (loa, beam, depth) = (v[0], v[3] * v[0], v[4] * v[0]);
            if (v[31] != 0.0 || v[32] != 0.0) != with_bulb(set) {
                continue;
            }
            seen += 1;
            let failures = check(&p, loa, beam, depth);
            if !failures.is_empty() {
                failed += 1;
                // The dataset's code has the sign of this row the other way round.
                only_vertex += usize::from(failures.iter().all(|f| f.index == 40));
                continue;
            }
            let _ = write!(sample, "{set},{r}");
            for c in rows[r] {
                let _ = write!(sample, ",{c}");
            }
            sample.push('\n');
            for (q, t) in QUANTITIES.iter().zip(&tables) {
                let _ = write!(reference, "{set},{r},{q}");
                for c in &t[r + 1] {
                    let _ = write!(reference, ",{c}");
                }
                reference.push('\n');
            }
            taken += 1;
            if taken == PER_SET {
                break;
            }
        }
        println!(
            "{set}: {taken} taken after {seen} hulls of the kind; {failed} of them fail the \
             constraints, {only_vertex} of those on index 40 only"
        );
        if taken < PER_SET {
            return Err(format!("{set}: only {taken} hulls pass the constraints"));
        }
    }
    let out = folder();
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    std::fs::write(out.join("sample.csv"), sample).map_err(|e| e.to_string())?;
    std::fs::write(out.join("reference.csv"), reference).map_err(|e| e.to_string())?;
    println!("wrote calibration/shipd/sample.csv and reference.csv");
    Ok(())
}

/// One sampled hull: its set, row, parameters and the dataset's measures per draught.
pub(crate) struct Hull {
    pub set: String,
    pub row: String,
    pub p: ShipdParameters,
    pub reference: Vec<Measures>,
}

pub(crate) fn load(dir: &Path) -> Result<Vec<Hull>> {
    let sample = read_csv(&dir.join("sample.csv"))?;
    let reference = read_csv(&dir.join("reference.csv"))?;
    let mut hulls = Vec::new();
    for row in &sample[1..] {
        let v: Vec<f64> = row[2..].iter().map(|c| number(c)).collect::<Result<_>>()?;
        let p = ShipdParameters(v.try_into().map_err(|_| "45 values")?);
        let of = |q: &str| -> Result<Vec<f64>> {
            let r = reference[1..]
                .iter()
                .find(|r| r[0] == row[0] && r[1] == row[1] && r[2] == q)
                .ok_or(format!("no {q} for {} {}", row[0], row[1]))?;
            r[3..].iter().map(|c| number(c)).collect()
        };
        let cols: Vec<Vec<f64>> = QUANTITIES.iter().map(|q| of(q)).collect::<Result<_>>()?;
        let reference = (0..DRAUGHT_FRACTIONS.len())
            .map(|k| Measures {
                volume: cols[0][k],
                waterplane_area: cols[1][k],
                lcb: cols[2][k],
                lcf: cols[3][k],
                ixx: cols[4][k],
                iyy: cols[5][k],
                waterline_length: cols[6][k],
            })
            .collect();
        hulls.push(Hull {
            set: row[0].clone(),
            row: row[1].clone(),
            p,
            reference,
        });
    }
    Ok(hulls)
}

fn fields(m: &Measures) -> [f64; 7] {
    [
        m.volume,
        m.waterplane_area,
        m.lcb,
        m.lcf,
        m.ixx,
        m.iyy,
        m.waterline_length,
    ]
}

/// The hull's measures at the dataset's draughts, at cut `r`.
fn measured(h: &Hull, r: Resolution) -> Result<Vec<Measures>> {
    let v = &h.p.0;
    let (loa, beam, depth) = (v[0], v[3] * v[0], v[4] * v[0]);
    let table = sections(&h.p, loa, beam, depth, r).map_err(|e| e.to_string())?;
    // At T/Dd = 1 the waterplane is the deck itself, where the closed section has no waterline:
    // the deck-level values are taken a hair below it.
    let draughts: Vec<f64> = DRAUGHT_FRACTIONS
        .iter()
        .map(|f| f.min(1.0 - 1e-9) * depth)
        .collect();
    Ok(measures(&table, loa, &draughts))
}

/// The hull's measures taken as the dataset takes its own (`dataset_measures`), with readings `r`.
fn measured_as_dataset(h: &Hull, r: Resolution) -> Vec<Measures> {
    let v = &h.p.0;
    dataset_measures(&h.p, v[0], v[3] * v[0], v[4] * v[0], r)
}

/// The worst difference of one hull per quantity, as a share of its limit (1 = at the limit), and
/// the draught where it is.
pub(crate) fn compare(h: &Hull, r: Resolution) -> Result<[(f64, f64, f64, f64); 7]> {
    Ok(worst_of(h, &measured(h, r)?))
}

fn worst_of(h: &Hull, ours: &[Measures]) -> [(f64, f64, f64, f64); 7] {
    let mut worst = [(0.0, 0.0, 0.0, 0.0); 7];
    for (k, (o, r)) in ours.iter().zip(&h.reference).enumerate() {
        for (q, (a, b)) in fields(o).iter().zip(fields(r)).enumerate() {
            let (limit, relative) = LIMITS[q];
            let diff = if relative {
                (a - b).abs() / b.abs()
            } else {
                (a - b).abs()
            };
            let share = diff / limit;
            if share > worst[q].0 || share.is_nan() {
                worst[q] = (share, DRAUGHT_FRACTIONS[k], *a, b);
            }
        }
    }
    worst
}

/// A hull the dataset's code measures off its own definitions, and why (`artefacts.csv`).
struct Artefact {
    set: String,
    row: String,
    cause: String,
}

/// The two places the dataset's code departs from the definitions, as a reading of the generator.
fn reading_of(cause: &str) -> Result<Resolution> {
    match cause {
        "stern_ellipse" => Ok(Resolution {
            stern_ellipse: Reading::DatasetCode,
            ..BENCHMARK
        }),
        "bulb_crossing" => Ok(Resolution {
            bulb_crossing: Reading::DatasetCode,
            ..BENCHMARK
        }),
        _ => Err(format!("artefacts.csv: unknown cause {cause:?}")),
    }
}

fn artefacts(dir: &Path) -> Result<Vec<Artefact>> {
    Ok(read_csv(&dir.join("artefacts.csv"))?[1..]
        .iter()
        .map(|r| Artefact {
            set: r[0].clone(),
            row: r[1].clone(),
            cause: r[2].clone(),
        })
        .collect())
}

/// What each artefact cause is, for the report.
const CAUSES: [(&str, &str); 2] = [
    (
        "stern_ellipse",
        "The stern taper is a quarter ellipse (`bit_EP_T` or `bit_EP_S` = 1) and the transom is wider \
         than the midbody at that height. The definitions run the ellipse from the midbody \
         half-breadth to the transom's. The dataset's code squares its vertical radius (`Ry = y_mid - \
         y_transom`, here negative) inside the root, so its ellipse is mirrored: it starts at \
         `2 y_transom - y_mid`, a step out from the midbody, and falls to the transom.",
    ),
    (
        "bulb_crossing",
        "Where the bow bulb meets the hull, the dataset's code takes the smallest root of the bow \
         cubic that lies aft of the stem and of the bulb's front. When the real crossing lies forward \
         of the bulb's front, the root it takes is the real part of a complex root (numpy orders a \
         complex number by its real part first) or a root aft of `delta_bow`. Its waterline then runs \
         at the bulb's width to `delta_bow`: wider than the hull where the bulb is wider, or one \
         waterline just below the bulb's top with up to 30 % less area than its neighbours.",
    ),
];

/// The gate: each hull's shape, measured as the dataset measures (`dataset_measures`), within the
/// spec's limits. A hull outside passes the gate only when `artefacts.csv` lists it and its cause,
/// read as the dataset's code reads it, brings it within the limits. Gives the pass table, the
/// artefact table and the problems that fail the gate.
fn shape_gate(hulls: &[Hull], listed: &[Artefact]) -> Result<(String, String, Vec<String>)> {
    let mut rows = String::new();
    let mut found = String::new();
    let mut problems = Vec::new();
    for set in SETS {
        let of_set: Vec<&Hull> = hulls.iter().filter(|h| h.set == set).collect();
        let (mut pass, mut excused) = (0, 0);
        let mut per_q = [0usize; 7];
        for h in &of_set {
            let w = worst_of(h, &measured_as_dataset(h, BENCHMARK));
            let inside = w.iter().all(|x| x.0 <= 1.0);
            for (q, x) in w.iter().enumerate() {
                if x.0 <= 1.0 {
                    per_q[q] += 1;
                }
            }
            let entry = listed.iter().find(|a| a.set == set && a.row == h.row);
            match (inside, entry) {
                (true, None) => pass += 1,
                (true, Some(a)) => problems.push(format!(
                    "{set} {}: listed as an artefact ({}) but within the limits: remove it from artefacts.csv",
                    h.row, a.cause
                )),
                (false, None) => problems.push(format!(
                    "{set} {}: outside the limits and not a listed artefact: {}",
                    h.row,
                    misses(&w)
                )),
                (false, Some(a)) => {
                    let wa = worst_of(h, &measured_as_dataset(h, reading_of(&a.cause)?));
                    let reproduced = wa.iter().all(|x| x.0 <= 1.0);
                    let worst = wa.iter().fold(0.0_f64, |m, x| m.max(x.0));
                    let _ = writeln!(
                        found,
                        "| {set} | {} | `{}` | {} | {} (worst {worst:.2} x the limit) |",
                        h.row,
                        a.cause,
                        misses(&w),
                        if reproduced { "within the limits" } else { "**still outside**" }
                    );
                    if reproduced {
                        excused += 1;
                    } else {
                        problems.push(format!(
                            "{set} {}: the dataset's code reading of `{}` does not reproduce it: {}",
                            h.row,
                            a.cause,
                            misses(&wa)
                        ));
                    }
                }
            }
        }
        let n = of_set.len();
        let _ = write!(rows, "| {set} | {n} | {pass} | {excused} |");
        for c in per_q {
            let _ = write!(rows, " {c} |");
        }
        rows.push('\n');
    }
    let mut head = String::from("| Set | Hulls | Pass | Dataset artefact | ");
    head.push_str(&QUANTITIES.join(" | "));
    head.push_str(" |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
    Ok((head + &rows, found, problems))
}

/// The quantities outside the limit, with their draught and size.
fn misses(w: &[(f64, f64, f64, f64); 7]) -> String {
    w.iter()
        .enumerate()
        .filter(|(_, x)| x.0 > 1.0 || x.0.is_nan())
        .map(|(q, x)| {
            format!(
                "{} at T/Dd {:.2}: OVD {:.6}, dataset {:.6} ({:.2} x the limit)",
                QUANTITIES[q], x.1, x.2, x.3, x.0
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The pass table and the list of hulls outside, at `levels`.
/// The pass table, the list of hulls outside and the (pass, hulls) count of each set.
type Table = (String, String, Vec<(usize, usize)>);

fn table(hulls: &[Hull], r: Resolution) -> Result<Table> {
    let mut outside = String::new();
    let mut rows = String::new();
    let mut counts = Vec::new();
    for set in SETS {
        let of_set: Vec<&Hull> = hulls.iter().filter(|h| h.set == set).collect();
        let mut pass = 0;
        let mut per_q = [0usize; 7];
        for h in &of_set {
            let w = compare(h, r)?;
            if w.iter().all(|x| x.0 <= 1.0) {
                pass += 1;
            } else {
                let _ = write!(outside, "| {set} | {} |", h.row);
                let mut draughts = Vec::new();
                for (q, x) in w.iter().enumerate() {
                    if x.0 > 1.0 {
                        draughts.push(x.1);
                        let _ = write!(
                            outside,
                            " {} at T/Dd {:.2}: OVD {:.6}, dataset {:.6} ({:.2} x the limit);",
                            QUANTITIES[q], x.1, x.2, x.3, x.0
                        );
                    }
                }
                // Where in the draught range the hull is outside.
                let at = if draughts.iter().all(|d| *d <= 0.25) {
                    "low draught: the immersed volume is small"
                } else if draughts.iter().all(|d| *d >= 1.0) {
                    "deck level: the waterplane at the deck"
                } else {
                    "between"
                };
                let _ = writeln!(outside, " | {at} |");
            }
            for (q, x) in w.iter().enumerate() {
                if x.0 <= 1.0 {
                    per_q[q] += 1;
                }
            }
        }
        let n = of_set.len();
        let _ = write!(
            rows,
            "| {set} | {n} | {pass} ({:.0} %) |",
            100.0 * pass as f64 / n as f64
        );
        for c in per_q {
            let _ = write!(rows, " {c} |");
        }
        rows.push('\n');
        counts.push((pass, n));
    }
    let mut head = String::from("| Set | Hulls | Pass | ");
    head.push_str(&QUANTITIES.join(" | "));
    head.push_str(" |\n|---|---|---|---|---|---|---|---|---|---|\n");
    Ok((head + &rows, outside, counts))
}

/// The share of each bulb hull's volume that the dataset's curvature row moves, and the hulls
/// where it changes whether the hull passes.
fn fillet_section(hulls: &[Hull]) -> Result<String> {
    let literal = Resolution {
        fillet_row: FilletRow::DatasetCode,
        ..BENCHMARK
    };
    let mut moved = 0;
    let mut n = 0;
    let mut rows = String::new();
    let mut worst = 0.0_f64;
    for h in hulls.iter().filter(|h| with_bulb(&h.set)) {
        n += 1;
        let (a, b) = (measured(h, BENCHMARK)?, measured(h, literal)?);
        let diff = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x.volume - y.volume).abs() / y.volume.abs())
            .fold(0.0_f64, f64::max);
        worst = worst.max(diff);
        let (wa, wb) = (compare(h, BENCHMARK)?, compare(h, literal)?);
        let (pa, pb) = (wa.iter().all(|x| x.0 <= 1.0), wb.iter().all(|x| x.0 <= 1.0));
        if diff > 1e-3 {
            moved += 1;
        }
        if diff > 1e-3 || pa != pb {
            let _ = writeln!(
                rows,
                "| {} | {} | {:.3} % | {} | {} |",
                h.set,
                h.row,
                100.0 * diff,
                if pa { "pass" } else { "outside" },
                if pb { "pass" } else { "outside" }
            );
        }
    }
    let mut out = format!(
        "\n## The fillet row\n\nThe quartic fillet between a bulb and the hull has one curvature \
         condition. OVD writes it from the definition (no curvature at the bulb end). The dataset's \
         code writes `6 x^2` where the second derivative has `6 x`. Of the {n} bulb hulls, {moved} \
         differ in volume by more than 0.1 % between the two rows (largest {:.3} %); each is listed, \
         with whether it passes under each row.\n\n",
        100.0 * worst
    );
    if rows.is_empty() {
        out.push_str("None differ.\n");
    } else {
        out.push_str(
            "| Set | Row | Volume difference | Definition | Dataset row |\n|---|---|---|---|---|\n",
        );
        out.push_str(&rows);
    }
    Ok(out)
}

pub(crate) fn benchmark() -> Result {
    let dir = folder();
    let hulls = load(&dir)?;
    let listed = artefacts(&dir)?;
    let mut report = String::from(
        "# ShipD benchmark\n\nWritten by `cargo run -p xtask -- shipd-benchmark` (issues #452, #453 and #648, \
         spec 0008 \"Benchmark against the dataset\"). The sample and its source are in `SOURCE.md`.\n\n\
         The benchmark compares OVD's ShipD-style hull with the dataset's measures at its ten draughts \
         (T/Dd 0.12 to 1.0), at L = 10 m. A hull passes when, at every draught, its volume, waterplane area \
         and waterline length are within 1 %, its LCB and LCF within 0.5 % of LOA, and its Ixx and Iyy \
         within 2 % of the dataset's. The tolerances are not loosened to make a hull pass.\n\n\
         ## The gate: OVD's shape, measured as the dataset measures\n\n\
         The dataset took its measures from 101 waterlines with 1000 points each, by the trapezoid rule, \
         and interpolated between waterlines. The gate measures OVD's shape the same way \
         (`ovd_core::shipd::dataset_measures`), so it compares the shape, not the integration rule. \
         Every hull must pass, except a hull listed in `artefacts.csv`: there the dataset's code departs \
         from the definitions, and the gate requires that reading that one place as the code does brings \
         the hull within the limits. Any other hull outside fails the gate.\n\n",
    );
    let (rows, found, problems) = shape_gate(&hulls, &listed)?;
    report.push_str(&rows);
    report.push_str(
        "\nThe last seven columns count the hulls within the limit for that quantity at every draught.\n\n\
         ## Hulls outside: the dataset's code\n\n",
    );
    for (cause, text) in CAUSES {
        let _ = writeln!(report, "- `{cause}`: {text}");
    }
    if found.is_empty() {
        report.push_str("\nNone.\n");
    } else {
        report.push_str(
            "\n| Set | Row | Cause | Outside, by the definitions | Read as the dataset's code |\n|---|---|---|---|---|\n",
        );
        report.push_str(&found);
    }
    report.push_str("\n## Hulls outside for any other reason\n\n");
    if problems.is_empty() {
        report.push_str("None.\n");
    } else {
        for p in &problems {
            let _ = writeln!(report, "- {p}");
        }
    }
    let (rows, outside, _) = table(&hulls, BENCHMARK)?;
    let _ = write!(
        report,
        "\n## OVD's own integration\n\nFor information: the same comparison with OVD's own hydrostatics, \
         its Sections Table cut at {} even stations plus the knuckles and the ends of each height's \
         waterline, and points at {} even heights plus the knuckle heights, integrated by Simpson's rule on \
         the ruled surface between sections (at T/Dd 1.0 a hair below the deck, where the closed section has \
         its deck). The hulls outside here are mostly at low draught, where the dataset's trapezoid rule over \
         its first waterlines differs from OVD's integral.\n\n{rows}\n",
        BENCHMARK.stations, BENCHMARK.levels
    );
    if outside.is_empty() {
        report.push_str("None outside.\n");
    } else {
        report.push_str("| Set | Row | Outside the limit | Where |\n|---|---|---|---|\n");
        report.push_str(&outside);
    }
    report.push_str(&fillet_section(&hulls)?);
    let (rows, _, _) = table(&hulls, DESIGN)?;
    let _ = write!(
        report,
        "\n## At a Design's cut\n\nFor information: a Design's ShipD hull is cut more coarsely ({} \
         even stations and {} even height levels, no waterline-end stations), so its hydrostatics \
         and GZ curves stay fast. At that cut OVD's own integration gives:\n\n{rows}",
        DESIGN.stations, DESIGN.levels
    );
    std::fs::write(dir.join("report.md"), &report).map_err(|e| e.to_string())?;
    print!("{report}");
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("the ShipD gate fails:\n{}", problems.join("\n")))
    }
}
