# Design: Data-Driven Sanitizer Engine

Instead of hardcoding rules, the FaaS embeds `filters.json` at compile time using Rust's `include_str!()`. 

## 1. The Generic Execution Engine
When `ToolCall::RunCommand(cmd)` returns an output:
1. The engine iterates through the JSON rules loaded via `std::sync::LazyLock`.
```json 
{
  "filters": [
    {
      "id": "cargo",
      "match_command": "^cargo\\s+",
      "strip_ansi": true,
      "strip_lines_matching": [
        "^\\s*(Compiling|Downloaded|Downloading|Updating|Blocking waiting for file lock)",
        "^\\s*Finished\\s+(release|dev)",
        "^\\s*Building\\s+\\["
      ],
      "max_lines": 100
    },
    {
      "id": "git-status",
      "match_command": "^git\\s+status",
      "strip_ansi": true,
      "strip_lines_matching": [
        "^On branch",
        "^Your branch is up to date",
        "^\\s*\\(use \"git",
        "^\\s*\\(commands in",
        "^nothing to commit"
      ],
      "max_lines": 50
    },
    {
      "id": "npm-yarn-pnpm",
      "match_command": "^(npm|yarn|pnpm|npx)\\s+",
      "strip_ansi": true,
      "strip_lines_matching": [
        "^npm (WARN|notice)",
        "^\\s*\\[\\.{3,}\\]",
        "^added \\d+ packages",
        "^Done in \\d+(\\.\\d+)?s",
        "npm ERR! A complete log of this run can be found in"
      ],
      "max_lines": 100
    },
    {
      "id": "gradle",
      "match_command": "^(gradle|\\./gradlew)\\s+",
      "strip_ansi": true,
      "strip_lines_matching": [
        "^> Task",
        "^> Transform",
        "^> \\d+%",
        "^Download(ing|ed)",
        "^BUILD SUCCESSFUL in",
        "^\\s*$"
      ],
      "max_lines": 100
    },
    {
      "id": "maven",
      "match_command": "^(mvn|\\./mvnw)\\s+",
      "strip_ansi": true,
      "strip_lines_matching": [
        "^\\[INFO\\] Download(ing|ed):",
        "^\\[INFO\\] ---",
        "^\\[INFO\\] BUILD SUCCESS",
        "^\\[INFO\\] Total time:"
      ],
      "max_lines": 100
    }
  ]
}
```
2. It stops at the first rule where `Regex::new(rule.match_command).is_match(cmd)` is true.
3. If `strip_ansi` is true, it removes terminal control characters (`\x1b\[[0-9;]*[a-zA-Z]`).
4. It condenses multiple blank lines (`\n{3,}`) into `\n\n`.
5. It iterates over the output lines, discarding any line that matches *any* pre-compiled regex in `strip_lines_matching`.
6. It applies `smart_truncate(max_lines)` to the remaining lines.
7. If no rule matches, it applies a default "Safety Filter" (ANSI strip + 100 lines truncate).


## 2. Smart Truncation Algorithm
If the output exceeds `max_lines` (e.g., 100 lines), we do not simply cut off the end (which contains the critical compiler error summary).
- **Strategy:** Keep the first 30% of `max_lines` (context), keep the last 70% of `max_lines` (summary/errors).
- Insert `\n[... {N} lines omitted ...]\n` between the two halves.