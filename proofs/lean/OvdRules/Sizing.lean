import OvdRules.Strictest

/-!
# Sizing

A Rule with `sizes: true` lends its limit to a sizing step: the channel width of the waterway
designer, the tank gap of the gas search. The step takes the governing Criterion of the
Requirement and builds the candidate at its effective bound. Two properties of that use belong
to the kernel:

- **Property 5 (`sized_meets_requirement`).** The sized value meets every Criterion of the
  Requirement. The sized value is the limit of the governing Criterion; for a strict Criterion
  (`gt`, `lt`) the limit itself does not meet it, so any value strictly beyond the limit is
  sized (`Bound.sizedAt`).
- **Property 6 (`sized_limit_mono_lower`, `sized_limit_mono_upper`).** Adding Criteria to a
  Requirement never lowers the sized value of a lower bound, and never raises that of an upper
  bound. The general form is `strictestBy_mono`: the new governing Criterion is at least as
  strict as the old one.

The limit here is the *effective* bound: the Rust evaluator folds the tolerance into it before
it compares (`limit - tolerance` for a lower bound, `limit + tolerance` for an upper bound).
-/

set_option linter.unusedSectionVars false

namespace OvdRules

variable {α γ : Type} [LE α] [DecidableLE α]

/-- `v` is a sizing value for `b`: the limit itself when the bound is not strict, any value
strictly beyond the limit (above a lower bound, below an upper one) when it is. -/
def Bound.sizedAt (b : Bound α) (v : α) : Prop :=
  match b.dir, b.strict with
  | .lower, false => v = b.limit
  | .upper, false => v = b.limit
  | .lower, true => ¬ v ≤ b.limit
  | .upper, true => ¬ b.limit ≤ v

section Laws
variable [TotalPreorder α]

/-- A sizing value for a bound meets the bound. -/
theorem holds_of_sizedAt (b : Bound α) {v : α} (h : b.sizedAt v) : b.holds v = true := by
  obtain ⟨d, l, s⟩ := b
  cases d <;> cases s <;>
    simp only [Bound.sizedAt, Bound.holds] at h ⊢
  · subst h; simp [TotalPreorder.refl v]
  · simpa using h
  · subst h; simp [TotalPreorder.refl v]
  · simpa using h

/-- **Property 5.** The value sized from the governing Criterion of a non-empty Requirement of
one direction meets every Criterion of the Requirement. -/
theorem sized_meets_requirement (f : γ → Bound α) (d : Dir) (acc : γ) (xs : List γ)
    (h : SameDir f d (acc :: xs)) {v : α} (hv : (f (strictestBy f acc xs)).sizedAt v) :
    ∀ y ∈ acc :: xs, (f y).holds v = true :=
  (holds_strictestBy_iff f d acc xs h v).1 (holds_of_sizedAt _ hv)

/-- The governing Criterion of a larger Requirement is at least as strict as the governing
Criterion of a smaller one: the larger holds every item of the smaller. -/
theorem strictestBy_mono (f : γ → Bound α) (d : Dir) (acc : γ) (xs : List γ) (acc' : γ)
    (xs' : List γ) (h' : SameDir f d (acc' :: xs'))
    (hsub : ∀ y ∈ acc :: xs, y ∈ acc' :: xs') :
    (f (strictestBy f acc' xs')).atLeastAsStrict (f (strictestBy f acc xs)) = true :=
  strictestBy_atLeastAsStrict f d acc' xs' h' _ (hsub _ (strictestBy_mem f acc xs))

/-- **Property 6, lower bounds.** Adding Criteria never makes the governing effective bound
smaller, so the sized value never falls. -/
theorem sized_limit_mono_lower (f : γ → Bound α) (acc : γ) (xs ys : List γ)
    (h' : SameDir f .lower (acc :: (xs ++ ys))) :
    (f (strictestBy f acc xs)).limit ≤ (f (strictestBy f acc (xs ++ ys))).limit := by
  have h := strictestBy_mono f .lower acc xs acc (xs ++ ys) h' (by
    intro y hy
    simp only [List.mem_cons, List.mem_append] at hy ⊢
    rcases hy with hy | hy
    · exact Or.inl hy
    · exact Or.inr (Or.inl hy))
  have hd : (f (strictestBy f acc (xs ++ ys))).dir = .lower :=
    h' _ (strictestBy_mem f acc (xs ++ ys))
  simp only [Bound.atLeastAsStrict, hd, Bool.and_eq_true, decide_eq_true_eq] at h
  exact h.1

/-- **Property 6, upper bounds.** Adding Criteria never makes the governing effective bound
larger, so the sized value never rises. -/
theorem sized_limit_mono_upper (f : γ → Bound α) (acc : γ) (xs ys : List γ)
    (h' : SameDir f .upper (acc :: (xs ++ ys))) :
    (f (strictestBy f acc (xs ++ ys))).limit ≤ (f (strictestBy f acc xs)).limit := by
  have h := strictestBy_mono f .upper acc xs acc (xs ++ ys) h' (by
    intro y hy
    simp only [List.mem_cons, List.mem_append] at hy ⊢
    rcases hy with hy | hy
    · exact Or.inl hy
    · exact Or.inr (Or.inl hy))
  have hd : (f (strictestBy f acc (xs ++ ys))).dir = .upper :=
    h' _ (strictestBy_mem f acc (xs ++ ys))
  simp only [Bound.atLeastAsStrict, hd, Bool.and_eq_true, decide_eq_true_eq] at h
  exact h.1

end Laws

end OvdRules
