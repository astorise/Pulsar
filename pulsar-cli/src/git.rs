use anyhow::{bail, Context};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sandbox {
    pub repo_root: PathBuf,
    pub worktree_path: PathBuf,
    pub branch: String,
}

impl Sandbox {
    pub fn create(cwd: &Path, session_id: &str) -> anyhow::Result<Option<Self>> {
        let Some(repo_root) = repo_root(cwd)? else {
            return Ok(None);
        };

        ensure_local_exclude(&repo_root)?;

        let safe_id = sanitize_session_id(session_id);
        let branch = format!("pulsar/sess-{safe_id}");
        let worktree_path = repo_root.join(".pulsar").join("worktrees").join(&safe_id);
        if let Some(parent) = worktree_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        run_git(
            &repo_root,
            &[
                "worktree",
                "add",
                "-b",
                &branch,
                path_arg(&worktree_path).as_str(),
                "HEAD",
            ],
        )?;

        Ok(Some(Self {
            repo_root,
            worktree_path,
            branch,
        }))
    }

    pub fn diff_stat(&self) -> anyhow::Result<String> {
        run_git(&self.worktree_path, &["diff", "--stat", "HEAD"])
    }

    pub fn apply_patch_to_repo(&self) -> anyhow::Result<()> {
        let patch = run_git_bytes(&self.worktree_path, &["diff", "--binary", "HEAD"])?;
        if patch.is_empty() {
            return Ok(());
        }

        let mut child = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(["apply", "--index"])
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn git apply")?;

        {
            use std::io::Write;
            let Some(stdin) = child.stdin.as_mut() else {
                bail!("failed to open git apply stdin");
            };
            stdin
                .write_all(&patch)
                .context("failed to send patch to git apply")?;
        }

        let output = child.wait_with_output().context("git apply failed")?;
        if !output.status.success() {
            bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
        }

        Ok(())
    }

    pub fn cleanup(&self) -> anyhow::Result<()> {
        let remove = run_git(
            &self.repo_root,
            &[
                "worktree",
                "remove",
                "--force",
                path_arg(&self.worktree_path).as_str(),
            ],
        );
        let branch = run_git(&self.repo_root, &["branch", "-D", &self.branch]);

        remove.and(branch).map(|_| ())
    }
}

pub fn merge_worktree(repo_root: &Path, worktree_branch: &str) -> anyhow::Result<Vec<String>> {
    let merge = run_git(
        repo_root,
        &["merge", worktree_branch, "--no-commit", "--no-ff"],
    );
    if merge.is_err() {
        return conflicted_files(repo_root);
    }
    conflicted_files(repo_root)
}

pub fn conflicted_files(repo_root: &Path) -> anyhow::Result<Vec<String>> {
    let output = run_git(repo_root, &["diff", "--name-only", "--diff-filter=U"])?;
    Ok(output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

pub fn repo_root(cwd: &Path) -> anyhow::Result<Option<PathBuf>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to run git rev-parse")?;

    if !output.status.success() {
        return Ok(None);
    }

    let root = String::from_utf8(output.stdout)
        .context("git rev-parse returned non UTF-8 output")?
        .trim()
        .to_string();
    Ok(Some(PathBuf::from(root)))
}

pub fn ensure_local_exclude(repo_root: &Path) -> anyhow::Result<()> {
    let exclude = repo_root.join(".git").join("info").join("exclude");
    let existing = fs::read_to_string(&exclude).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == ".pulsar/") {
        return Ok(());
    }

    let mut next = existing;
    if !next.ends_with('\n') && !next.is_empty() {
        next.push('\n');
    }
    next.push_str(".pulsar/\n");
    fs::write(&exclude, next).with_context(|| format!("failed to update {}", exclude.display()))
}

pub fn sanitize_session_id(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "session".to_string()
    } else {
        trimmed.chars().take(48).collect()
    }
}

fn run_git(cwd: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;

    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_git_bytes(cwd: &Path, args: &[&str]) -> anyhow::Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;

    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(output.stdout)
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_is_git_branch_safe() {
        assert_eq!(
            sanitize_session_id("secret_123 ABC/../x"),
            "secret-123-abc-x"
        );
    }

    #[test]
    fn empty_session_id_has_fallback() {
        assert_eq!(sanitize_session_id("!!!"), "session");
    }

    #[test]
    fn conflict_output_can_be_parsed() {
        let files = "src/lib.rs\nCargo.toml\n"
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(files, vec!["src/lib.rs", "Cargo.toml"]);
    }
}
