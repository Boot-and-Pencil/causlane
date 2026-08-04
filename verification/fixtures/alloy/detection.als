module exact_selection_detection
sig Candidate {}
one sig Requested in Candidate {}
one sig Selected in Candidate {}
assert SelectionNeverWidens { Selected = Requested }
check SelectionNeverWidens for 4

