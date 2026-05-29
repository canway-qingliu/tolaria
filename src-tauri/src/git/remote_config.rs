use std::path::Path;

use super::command::{git_output, stderr_or_failure, stdout_lines};

const _DEFAULT_FETCH_REFSPEC: &str = "+refs/heads/*:refs/remotes/origin/*";
const REMOTE_URL_CONFIG_PATTERN: &str = r"^remote\..*\.url$";
// Constants retained for backwards-compatible schema; actual remote config uses dynamic keys
// via configure_remote(vault, remote_name, remote_url).
const _ORIGIN_URL_CONFIG_KEY: &str = "remote.origin.url";
const _ORIGIN_FETCH_CONFIG_KEY: &str = "remote.origin.fetch";

pub(super) fn has_configured_remote(vault: &Path) -> Result<bool, String> {
    Ok(!list_configured_remotes(vault)?.is_empty())
}

pub(super) fn list_configured_remotes(vault: &Path) -> Result<Vec<String>, String> {
    let output = git_output(
        vault,
        &["config", "--get-regexp", REMOTE_URL_CONFIG_PATTERN],
    )
    .map_err(|e| format!("Failed to inspect git remotes: {e}"))?;

    if output.status.code() == Some(1) {
        return Ok(Vec::new());
    }
    if !output.status.success() {
        return Err(stderr_or_failure("git config --get-regexp", &output));
    }

    Ok(stdout_lines(&output)
        .into_iter()
        .filter_map(|line| remote_name_from_url_config(&line))
        .collect())
}

fn remote_name_from_url_config(line: &str) -> Option<String> {
    let (key, value) = line.split_once(' ')?;
    if value.trim().is_empty() {
        return None;
    }
    key.strip_prefix("remote.")
        .and_then(|name| name.strip_suffix(".url"))
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
}

pub(super) fn configure_origin_remote(vault: &Path, remote_url: &str) -> Result<(), String> {
    configure_remote(vault, "origin", remote_url)
}

pub(super) fn configure_remote(vault: &Path, remote_name: &str, remote_url: &str) -> Result<(), String> {
    let url_key = format!("remote.{}.url", remote_name);
    let fetch_key = format!("remote.{}.fetch", remote_name);
    let fetch_refspec = format!("+refs/heads/*:refs/remotes/{}/*", remote_name);
    run_git_config(vault, &url_key, remote_url)?;
    run_git_config(vault, &fetch_key, &fetch_refspec)
}

fn run_git_config(vault: &Path, key: &str, value: &str) -> Result<(), String> {
    let output = git_output(vault, &["config", "--local", "--replace-all", key, value])
        .map_err(|e| format!("Failed to run git config: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    Err(stderr_or_failure("git config", &output))
}
