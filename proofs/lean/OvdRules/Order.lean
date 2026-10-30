/-!
# The order the proofs need

Core Lean has no `LinearOrder` class (that is in Mathlib). The proofs need less: a total
preorder on the limit type. `Int`, `Nat`, the rationals and the reals all have one. `Float` has
none, because `NaN ≤ NaN` is false: see the README.
-/

namespace OvdRules

/-- A total preorder: reflexive, transitive and total. Antisymmetry is not needed. -/
class TotalPreorder (α : Type) [LE α] : Prop where
  refl : ∀ a : α, a ≤ a
  trans : ∀ {a b c : α}, a ≤ b → b ≤ c → a ≤ c
  total : ∀ a b : α, a ≤ b ∨ b ≤ a

instance : TotalPreorder Int where
  refl := Int.le_refl
  trans := Int.le_trans
  total := Int.le_total

instance : TotalPreorder Nat where
  refl := Nat.le_refl
  trans := Nat.le_trans
  total := Nat.le_total

end OvdRules
