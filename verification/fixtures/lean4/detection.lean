def selectedIdentity (requested : Nat) (available : List Nat) : Option Nat :=
  available.find? (fun candidate => candidate == requested)

theorem widened_selection_is_rejected :
    selectedIdentity 3 [1, 3, 5] = some 5 := by
  rfl

