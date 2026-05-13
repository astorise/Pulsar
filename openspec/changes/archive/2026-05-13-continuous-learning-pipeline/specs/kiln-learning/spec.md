## ADDED Requirements

### Requirement: Success-Only Trace Filtering
The skill extractor SHALL remove failed attempts from completed traces before turning them into training examples.

#### Scenario: Trace contains failed compiler loops
- **GIVEN** a trace includes failed commands followed by a successful solution
- **WHEN** the extractor filters the trace
- **THEN** only the corrected path to the solution remains

### Requirement: ShareGPT Dataset Formatting
The skill extractor SHALL format filtered traces as ShareGPT-style conversations before appending them to the dataset.

#### Scenario: Training example is generated
- **GIVEN** a filtered task trace
- **WHEN** the extractor builds the dataset entry
- **THEN** the entry contains alternating conversation messages suitable for fine-tuning

### Requirement: Native Training Trigger
The skill extractor SHALL call the host `training.submit-job` interface when the dataset reaches the configured threshold.

#### Scenario: Dataset reaches threshold
- **GIVEN** enough examples have been generated
- **WHEN** the threshold is met
- **THEN** a LoRA training job is submitted through the Tachyon host interface
