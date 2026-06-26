use causlane_cli::cli_shared::DEFAULT_FORMAL_LANE;
use causlane_formal::{FormalProfile, LaneCheck};

pub(crate) fn formal_profile_from_arg(value: &str) -> FormalProfile {
    if value == "base" {
        FormalProfile::Base
    } else if value == "rust" {
        FormalProfile::Rust
    } else if value == "proof" {
        FormalProfile::Proof
    } else if value == "all" {
        FormalProfile::All
    } else {
        FormalProfile::Custom
    }
}

pub(crate) fn formal_lane_checks(lane: &str) -> Vec<LaneCheck> {
    if lane == DEFAULT_FORMAL_LANE {
        return vec![LaneCheck::new(
            "local_publication_may_be_skipped",
            "ok",
            false,
        )];
    }
    vec![
        LaneCheck::new("clean_worktree", "unknown", true),
        LaneCheck::new("no_working_tree_tool_versions", "unknown", true),
    ]
}

#[cfg(test)]
mod tests {
    use super::formal_lane_checks;

    #[test]
    fn local_smoke_keeps_optional_publication_skip() {
        let checks = formal_lane_checks("local_smoke");
        let names: Vec<&str> = checks.iter().map(|check| check.name.as_str()).collect();
        let required: Vec<bool> = checks.iter().map(|check| check.required).collect();
        assert_eq!(names, ["local_publication_may_be_skipped"]);
        assert_eq!(required, [false]);
    }

    #[test]
    fn non_local_lanes_share_publication_contract() {
        for lane in ["provider_lane", "ci_like_lane"] {
            let checks = formal_lane_checks(lane);
            let names: Vec<&str> = checks.iter().map(|check| check.name.as_str()).collect();
            assert_eq!(names, ["clean_worktree", "no_working_tree_tool_versions"]);
            assert!(checks.iter().all(|check| check.required));
        }
    }
}
