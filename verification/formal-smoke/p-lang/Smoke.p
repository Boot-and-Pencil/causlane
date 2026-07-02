event eGateOpened;
event eUseAttempted;

spec UseRequiresGate observes eGateOpened, eUseAttempted {
  start state NeedGate {
    on eGateOpened goto GateSeen;
    on eUseAttempted goto Bad;
  }

  state GateSeen {
    ignore eGateOpened, eUseAttempted;
  }

  state Bad {
    entry {
      assert false, "use attempted before gate";
    }
  }
}

machine GateBeforeUseValidHarness {
  start state Init {
    entry {
      announce eGateOpened;
      announce eUseAttempted;
    }
  }
}

machine GateBeforeUseInvalidHarness {
  start state Init {
    entry {
      announce eUseAttempted;
    }
  }
}

test TcGateBeforeUseValid [main=GateBeforeUseValidHarness]:
  assert UseRequiresGate in
  { GateBeforeUseValidHarness };

test TcGateBeforeUseInvalid [main=GateBeforeUseInvalidHarness]:
  assert UseRequiresGate in
  { GateBeforeUseInvalidHarness };
