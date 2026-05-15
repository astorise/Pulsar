#![no_main]

use libfuzzer_sys::fuzz_target;
use orchestrator::{PathGuard, ToolError};

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let guard = PathGuard::new(".");

    if let Ok(path) = guard.validate(&input) {
        assert!(!path.contains('\0'));
        assert!(!path.starts_with('/'));
        assert!(!path.contains('\\'));
        assert!(!path.split('/').any(|component| component == ".."));
    }

    if input.starts_with("../") || input == ".." {
        assert!(matches!(
            guard.validate(&input),
            Err(ToolError::UnauthorizedAccess { .. })
        ));
    }
});
