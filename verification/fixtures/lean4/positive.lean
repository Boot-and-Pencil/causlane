def selectedIdentity (requested : Nat) (available : List Nat) : Option Nat :=
  available.find? (fun candidate => candidate == requested)

theorem exact_selection_is_preserved :
    selectedIdentity 3 [1, 3, 5] = some 3 := by
  rfl

