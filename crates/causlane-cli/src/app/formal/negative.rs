//! Replay-backed negative control collection for formal coverage.

use std::path::Path;

use causlane_codegen::{NegativeControl, NegativeControlStatus};
use causlane_contracts::CompiledDispatchBundle;
use causlane_replay::{ReplayError, ReplayScenario};

use super::io::{read_file, FormalIo};
use super::FormalServiceError;

pub(super) fn collect_negative_controls(
    io: &mut dyn FormalIo,
    dir: &str,
    bundle: &CompiledDispatchBundle,
) -> Result<Vec<NegativeControl>, FormalServiceError> {
    let Some(entries) =
        io.read_dir_paths(Path::new(dir))
            .map_err(|err| FormalServiceError::Io {
                path: dir.to_owned(),
                message: err.to_string(),
            })?
    else {
        return Ok(Vec::new());
    };

    let mut controls = Vec::new();
    for path in entries {
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("_invalid.scenario.yaml"))
        {
            continue;
        }
        let yaml = read_file(io, &path)?;
        let scenario = ReplayScenario::from_yaml_str(&yaml)?;
        let expected = scenario.expected_error_code.clone();
        let outcome = scenario.to_trace().verify_with_bundle(bundle);
        let actual_error_code = outcome
            .as_ref()
            .err()
            .map(|err| err.code().as_str().to_owned());
        let status = negative_control_status(expected.as_deref(), &outcome);
        controls.push(NegativeControl {
            scenario: scenario.scenario_id,
            expected_error_code: expected,
            status,
            actual_error_code,
        });
    }
    controls.sort_by(|left, right| left.scenario.cmp(&right.scenario));
    Ok(controls)
}

fn negative_control_status(
    expected: Option<&str>,
    outcome: &Result<(), ReplayError>,
) -> NegativeControlStatus {
    match (expected, outcome) {
        (Some(code), Err(err)) if err.code().as_str() == code => {
            NegativeControlStatus::RefutedByReplay
        }
        (Some(_code), Err(_err)) => NegativeControlStatus::WrongCode,
        (Some(_code), Ok(())) => NegativeControlStatus::UnexpectedPass,
        (None, Err(_err)) => NegativeControlStatus::RefutedByReplay,
        (None, Ok(())) => NegativeControlStatus::UnexpectedPass,
    }
}
