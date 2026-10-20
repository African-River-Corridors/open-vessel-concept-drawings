import OvcdRules.Order

/-!
# Bounds

A Rule compares one Measure value against one limit. `lower` is `ge` (or `gt` when strict);
`upper` is `le` (or `lt` when strict). Only `≤` is used, so the same definitions run on `Float`
in the oracle. A strict bound is written as "not the other way round".
-/

set_option linter.unusedSectionVars false

namespace OvcdRules

inductive Dir where
  | lower
  | upper
  deriving Repr, DecidableEq, BEq

structure Bound (α : Type) where
  dir : Dir
  limit : α
  strict : Bool
  deriving Repr

variable {α : Type} [LE α] [DecidableLE α]

/-- The value meets the bound. -/
def Bound.holds (b : Bound α) (v : α) : Bool :=
  match b.dir, b.strict with
  | .lower, false => decide (b.limit ≤ v)
  | .lower, true => !decide (v ≤ b.limit)
  | .upper, false => decide (v ≤ b.limit)
  | .upper, true => !decide (b.limit ≤ v)

/-- `b` is at least as strict as `c`. Meant for two bounds of one direction; the direction of `b`
decides. Lower: the limit is higher, or equal and strict wherever `c` is strict. Upper: mirrored. -/
def Bound.atLeastAsStrict (b c : Bound α) : Bool :=
  match b.dir with
  | .lower => decide (c.limit ≤ b.limit) && (!decide (b.limit ≤ c.limit) || !c.strict || b.strict)
  | .upper => decide (b.limit ≤ c.limit) && (!decide (c.limit ≤ b.limit) || !c.strict || b.strict)

section Laws
variable [TotalPreorder α]

private theorem not_le_le {a b : α} (h : ¬ a ≤ b) : b ≤ a := by
  rcases TotalPreorder.total a b with h' | h'
  · exact absurd h' h
  · exact h'

theorem atLeastAsStrict_refl (b : Bound α) : b.atLeastAsStrict b = true := by
  obtain ⟨d, l, s⟩ := b
  cases d <;> cases s <;> simp [Bound.atLeastAsStrict, TotalPreorder.refl l]

/-- Totality: of two bounds of one direction, one is at least as strict as the other. -/
theorem atLeastAsStrict_total (b c : Bound α) (h : b.dir = c.dir) :
    b.atLeastAsStrict c = true ∨ c.atLeastAsStrict b = true := by
  obtain ⟨d, l, s⟩ := b
  obtain ⟨d', l', s'⟩ := c
  simp only at h
  subst h
  rcases TotalPreorder.total l l' with h1 | h1 <;>
  by_cases h2 : l' ≤ l <;> cases s <;> cases s' <;> cases d <;>
    simp_all [Bound.atLeastAsStrict]
  all_goals first | exact Decidable.em _ | exact (Decidable.em _).symm

/-- Transitivity, for three bounds of one direction. -/
theorem atLeastAsStrict_trans {a b c : Bound α} (hab : a.dir = b.dir) (hbc : b.dir = c.dir)
    (h1 : a.atLeastAsStrict b = true) (h2 : b.atLeastAsStrict c = true) :
    a.atLeastAsStrict c = true := by
  obtain ⟨d, la, sa⟩ := a
  obtain ⟨db, lb, sb⟩ := b
  obtain ⟨dc, lc, sc⟩ := c
  simp only at hab hbc
  subst hab; subst hbc
  cases d <;> simp only [Bound.atLeastAsStrict, Bool.and_eq_true, Bool.or_eq_true,
    decide_eq_true_eq, Bool.not_eq_true', decide_eq_false_iff_not] at h1 h2 ⊢
  · obtain ⟨h1a, h1b⟩ := h1
    obtain ⟨h2a, h2b⟩ := h2
    refine ⟨TotalPreorder.trans h2a h1a, ?_⟩
    by_cases hac : la ≤ lc
    · have hba : lb ≤ lc := TotalPreorder.trans h1a hac
      have hab' : la ≤ lb := TotalPreorder.trans hac h2a
      cases sc <;> cases sb <;> cases sa <;> simp_all
    · exact Or.inl (Or.inl hac)
  · obtain ⟨h1a, h1b⟩ := h1
    obtain ⟨h2a, h2b⟩ := h2
    refine ⟨TotalPreorder.trans h1a h2a, ?_⟩
    by_cases hca : lc ≤ la
    · have hcb : lc ≤ lb := TotalPreorder.trans hca h1a
      have hba : lb ≤ la := TotalPreorder.trans h2a hca
      cases sc <;> cases sb <;> cases sa <;> simp_all
    · exact Or.inl (Or.inl hca)

/-- A stricter bound implies the looser one: whatever meets `b` meets `c`. -/
theorem holds_of_atLeastAsStrict {b c : Bound α} (hd : b.dir = c.dir)
    (h : b.atLeastAsStrict c = true) {v : α} (hb : b.holds v = true) : c.holds v = true := by
  obtain ⟨d, l, s⟩ := b
  obtain ⟨d', l', s'⟩ := c
  simp only at hd
  subst hd
  cases d <;> cases s <;> cases s' <;>
    simp only [Bound.atLeastAsStrict, Bound.holds, Bool.and_eq_true, Bool.or_eq_true,
      decide_eq_true_eq, Bool.not_eq_true', decide_eq_false_iff_not, Bool.not_true,
      Bool.not_false, Bool.false_eq_true, or_false, or_true, and_true] at h hb ⊢
  -- lower, b non-strict, c non-strict
  · exact TotalPreorder.trans h hb
  -- lower, b non-strict, c strict: v ≤ l' ≤ l would give l ≤ l'
  · exact fun hv => h.2 (TotalPreorder.trans hb hv)
  -- lower, b strict, c non-strict
  · exact TotalPreorder.trans h (not_le_le hb)
  -- lower, b strict, c strict
  · exact fun hv => hb (TotalPreorder.trans hv h)
  -- upper, b non-strict, c non-strict
  · exact TotalPreorder.trans hb h
  -- upper, b non-strict, c strict
  · exact fun hv => h.2 (TotalPreorder.trans hv hb)
  -- upper, b strict, c non-strict
  · exact TotalPreorder.trans (not_le_le hb) h
  -- upper, b strict, c strict
  · exact fun hv => hb (TotalPreorder.trans h hv)

end Laws

end OvcdRules
