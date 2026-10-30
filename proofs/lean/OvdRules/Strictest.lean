import OvdRules.Bound

/-!
# Strictest governs

Of several Rules on one requirement (one Measure, one object, one direction), the strictest bound
governs. `strictestBy f acc xs` walks the list and keeps the first item whose bound `f` is at least
as strict as every other. On a tie the earlier item stays, so the governing Rule is deterministic.

Property 1 (`holds_strictestBy_iff`): a value meets the strictest bound exactly when it meets
every bound. So "strictest governs" and "every Rule applies" give the same verdict.
-/

set_option linter.unusedSectionVars false

namespace OvdRules

variable {α γ : Type} [LE α] [DecidableLE α]

/-- Keep `acc` unless a later item is strictly stricter. -/
def strictestBy (f : γ → Bound α) (acc : γ) : List γ → γ
  | [] => acc
  | x :: xs => strictestBy f (if (f acc).atLeastAsStrict (f x) then acc else x) xs

/-- All items have bounds of direction `d`. -/
def SameDir (f : γ → Bound α) (d : Dir) (xs : List γ) : Prop := ∀ x ∈ xs, (f x).dir = d

theorem strictestBy_mem (f : γ → Bound α) (acc : γ) (xs : List γ) :
    strictestBy f acc xs ∈ acc :: xs := by
  induction xs generalizing acc with
  | nil => simp [strictestBy]
  | cons x xs ih =>
    simp only [strictestBy]
    split
    · have := ih acc
      simp only [List.mem_cons] at this ⊢
      rcases this with h | h
      · exact Or.inl h
      · exact Or.inr (Or.inr h)
    · have := ih x
      simp only [List.mem_cons] at this ⊢
      exact Or.inr this

section Laws
variable [TotalPreorder α]

/-- The result is at least as strict as every item of the list. -/
theorem strictestBy_atLeastAsStrict (f : γ → Bound α) (d : Dir) (acc : γ) (xs : List γ)
    (h : SameDir f d (acc :: xs)) :
    ∀ y ∈ acc :: xs, (f (strictestBy f acc xs)).atLeastAsStrict (f y) = true := by
  induction xs generalizing acc with
  | nil =>
    intro y hy
    simp only [List.mem_singleton] at hy
    subst hy
    exact atLeastAsStrict_refl _
  | cons x xs ih =>
    intro y hy
    have hacc : (f acc).dir = d := h acc (by simp)
    have hx : (f x).dir = d := h x (by simp)
    simp only [strictestBy]
    split
    · rename_i hax
      have h' : SameDir f d (acc :: xs) := fun z hz => h z (by
        simp only [List.mem_cons] at hz ⊢
        rcases hz with hz | hz
        · exact Or.inl hz
        · exact Or.inr (Or.inr hz))
      have ih' := ih acc h'
      simp only [List.mem_cons] at hy
      rcases hy with rfl | rfl | hy
      · exact ih' _ (by simp)
      · have hr := ih' acc (by simp)
        have hrd : (f (strictestBy f acc xs)).dir = d := h' _ (strictestBy_mem f acc xs)
        exact atLeastAsStrict_trans (by rw [hrd, hacc]) (by rw [hacc, hx]) hr hax
      · exact ih' _ (by simp [hy])
    · rename_i hax
      have h' : SameDir f d (x :: xs) := fun z hz => h z (List.mem_cons_of_mem _ hz)
      have ih' := ih x h'
      have hxa : (f x).atLeastAsStrict (f acc) = true := by
        rcases atLeastAsStrict_total (f acc) (f x) (by rw [hacc, hx]) with h1 | h1
        · exact absurd h1 hax
        · exact h1
      simp only [List.mem_cons] at hy
      rcases hy with rfl | rfl | hy
      · have hr := ih' x (by simp)
        have hrd : (f (strictestBy f x xs)).dir = d := h' _ (strictestBy_mem f x xs)
        exact atLeastAsStrict_trans (by rw [hrd, hx]) (by rw [hx, hacc]) hr hxa
      · exact ih' _ (by simp)
      · exact ih' _ (by simp [hy])

/-- **Property 1.** For bounds of one direction, the strictest holds exactly when all hold. -/
theorem holds_strictestBy_iff (f : γ → Bound α) (d : Dir) (acc : γ) (xs : List γ)
    (h : SameDir f d (acc :: xs)) (v : α) :
    (f (strictestBy f acc xs)).holds v = true ↔ ∀ y ∈ acc :: xs, (f y).holds v = true := by
  constructor
  · intro hs y hy
    have hr := strictestBy_atLeastAsStrict f d acc xs h y hy
    have hrd : (f (strictestBy f acc xs)).dir = d := h _ (strictestBy_mem f acc xs)
    exact holds_of_atLeastAsStrict (by rw [hrd, h y hy]) hr hs
  · intro hall
    exact hall _ (strictestBy_mem f acc xs)

end Laws

end OvdRules
