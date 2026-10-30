import Lean.Data.Json
import OvdRules

/-!
# `ovd-rules-oracle`

Reads cases as JSON, evaluates them with the proved definitions at `Float`, and prints the
verdict, each row's outcome and the governing Rule of each requirement. A Rust test can write
the same JSON from property-test cases and compare (the differential-test seam). See the README
for the input form and for why `Float` needs finite limits and values.
-/

open Lean OvdRules

def parseDir : String → Except String Dir
  | "lower" => .ok .lower
  | "upper" => .ok .upper
  | s => .error s!"unknown dir '{s}' (expected lower or upper)"

def parseRule (j : Json) : Except String (Rule Float) := do
  let id ← j.getObjValAs? String "id"
  let key ← j.getObjValAs? String "key"
  let dir ← parseDir (← j.getObjValAs? String "dir")
  let limit ← j.getObjValAs? Float "limit"
  let strict ← j.getObjValAs? Bool "strict"
  let blocking ← j.getObjValAs? Bool "blocking"
  return { id, key, bound := { dir, limit, strict }, blocking }

/-- Whether a Rule lends its limit to a sizing step (`sizes`); absent means false. -/
def parseSizes (j : Json) : Bool :=
  match j.getObjVal? "sizes" with
  | .ok (.bool b) => b
  | _ => false

/-- Measure values: a missing key or `null` means not evaluated. -/
def parseValues (j : Json) : Except String (List (String × Float)) := do
  let obj ← j.getObj?
  obj.fold (init := .ok []) fun acc k v => do
    let acc ← acc
    match v with
    | .null => return acc
    | _ => return acc ++ [(k, ← fromJson? v)]

def dirName : Dir → String
  | .lower => "lower"
  | .upper => "upper"

def outcomeName : Outcome → String
  | .pass => "pass"
  | .fail => "fail"
  | .notEvaluated => "not evaluated"

/-- Requirement groups in first-seen order: rules with one key and one direction. -/
def groups (rs : List (Rule Float)) : List (String × List (Rule Float)) :=
  let names := rs.foldl (fun acc r =>
    let n := s!"{r.key}|{dirName r.bound.dir}"
    if acc.contains n then acc else acc ++ [n]) []
  names.map fun n => (n, rs.filter fun r => s!"{r.key}|{dirName r.bound.dir}" == n)

def evalCase (j : Json) : Except String Json := do
  let name ← j.getObjValAs? String "name"
  let ruleJson := (← j.getObjValAs? (Array Json) "rules").toList
  let rules ← ruleJson.mapM parseRule
  let sizing := (rules.zip (ruleJson.map parseSizes)).filterMap fun (r, s) =>
    if s then some r.id else none
  let values ← parseValues (← j.getObjVal? "values")
  let design : Design Float := fun k => (values.find? (·.1 == k)).map (·.2)
  -- Spec 0004: a failing Rule that does not block reports `warn`; the governing Rule is the
  -- strictest of the blocking Rules of the requirement, or of all of them if none blocks.
  let rows := rules.map fun r =>
    let o := r.outcome design
    (r.id, Json.str (if o == .fail && !r.blocking then "warn" else outcomeName o))
  let gov := (groups rules).filterMap fun (n, g) =>
    let blocking := g.filter (·.blocking)
    match (if blocking.isEmpty then g else blocking) with
    | [] => none
    | r :: rest => some (n, Json.str (governing r rest).id)
  -- Sizing: the governing bound of a requirement that has a `sizes` Rule is the value the sizing
  -- step builds at (the limit, as the effective bound; for a strict bound, any value beyond it).
  let sized := (groups rules).filterMap fun (n, g) =>
    let blocking := g.filter (·.blocking)
    match (if blocking.isEmpty then g else blocking) with
    | [] => none
    | r :: rest =>
      if g.any (fun x => sizing.contains x.id) then
        let w := governing r rest
        some (n, Json.mkObj [("rule", .str w.id), ("limit", toJson w.bound.limit),
                             ("strict", .bool w.bound.strict)])
      else none
  return Json.mkObj [
    ("name", .str name),
    ("ok", .bool (ok rules design)),
    ("outcomes", Json.mkObj rows),
    ("governing", Json.mkObj gov),
    ("sized", Json.mkObj sized)]

def main (args : List String) : IO UInt32 := do
  let some path := args.head?
    | IO.eprintln "usage: ovd-rules-oracle <cases.json>"; return 2
  let text ← IO.FS.readFile path
  match (do
      let j ← Json.parse text
      let cases ← j.getObjValAs? (Array Json) "cases"
      let out ← cases.toList.mapM evalCase
      return Json.mkObj [("cases", Json.arr out.toArray)]) with
  | .ok out => IO.println out.pretty; return 0
  | .error e => IO.eprintln s!"ovd-rules-oracle: {e}"; return 1
