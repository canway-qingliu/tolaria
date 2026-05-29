use crate::git::GitAddRemoteResult;
use serde::Deserialize;

use super::expand_tilde;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitAddRemoteRequest {
    vault_path: String,
    remote_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitAddRemoteNamedRequest {
    vault_path: String,
    remote_name: String,
    remote_url: String,
}

#[cfg(desktop)]
#[tauri::command]
pub async fn git_add_remote(request: GitAddRemoteRequest) -> Result<GitAddRemoteResult, String> {
    let vault_path = expand_tilde(&request.vault_path).into_owned();
    let remote_url = request.remote_url;
    tokio::task::spawn_blocking(move || crate::git::git_add_remote(&vault_path, &remote_url))
        .await
        .map_err(|e| format!("Task panicked: {e}"))?
}

#[cfg(desktop)]
#[tauri::command]
pub async fn git_add_remote_named(
    request: GitAddRemoteNamedRequest,
) -> Result<GitAddRemoteResult, String> {
    let vault_path = expand_tilde(&request.vault_path).into_owned();
    let remote_name = request.remote_name;
    let remote_url = request.remote_url;
    tokio::task::spawn_blocking(move || {
        crate::git::git_add_remote_named(&vault_path, &remote_name, &remote_url)
    })
    .await
    .map_err(|e| format!("Task panicked: {e}"))?
}

#[cfg(desktop)]
#[tauri::command]
pub async fn git_remove_remote(vault_path: String, remote_name: String) -> Result<(), String> {
    let vault_path = expand_tilde(&vault_path).into_owned();
    tokio::task::spawn_blocking(move || crate::git::git_remove_remote(&vault_path, &remote_name))
        .await
        .map_err(|e| format!("Task panicked: {e}"))?
}

#[cfg(mobile)]
#[tauri::command]
pub async fn git_add_remote(_request: GitAddRemoteRequest) -> Result<GitAddRemoteResult, String> {
    Err("Adding git remotes is not available on mobile".into())
}

#[cfg(mobile)]
#[tauri::command]
pub async fn git_add_remote_named(
    _request: GitAddRemoteNamedRequest,
) -> Result<GitAddRemoteResult, String> {
    Err("Adding git remotes is not available on mobile".into())
}

#[cfg(mobile)]
#[tauri::command]
pub async fn git_remove_remote(_vault_path: String, _remote_name: String) -> Result<(), String> {
    Err("Removing git remotes is not available on mobile".into())
}
