# Implementation Tasks

- [x] **Task 1: Extend Skill Extractor WIT**
  - [cite_start]Import the `tachyon:ai/training` interface in the FaaS's `world.wit` file.

- [x] **Task 2: Implement the Trace Filter (The "Happy Path")**
  - In `faas/skill-extractor/src/lib.rs`, write an algorithm that iterates backwards through a session trace, stripping failed compiler attempts to produce a clean, direct path to the solution.

- [x] **Task 3: Implement ShareGPT Formatter**
  - Define the Serde structs for the ShareGPT format (`Conversation`, `Message`).
  - Map the filtered trace into this structure and append it to the dataset file.

- [x] **Task 4: Implement Training Trigger**
  - Monitor the dataset size. [cite_start]When it hits the threshold, call `tachyon::ai::training::submit_job(dataset_path, base_model)`.

- [x] **Task 5: Telemetry Feedback**
  - Update the CLI output to notify the developer: `[Kiln] 🎯 New training example generated. Dataset size: 500. 🚀 LoRA training job submitted to Tachyon host.`