use vstd::prelude::*;

verus! {

spec fn selected_exact(requested: int, selected: int) -> bool {
    selected == requested
}

proof fn detection_rejects_parent_fallback(requested: int)
    ensures selected_exact(requested, requested - 1)
{
}

}

fn main() {}

