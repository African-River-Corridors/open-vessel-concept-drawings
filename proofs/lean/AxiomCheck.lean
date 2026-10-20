import OvcdRules
open OvcdRules
-- CI fails if any theorem below uses `sorryAx`. Allowed: propext, Classical.choice, Quot.sound.
#print axioms atLeastAsStrict_total
#print axioms atLeastAsStrict_trans
#print axioms holds_strictestBy_iff
#print axioms no_value_of_conflict
#print axioms witness_of_not_conflict
#print axioms ok_of_ok_append
#print axioms fit_sound
#print axioms fit_cargo_antitone
#print axioms fit_none_append
