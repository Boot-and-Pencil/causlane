//! Parse-boundary fuzz target for registry YAML compilation.
//!
//! The target delegates validation to `CompiledDispatchBundle::compile`, the
//! same authority used by normal contract loading.

#![no_main]

use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(manifest) = RegistryManifest::from_yaml_str(text) {
            let _bundle = CompiledDispatchBundle::compile(&manifest);
        }
    }
});
