use vstd::prelude::*;

verus! {

spec fn selected_exact(requested: int, selected: int) -> bool {
    selected == requested
}

proof fn exact_selection_preserves_request(requested: int)
    ensures selected_exact(requested, requested)
{
}

}

fn main() {}

