import OvcdRules.Strictest

/-!
# Rules, outcomes and the verdict of an Option

A Rule bounds the value of one requirement key (Measure id, its arguments and the object). A
design gives each key a value, or `none` when the engine cannot evaluate the Measure. A row is
`pass`, `fail` or `notEvaluated`. `ok` is the verdict: no blocking Rule fails. A `notEvaluated`
row does not fail: the result states it (spec 0004), and `fit` calls the design feasible with
the gap named.

Property 3 (`ok_of_ok_append`): adding Rules never turns a failing design into a passing one.
-/

namespace OvcdRules

structure Rule (α : Type) where
  id : String
  key : String
  bound : Bound α
  blocking : Bool
  deriving Repr

inductive Outcome where
  | pass
  | fail
  | notEvaluated
  deriving Repr, DecidableEq, BEq

/-- A design: the value of each requirement key, `none` when not evaluated. -/
abbrev Design (α : Type) := String → Option α

variable {α : Type} [LE α] [DecidableLE α]

def Rule.outcome (r : Rule α) (d : Design α) : Outcome :=
  match d r.key with
  | none => .notEvaluated
  | some v => if r.bound.holds v then .pass else .fail

/-- The row blocks the design: a blocking Rule that fails. -/
def Rule.blocks (r : Rule α) (d : Design α) : Bool :=
  r.blocking && r.outcome d == .fail

/-- The verdict: no blocking Rule fails. -/
def ok (rs : List (Rule α)) (d : Design α) : Bool :=
  rs.all fun r => !r.blocks d

theorem ok_iff (rs : List (Rule α)) (d : Design α) :
    ok rs d = true ↔ ∀ r ∈ rs, r.blocks d = false := by
  simp [ok]

/-- **Property 3 (monotonicity).** A design that passes more Rules passes fewer. -/
theorem ok_of_ok_append (rs rs' : List (Rule α)) (d : Design α) (h : ok (rs ++ rs') d = true) :
    ok rs d = true := by
  simp only [ok, List.all_append, Bool.and_eq_true] at h
  exact h.1

/-- The same, read the other way: adding Rules to a failing design keeps it failing. -/
theorem not_ok_append (rs rs' : List (Rule α)) (d : Design α) (h : ok rs d = false) :
    ok (rs ++ rs') d = false := by
  cases h' : ok (rs ++ rs') d
  · rfl
  · rw [ok_of_ok_append rs rs' d h'] at h
    exact absurd h (by decide)

/-- The governing Rule of a non-empty group of Rules on one requirement: the strictest bound,
the earlier Rule on a tie. -/
def governing (r : Rule α) (rs : List (Rule α)) : Rule α :=
  strictestBy Rule.bound r rs

end OvcdRules
