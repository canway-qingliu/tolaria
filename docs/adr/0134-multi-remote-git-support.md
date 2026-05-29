# ADR 0134: Multi-Remote Git Support

## Status

Accepted

## Context

Tolaria has historically supported exactly one Git remote per vault, hardcoded as `"origin"`. All sync operations (`git_pull`, `git_push`, `git_remote_status`) operated on this single remote. Users who want to mirror vaults to multiple providers (e.g., push to both GitHub and GitLab, or maintain a backup remote) could not do so.

This ADR covers the changes needed throughout the stack to support **multiple named remotes** per vault, with parallel push/pull fan-out.

## Decision

### Backend

1. **`git_remote_status()`** now iterates all configured remotes (via `list_configured_remotes()`) and computes `ahead`/`behind` for each via `HEAD...refs/remotes/<name>/<branch>`. The return type embeds a `Vec<RemoteInfo>`:

   ```rust
   pub struct RemoteInfo {
       pub name: String,
       pub ahead: u32,
       pub behind: u32,
   }

   pub struct GitRemoteStatus {
       pub branch: String,
       pub ahead: u32,   // sum across all remotes
       pub behind: u32,  // sum across all remotes
       pub has_remote: bool,
       pub remotes: Vec<RemoteInfo>,
   }
   ```

   The aggregate `ahead`/`behind` fields are retained for backward-compatible status-bar display. The `remotes` array provides per-remote detail for the UI.

2. **`git_pull_remote(vault_path, remote_name)`** and **`git_push_remote(vault_path, remote_name)`** are added as explicit per-remote variants. The original `git_pull`/`git_push` still operates on all remotes sequentially (for backward compatibility with the auto-sync loop that calls them).

3. **`git_add_remote()`** no longer rejects when a remote already exists — users can add as many remotes as they want.

4. **`git_add_remote_named(vault_path, remote_name, remote_url)`** accepts an explicit remote name instead of hardcoding `"origin"`.

5. **`git_remove_remote(vault_path, remote_name)`** removes a specific remote.

6. **`git_list_remotes(vault_path)`** → `Vec<String>` returns all configured remote names for a vault.

7. `configure_origin_remote()` is generalized to `configure_remote(vault, remote_name, remote_url)` which derives config keys dynamically from the provided name.

### Frontend

8. **`GitRemoteStatus`** TypeScript type gains a `remotes: { name: string; ahead: number; behind: number }[]` field.

9. **`aggregateRemoteStatuses()`** in `useAutoSync.ts` is updated to aggregate `remotes[]` across vaults via `flatMap`.

10. **`RemoteStatusSummary`** (status bar popup) now renders one row per entry in `remotes[]` when there are multiple remotes, showing the remote name and its own ahead/behind state. Falls back to the single-remote aggregate display for backward compatibility.

11. **`PullAction`** shows one pull button per remote when `remotes.length > 1`, each labeled with the remote name.

12. **`AddRemoteModal`** now:
    - Fetches and displays the list of currently configured remotes on open.
    - Adds an optional "Remote Name" input field (defaults to blank → uses `"origin"`).
    - Shows a Remove button next to each listed remote.
    - Calls `git_add_remote_named` when a name is provided, otherwise falls back to `git_add_remote`.

## Consequences

- Existing single-remote workflows remain fully functional — all existing commands operate on all remotes or on the first remote in the list.
- The UI gracefully degrades when `remotes[]` is empty or absent (shows "No remote configured").
- Push/pull fan-out is sequential, not parallel, to avoid saturating git auth helpers. This is a deliberate choice given the low fan-out factor (typically 2–3 remotes).
- The `ahead`/`behind` fields in `GitRemoteStatus` now represent the sum across all remotes for display purposes. A vault with `origin` 2 ahead and `backup` 1 ahead will show `ahead: 3`.

## Alternatives Considered

- **Parallel push/pull**: Using `Promise.all` to fan out to all remotes simultaneously. Rejected because git auth helpers (SSH keys, GCM) may prompt or have rate limits, and sequential execution provides cleaner error reporting per remote.
- **Per-remote status-only commands**: Rather than embedding `remotes[]` in `GitRemoteStatus`, we could have a separate `git_remote_statuses()` command returning `Vec<GitRemoteStatus>`. Rejected because the single-status display in the status bar would then need to aggregate multiple `GitRemoteStatus` objects anyway — embedding `remotes[]` is simpler.