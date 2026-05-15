#![no_main]

use libfuzzer_sys::fuzz_target;
use orchestrator::{contains_shell_operator, parse_legacy_command, ToolError};

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let parsed = parse_legacy_command(&input);

    if contains_shell_operator(&input) || input.contains('\0') {
        assert!(matches!(parsed, Err(ToolError::CommandInjection { .. })));
        return;
    }

    if let Ok(command) = parsed {
        assert!(!command.executable().is_empty());
        for arg in command.args() {
            assert!(!contains_shell_operator(arg));
            assert!(!arg.contains('\0'));
        }
    }
});
