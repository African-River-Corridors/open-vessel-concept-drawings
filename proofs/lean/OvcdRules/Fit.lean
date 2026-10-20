import OvcdRules.Rule

/-!
# `fit`

`fit` keeps the candidates whose verdict is `ok` and returns the one with the most cargo (the
first one on a tie), or `none` when no candidate passes.

Property 4 (`fit_sound`): `fit` never returns a candidate that fails a selected blocking Rule.
`fit_cargo_antitone`: adding Rules never raises the cargo of the answer.

The candidate list is fixed here. In the engine a `size` Rule also shapes the candidates; the
model leaves that out (see the README).
-/

namespace OvcdRules

variable {α β : Type} [LE α] [DecidableLE α]

/-- Keep the better of the best so far and the next candidate; the earlier one on a tie. -/
def better (cargo : β → Nat) : Option β → β → Option β
  | none, c => some c
  | some b, c => if cargo b < cargo c then some c else some b

def fit (rs : List (Rule α)) (design : β → Design α) (cargo : β → Nat) (cs : List β) :
    Option β :=
  (cs.filter fun c => ok rs (design c)).foldl (better cargo) none

theorem foldl_better_mem (cargo : β → Nat) (xs : List β) (init : Option β) (c : β)
    (h : xs.foldl (better cargo) init = some c) : init = some c ∨ c ∈ xs := by
  induction xs generalizing init with
  | nil => exact Or.inl h
  | cons x xs ih =>
    simp only [List.foldl] at h
    rcases ih _ h with h' | h'
    · cases init with
      | none =>
        simp only [better, Option.some.injEq] at h'
        exact Or.inr (h' ▸ List.mem_cons_self ..)
      | some b =>
        simp only [better] at h'
        split at h'
        · simp only [Option.some.injEq] at h'
          exact Or.inr (h' ▸ List.mem_cons_self ..)
        · exact Or.inl h'
    · exact Or.inr (List.mem_cons_of_mem _ h')

theorem foldl_better_max (cargo : β → Nat) (xs : List β) (init : Option β) (c : β)
    (h : xs.foldl (better cargo) init = some c) :
    (∀ b, init = some b → cargo b ≤ cargo c) ∧ ∀ y ∈ xs, cargo y ≤ cargo c := by
  induction xs generalizing init with
  | nil =>
    simp only [List.foldl] at h
    subst h
    exact ⟨fun b hb => by simp at hb; subst hb; exact Nat.le_refl _, by simp⟩
  | cons x xs ih =>
    simp only [List.foldl] at h
    obtain ⟨h1, h2⟩ := ih _ h
    refine ⟨?_, ?_⟩
    · intro b hb
      subst hb
      by_cases hlt : cargo b < cargo x
      · exact Nat.le_trans (Nat.le_of_lt hlt) (h1 x (by simp [better, hlt]))
      · exact h1 b (by simp [better, hlt])
    · intro y hy
      simp only [List.mem_cons] at hy
      rcases hy with rfl | hy
      · cases init with
        | none => exact h1 _ (by simp [better])
        | some b =>
          by_cases hlt : cargo b < cargo y
          · exact h1 y (by simp [better, hlt])
          · exact Nat.le_trans (Nat.le_of_not_lt hlt) (h1 b (by simp [better, hlt]))
      · exact h2 y hy

/-- **Property 4 (soundness).** `fit` returns a candidate from the list that meets every
selected blocking Rule. -/
theorem fit_sound (rs : List (Rule α)) (design : β → Design α) (cargo : β → Nat) (cs : List β)
    (c : β) (h : fit rs design cargo cs = some c) : c ∈ cs ∧ ok rs (design c) = true := by
  rcases foldl_better_mem cargo _ none c h with h' | h'
  · exact absurd h' (by simp)
  · exact List.mem_filter.mp h'

/-- `fit` returns the most cargo among the candidates that pass. -/
theorem fit_max (rs : List (Rule α)) (design : β → Design α) (cargo : β → Nat) (cs : List β)
    (c : β) (h : fit rs design cargo cs = some c) (y : β) (hy : y ∈ cs)
    (hok : ok rs (design y) = true) : cargo y ≤ cargo c :=
  (foldl_better_max cargo _ none c h).2 y (List.mem_filter.mpr ⟨hy, hok⟩)

/-- **Property 4 (cargo).** Adding Rules never raises the cargo of the answer. -/
theorem fit_cargo_antitone (rs rs' : List (Rule α)) (design : β → Design α) (cargo : β → Nat)
    (cs : List β) (c c' : β) (h : fit rs design cargo cs = some c)
    (h' : fit (rs ++ rs') design cargo cs = some c') : cargo c' ≤ cargo c := by
  obtain ⟨hmem, hok⟩ := fit_sound _ design cargo cs c' h'
  exact fit_max rs design cargo cs c h c' hmem (ok_of_ok_append rs rs' _ hok)

/-- Adding Rules never makes `fit` find an answer where it found none. -/
theorem fit_none_append (rs rs' : List (Rule α)) (design : β → Design α) (cargo : β → Nat)
    (cs : List β) (h : fit rs design cargo cs = none) :
    fit (rs ++ rs') design cargo cs = none := by
  cases h' : fit (rs ++ rs') design cargo cs with
  | none => rfl
  | some c' =>
    obtain ⟨hmem, hok⟩ := fit_sound _ design cargo cs c' h'
    have hok' := ok_of_ok_append rs rs' _ hok
    have : (cs.filter fun c => ok rs (design c)) ≠ [] := by
      intro he
      have := (List.mem_filter (p := fun c => ok rs (design c))).mpr ⟨hmem, hok'⟩
      rw [he] at this
      simp at this
    -- a non-empty list always gives an answer
    obtain ⟨x, xs, hx⟩ := List.exists_cons_of_ne_nil this
    have : ∀ (ys : List β) (i : Option β), i ≠ none → ys.foldl (better cargo) i ≠ none := by
      intro ys
      induction ys with
      | nil => intro i hi; exact hi
      | cons y ys ih =>
        intro i _
        apply ih
        cases i with
        | none => simp [better]
        | some b => simp only [better]; split <;> simp
    have hne := this xs (better cargo none x) (by simp [better])
    simp only [fit, hx, List.foldl] at h
    exact absurd h hne

end OvcdRules
