import OvcdRules.Bound

/-!
# Conflicting bounds

A lower bound and an upper bound on one requirement conflict when no value can meet both. The
loader can check this for constant limits. Property 2 (`no_value_of_conflict`): the check is
sound. `witness_of_not_conflict`: when the check finds no conflict and both bounds are
non-strict, the lower limit meets both, so the check refuses nothing that could be met.
-/

set_option linter.unusedSectionVars false

namespace OvcdRules

variable {α : Type} [LE α] [DecidableLE α]

/-- `lo` (a lower bound) and `up` (an upper bound) leave no value. -/
def conflict (lo up : Bound α) : Bool :=
  !decide (lo.limit ≤ up.limit) ||
    (decide (up.limit ≤ lo.limit) && (lo.strict || up.strict))

section Laws
variable [TotalPreorder α]

private theorem le_of_not_le' {a b : α} (h : ¬ a ≤ b) : b ≤ a := by
  rcases TotalPreorder.total a b with h' | h'
  · exact absurd h' h
  · exact h'

/-- A lower bound that holds gives `limit ≤ v`. -/
theorem lower_le {lo : Bound α} (hd : lo.dir = .lower) {v : α} (h : lo.holds v = true) :
    lo.limit ≤ v := by
  obtain ⟨d, l, s⟩ := lo
  simp only at hd
  subst hd
  cases s <;> simp [Bound.holds] at h ⊢
  · exact h
  · exact le_of_not_le' h

/-- An upper bound that holds gives `v ≤ limit`. -/
theorem le_upper {up : Bound α} (hd : up.dir = .upper) {v : α} (h : up.holds v = true) :
    v ≤ up.limit := by
  obtain ⟨d, l, s⟩ := up
  simp only at hd
  subst hd
  cases s <;> simp [Bound.holds] at h ⊢
  · exact h
  · exact le_of_not_le' h

/-- **Property 2.** If the check reports a conflict, no value meets both bounds. -/
theorem no_value_of_conflict {lo up : Bound α} (hlo : lo.dir = .lower) (hup : up.dir = .upper)
    (hc : conflict lo up = true) (v : α) : ¬ (lo.holds v = true ∧ up.holds v = true) := by
  intro ⟨h1, h2⟩
  have hl := lower_le hlo h1
  have hu := le_upper hup h2
  have hlu : lo.limit ≤ up.limit := TotalPreorder.trans hl hu
  simp only [conflict, Bool.or_eq_true, Bool.not_eq_true', decide_eq_false_iff_not,
    Bool.and_eq_true, decide_eq_true_eq] at hc
  rcases hc with hc | ⟨hul, hs⟩
  · exact hc hlu
  · obtain ⟨d, l, s⟩ := lo
    obtain ⟨d', l', s'⟩ := up
    simp only at hlo hup
    subst hlo; subst hup
    cases s <;> cases s' <;> simp [Bound.holds] at h1 h2 hs
    · exact h2 (TotalPreorder.trans hul hl)
    · exact h1 (TotalPreorder.trans hu hul)
    · exact h1 (TotalPreorder.trans hu hul)

/-- With no conflict and two non-strict bounds, the lower limit meets both. -/
theorem witness_of_not_conflict {lo up : Bound α} (hlo : lo.dir = .lower)
    (hup : up.dir = .upper) (hs1 : lo.strict = false) (hs2 : up.strict = false)
    (hc : conflict lo up = false) : lo.holds lo.limit = true ∧ up.holds lo.limit = true := by
  obtain ⟨d, l, s⟩ := lo
  obtain ⟨d', l', s'⟩ := up
  simp only at hlo hup hs1 hs2
  subst hlo; subst hup; subst hs1; subst hs2
  simp only [conflict, Bool.or_false, Bool.and_false, Bool.not_eq_false',
    decide_eq_true_eq] at hc
  simp [Bound.holds, TotalPreorder.refl l, hc]

end Laws

end OvcdRules
