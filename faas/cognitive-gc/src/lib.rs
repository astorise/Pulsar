use serde::{Deserialize, Serialize};

pub const SIMILARITY_THRESHOLD: f32 = 0.85;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub key: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedObservation {
    pub observation: Observation,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JudgeResult {
    pub has_contradiction: bool,
    pub golden_fact: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicSwapPlan {
    pub deletes: Vec<String>,
    pub insert_key: Option<String>,
    pub insert_value: Option<String>,
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(l, r)| l * r)
        .sum::<f32>();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

pub fn cluster_similar(
    observations: &[EmbeddedObservation],
    threshold: f32,
) -> Vec<Vec<EmbeddedObservation>> {
    let mut used = vec![false; observations.len()];
    let mut groups = Vec::new();

    for (idx, item) in observations.iter().enumerate() {
        if used[idx] {
            continue;
        }
        let mut group = vec![item.clone()];
        used[idx] = true;
        for (other_idx, other) in observations.iter().enumerate().skip(idx + 1) {
            if !used[other_idx] && cosine_similarity(&item.embedding, &other.embedding) >= threshold
            {
                used[other_idx] = true;
                group.push(other.clone());
            }
        }
        if group.len() > 1 {
            groups.push(group);
        }
    }

    groups
}

pub fn build_judge_prompt(group: &[EmbeddedObservation]) -> String {
    let facts = group
        .iter()
        .map(|item| format!("- {}", item.observation.content))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "You are the Memory Judge. Identify contradictions and return JSON {{\"has_contradiction\":bool,\"golden_fact\":string|null}}.\n\nFacts:\n{facts}"
    )
}

pub fn parse_judge_result(response: &str) -> Result<JudgeResult, String> {
    serde_json::from_str(response).map_err(|err| err.to_string())
}

pub fn compaction_plan(group: &[EmbeddedObservation], judge: &JudgeResult) -> AtomicSwapPlan {
    if !judge.has_contradiction {
        return AtomicSwapPlan {
            deletes: Vec::new(),
            insert_key: None,
            insert_value: None,
        };
    }

    AtomicSwapPlan {
        deletes: group
            .iter()
            .map(|item| item.observation.key.clone())
            .collect(),
        insert_key: judge
            .golden_fact
            .as_ref()
            .map(|_| "v1:mcp:golden".to_string()),
        insert_value: judge.golden_fact.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embedded(key: &str, content: &str, embedding: Vec<f32>) -> EmbeddedObservation {
        EmbeddedObservation {
            observation: Observation {
                key: key.to_string(),
                content: content.to_string(),
            },
            embedding,
        }
    }

    #[test]
    fn similar_items_cluster() {
        let groups = cluster_similar(
            &[
                embedded("a", "port is 3000", vec![1.0, 0.0]),
                embedded("b", "port is 3001", vec![0.99, 0.01]),
                embedded("c", "unrelated", vec![0.0, 1.0]),
            ],
            SIMILARITY_THRESHOLD,
        );
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 2);
    }

    #[test]
    fn judge_result_parses_and_builds_swap() {
        let judge =
            parse_judge_result(r#"{"has_contradiction":true,"golden_fact":"port is 3000"}"#)
                .unwrap();
        let plan = compaction_plan(&[embedded("a", "old", vec![1.0])], &judge);
        assert_eq!(plan.deletes, vec!["a"]);
        assert_eq!(plan.insert_value.as_deref(), Some("port is 3000"));
    }
}
