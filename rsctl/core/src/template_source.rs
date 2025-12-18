use anyhow::{anyhow, Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve the template root directory.
///
/// - `None` or empty => local workspace templates (`templates/`)
/// - local path => that directory
/// - git/http/ssh URL => clone and return cloned dir (preferring `<repo>/templates` if present)
pub fn resolve_template_root(remote: Option<&str>) -> Result<PathBuf> {
    match remote {
        None => Ok(PathBuf::from("templates")),
        Some(s) if s.trim().is_empty() => Ok(PathBuf::from("templates")),
        Some(s) => {
            let s = s.trim();
            if looks_like_url_or_git(s) {
                let dir = clone_remote_templates(s)?;
                let repo_templates = dir.join("templates");
                if repo_templates.is_dir() {
                    Ok(repo_templates)
                } else {
                    Ok(dir)
                }
            } else {
                Ok(PathBuf::from(s))
            }
        }
    }
}

fn looks_like_url_or_git(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("git@")
        || s.starts_with("ssh://")
        || s.ends_with(".git")
}

fn clone_remote_templates(remote: &str) -> Result<PathBuf> {
    let tmp = std::env::temp_dir();
    let repo_name = remote_repo_basename(remote).unwrap_or_else(|| "templates".to_string());
    let uniq = format!(
        "{}-{}",
        repo_name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let dest = tmp.join("rsctl").join("repos").join(uniq);
    fs::create_dir_all(&dest)
        .with_context(|| format!("failed to create temp dir: {}", dest.display()))?;

    let status = Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(remote)
        .arg(&dest)
        .status()
        .with_context(|| "failed to execute `git clone` (is git installed and in PATH?)")?;

    if !status.success() {
        return Err(anyhow!("git clone failed for remote: {remote}"));
    }

    Ok(dest)
}

fn remote_repo_basename(remote: &str) -> Option<String> {
    let last = remote
        .rsplit(|c| c == '/' || c == ':')
        .next()
        .and_then(|s| if s.is_empty() { None } else { Some(s) })?;

    Some(
        Path::new(last)
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or(last)
            .to_string(),
    )
}

pub fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}


