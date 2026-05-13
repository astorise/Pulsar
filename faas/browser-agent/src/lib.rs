use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionCheck {
    pub passed: bool,
    pub findings: Vec<String>,
}

pub fn build_vision_prompt(task: &str) -> String {
    format!(
        "Inspect the screenshot for the UI task below. Return JSON with passed and findings.\n\nTask:\n{task}"
    )
}

pub fn parse_vision_check(response: &str) -> Result<VisionCheck, String> {
    serde_json::from_str(response).map_err(|err| err.to_string())
}

pub fn smolvm_rootfs_packages() -> Vec<&'static str> {
    vec!["alpine-base", "chromium", "xvfb-run", "ttf-dejavu"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vision_prompt_contains_task() {
        assert!(build_vision_prompt("fix button overlap").contains("fix button overlap"));
    }

    #[test]
    fn vision_response_parses() {
        let check = parse_vision_check(r#"{"passed":true,"findings":[]}"#).unwrap();
        assert!(check.passed);
    }
}
