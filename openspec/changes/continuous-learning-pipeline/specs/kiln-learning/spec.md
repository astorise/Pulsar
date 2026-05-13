# Specification: Native Fine-Tuning Protocol

## Requirement: Success-Only Filtering
The dataset MUST only contain the optimal, corrected path to the solution to prevent the model from learning negative behaviors.

## Requirement: Native WIT Integration
[cite_start]The `faas/skill-extractor` MUST interact with the Host's fine-tuning capabilities via the standard `tachyon:ai/training` interface, completely avoiding external CLI calls.

## WIT Interface Integration
```wit
package tachyon:ai@1.0.0;

interface training {
    /// Soumet une tâche d'entraînement LoRA asynchrone (Fire-and-Forget).
    submit-job: func(dataset-path: string, base-model: string) -> result<string, string>;
}
```