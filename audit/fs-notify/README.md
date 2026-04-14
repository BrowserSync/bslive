# Filesystem notify audit (`bsnext_fs`)

This folder holds **machine-generated logs** and **human summaries** for what `notify` emits on each OS and how `platform_accepts` in `crates/bsnext_fs/src/watcher.rs` classifies those events.

## Run the same experiment everywhere

From the **repository root** (the worktree root is fine):

```bash
chmod +x scripts/fs-notify-audit.sh   # once
./scripts/fs-notify-audit.sh
```

Defaults:

| Env | Purpose |
|-----|---------|
| `RUST_LOG` | `tracing` filter; default `bsnext_fs=trace`. Same variable on all platforms. |
| `FS_AUDIT_ROOT` | Watch directory (default `target/fs-notify-audit-root`). |
| `FS_AUDIT_LOG` | Log file (default `audit/fs-notify/logs/raw-<uname -s lower>.log`). |

**Narrow platform decision lines only:**

```bash
RUST_LOG=bsnext_fs::platform_accepts=trace,bsnext_fs::watcher=trace ./scripts/fs-notify-audit.sh
```

**Windows:** use Git Bash, WSL, or translate the script; env vars and `RUST_LOG` behave the same.

## What the script does

1. Recreates `FS_AUDIT_ROOT` with `existing.txt`.
2. Runs `cargo run -p bsnext_fs --example fs_watcher_audit <root>` (recursive watch on that root).
3. Inserts `AUDIT_STEP …` markers into the log, then for each step:
   - **content-changed** — append bytes to `existing.txt`
   - **touched** — `touch existing.txt`
   - **file-added** — create `audit-new-file.txt`
   - **file-removed** — delete that file
   - **folder-added** — `mkdir audit-new-dir`
   - **folder-removed** — `rmdir audit-new-dir`

4. Writes a raw trace log to `FS_AUDIT_LOG`.

## Interpreting the log

- **`bsnext_fs::platform_accepts`** — which branch ran and `accept=true|false` for the current OS label (`platform="macos"`, `platform="unix"`, or `platform="windows"`).
- **`[accept]` / `[not-accepted]`** — after filters, whether an `InnerChangeEvent` was sent (see `watcher.rs` callback).

POSIX policy today still **rejects** top-level `Create` and `Remove`; many real-world creates/removes may show up as **`Modify`** on some backends—use the raw `Event { kind: … }` lines to see what the OS sent.

## Reports

| File | Platform |
|------|----------|
| `report-darwin.md` | macOS (representative run; table from a captured log) |

Raw logs under `logs/` are **gitignored** (regenerate on each machine). After `./scripts/fs-notify-audit.sh`, open `logs/raw-<kernel>.log` and refresh the markdown table if needed.
