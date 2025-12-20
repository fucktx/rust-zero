use std::path::PathBuf;
use anyhow::{anyhow, Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf as StdPathBuf};
use std::process::Command;

/// Resolve default template root directory from user's home.
///
/// Lookup order:
/// 1) `~/.rsctl/templates` (preferred)
/// 2) `~/.rsctl`
/// 3) search upwards from current dir for `templates/` (repo/workspace local)
/// 4) `None` (caller may fallback to `templates/` by itself)
pub fn default_template_root() -> Option<PathBuf> {
    let home = home_dir()?;
    let rsctl_home = home.join(".rsctl");
    let rsctl_templates = rsctl_home.join("templates");
    if rsctl_templates.is_dir() {
        Some(rsctl_templates)
    } else if rsctl_home.is_dir() {
        Some(rsctl_home)
    } else {
        find_templates_upwards()
    }
}

/// Resolve the template root directory.
///
/// - `None` or empty => default template root (see `default_template_root`)
/// - local path => that directory
/// - git/http/ssh URL => clone and return cloned dir (preferring `<repo>/templates` if present)
pub fn resolve_template_root(remote: Option<&str>) -> Result<StdPathBuf> {
    match remote {
        None => Ok(default_template_root().unwrap_or_else(|| StdPathBuf::from("templates"))),
        Some(s) if s.trim().is_empty() => {
            Ok(default_template_root().unwrap_or_else(|| StdPathBuf::from("templates")))
        }
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
                Ok(StdPathBuf::from(s))
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

fn clone_remote_templates(remote: &str) -> Result<StdPathBuf> {
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

fn find_templates_upwards() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join("templates");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn home_dir() -> Option<PathBuf> {
    // Unix: HOME
    if let Some(h) = std::env::var_os("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    // Windows: USERPROFILE, or HOMEDRIVE + HOMEPATH
    if let Some(h) = std::env::var_os("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    let drive = std::env::var_os("HOMEDRIVE");
    let path = std::env::var_os("HOMEPATH");
    match (drive, path) {
        (Some(d), Some(p)) if !d.is_empty() && !p.is_empty() => Some(PathBuf::from(d).join(p)),
        _ => None,
    }
}


