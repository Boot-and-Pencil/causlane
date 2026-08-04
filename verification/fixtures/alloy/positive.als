module exact_selection
sig Candidate {}
one sig Requested in Candidate {}
one sig Selected in Candidate {}
fact ExactSelection { Selected = Requested }
assert SelectionNeverWidens { Selected = Requested }
check SelectionNeverWidens for 4
run { some Candidate } for 4

