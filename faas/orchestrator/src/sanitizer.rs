use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;

const MAX_LINES: usize = 120;

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]") {
    Ok(regex) => regex,
    Err(err) => panic!("valid ANSI regex: {err}"),
});

static FILTER_RULES: LazyLock<Vec<FilterRule>> = LazyLock::new(|| {
    let raw: Vec<RawFilterRule> = match serde_json::from_str(include_str!("../../filters.json")) {
        Ok(rules) => rules,
        Err(err) => panic!("valid filters.json: {err}"),
    };
    raw.into_iter()
        .filter(|rule| rule.enabled)
        .map(|rule| FilterRule {
            commands: rule.commands,
            regex: match Regex::new(&rule.pattern) {
                Ok(regex) => regex,
                Err(err) => panic!("valid filter regex: {err}"),
            },
        })
        .collect()
});

#[derive(Debug, Deserialize)]
struct RawFilterRule {
    #[serde(rename = "name")]
    _name: String,
    commands: Vec<String>,
    pattern: String,
    enabled: bool,
}

#[derive(Debug)]
struct FilterRule {
    commands: Vec<String>,
    regex: Regex,
}

pub fn clean_output(command: &str, raw_output: &str) -> String {
    let without_ansi = ANSI_RE.replace_all(raw_output, "");
    let mut blank_count = 0usize;
    let mut lines = Vec::new();

    for line in without_ansi.lines() {
        let trimmed = line.trim_end();
        if should_filter_line(command, trimmed) {
            continue;
        }

        if trimmed.trim().is_empty() {
            blank_count += 1;
            if blank_count > 2 {
                continue;
            }
        } else {
            blank_count = 0;
        }

        lines.push(trimmed.to_string());
    }

    smart_truncate(&lines.join("\n"), MAX_LINES)
}

pub fn smart_truncate(output: &str, max_lines: usize) -> String {
    let lines = output.lines().collect::<Vec<_>>();
    if lines.len() <= max_lines {
        return output.to_string();
    }

    let head_count = ((max_lines as f32) * 0.3).ceil() as usize;
    let tail_count = max_lines.saturating_sub(head_count).max(1);
    let mut kept = Vec::with_capacity(max_lines + 1);
    kept.extend(lines.iter().take(head_count).copied());
    kept.push("[... omitted ...]");
    kept.extend(
        lines
            .iter()
            .skip(lines.len().saturating_sub(tail_count))
            .copied(),
    );
    kept.join("\n")
}

fn should_filter_line(command: &str, line: &str) -> bool {
    FILTER_RULES
        .iter()
        .any(|rule| rule_matches_command(rule, command) && rule.regex.is_match(line))
}

fn rule_matches_command(rule: &FilterRule, command: &str) -> bool {
    let command = command.trim_start();
    rule.commands
        .iter()
        .any(|prefix| command == prefix || command.starts_with(&format!("{prefix} ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_and_filters_cargo_noise() {
        let cleaned = clean_output(
            "cargo test",
            "\u{1b}[31mCompiling crate v0.1.0\u{1b}[0m\nerror: expected item\n",
        );

        assert!(!cleaned.contains("Compiling"));
        assert!(cleaned.contains("error: expected item"));
        assert!(!cleaned.contains("\u{1b}"));
    }

    #[test]
    fn smart_truncate_keeps_head_and_tail() {
        let output = (0..10)
            .map(|idx| format!("line {idx}"))
            .collect::<Vec<_>>()
            .join("\n");
        let truncated = smart_truncate(&output, 5);

        assert!(truncated.contains("line 0"));
        assert!(truncated.contains("[... omitted ...]"));
        assert!(truncated.contains("line 9"));
    }
}
