event eRequested;
event eSelected;

spec SelectionRequiresRequest observes eRequested, eSelected {
  start state AwaitRequest {
    on eRequested goto RequestSeen;
    on eSelected goto Bad;
  }
  state RequestSeen {
    ignore eRequested, eSelected;
  }
  state Bad {
    entry {
      assert false, "selection happened before exact request";
    }
  }
}

machine ExactSelectionPositiveHarness {
  start state Init {
    entry {
      announce eRequested;
      announce eSelected;
    }
  }
}

machine ExactSelectionDetectionHarness {
  start state Init {
    entry {
      announce eSelected;
    }
  }
}

test TcExactSelectionPositive [main=ExactSelectionPositiveHarness]:
  assert SelectionRequiresRequest in
  { ExactSelectionPositiveHarness };

test TcExactSelectionDetection [main=ExactSelectionDetectionHarness]:
  assert SelectionRequiresRequest in
  { ExactSelectionDetectionHarness };

