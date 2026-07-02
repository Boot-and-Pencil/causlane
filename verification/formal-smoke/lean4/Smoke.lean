inductive SmokeEvent where
  | gateOpened
  | useAttempted
deriving DecidableEq, Repr

structure SmokeState where
  gateSeen : Bool
deriving Repr

def eventAllowed (state : SmokeState) (event : SmokeEvent) : Bool :=
  match event with
  | SmokeEvent.gateOpened => true
  | SmokeEvent.useAttempted => state.gateSeen

def step (state : SmokeState) (event : SmokeEvent) : SmokeState :=
  match event with
  | SmokeEvent.gateOpened => { gateSeen := true }
  | SmokeEvent.useAttempted => state

def reduce : List SmokeEvent -> Bool
  | [] => true
  | event :: rest =>
      let state : SmokeState := { gateSeen := false }
      if eventAllowed state event then
        reduceFrom (step state event) rest
      else
        false
where
  reduceFrom (state : SmokeState) : List SmokeEvent -> Bool
    | [] => true
    | event :: rest =>
        if eventAllowed state event then
          reduceFrom (step state event) rest
        else
          false

theorem valid_gate_then_use_is_accepted :
    reduce [SmokeEvent.gateOpened, SmokeEvent.useAttempted] = true := by
  rfl

theorem invalid_use_before_gate_is_rejected :
    reduce [SmokeEvent.useAttempted] = false := by
  rfl
