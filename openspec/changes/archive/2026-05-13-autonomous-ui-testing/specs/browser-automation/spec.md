## ADDED Requirements

### Requirement: Browser CDP Session Interface
The host SHALL expose a `tachyon:browser/cdp` session resource with navigation, JavaScript evaluation, screenshot, and lifecycle operations.

#### Scenario: Browser agent inspects a preview
- **GIVEN** a UI task has a local preview URL
- **WHEN** the browser agent opens a CDP session
- **THEN** it can evaluate JavaScript and capture screenshots

### Requirement: MicroVM Ephemerality
When using the `smolvm` engine profile, the host SHALL destroy the microVM when the CDP session is dropped or times out.

#### Scenario: CDP session ends
- **GIVEN** a microVM-backed browser session is active
- **WHEN** the resource is dropped
- **THEN** the microVM is destroyed without persistent browser state

### Requirement: Vision Validation
The supervisor SHALL route UI-changing sub-tasks through a browser agent and require a vision-model check before merging.

#### Scenario: UI sub-task completes
- **GIVEN** a child task changed HTML, CSS, or JS
- **WHEN** reduce evaluates the child result
- **THEN** a screenshot is validated before merge
