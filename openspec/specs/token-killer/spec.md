# token-killer Specification

## Purpose
TBD - created by archiving change token-killer-sanitization. Update Purpose after archive.
## Requirements
### Requirement: ANSI And Whitespace Removal
The orchestrator SHALL strip ANSI escape sequences, trim trailing whitespace, and condense repeated blank lines before storing command output in trace context.

#### Scenario: Command output contains terminal control codes
- **GIVEN** raw command output contains ANSI color sequences and extra blank lines
- **WHEN** the sanitizer cleans the output
- **THEN** the trace contains plain text with stable whitespace

### Requirement: Data-Driven Noise Reduction
The orchestrator SHALL use embedded JSON filter rules to remove known noisy lifecycle lines from Git, Cargo, Node, and Java output.

#### Scenario: Command output contains progress noise
- **GIVEN** command output contains a line matching an enabled filter rule
- **WHEN** the sanitizer runs for that command
- **THEN** the matching line is omitted from the stored trace

### Requirement: Smart Truncation
When command output exceeds the maximum line count, the orchestrator SHALL retain the beginning and end of the output with an omission marker in the middle.

#### Scenario: Output is too long
- **GIVEN** sanitized output exceeds the configured line limit
- **WHEN** the truncator runs
- **THEN** the stored trace keeps the first 30 percent and last 70 percent of allowed lines
