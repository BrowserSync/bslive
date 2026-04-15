# Filesystem notify audit (`bsnext_fs`)

This folder holds **machine-generated logs** and **human summaries** for what `notify` emits on each OS and how `platform_accepts` in `crates/bsnext_fs/src/watcher.rs` classifies those events.

The goal is **rich, ordered logs** you can use both for **scripted benchmarks** and for **long manual sessions** (different editors, bulk git operations, etc.), then derive UX-level patterns from the trace stream later—not to bake policy into this layer.

## Run the same experiment everywhere

From the **repository root** (the worktree root is fine):

```bash
chmod +x scripts/fs-notify-audit.sh   # once
./scripts/fs-notify-audit.sh
```

### Env (cross-platform)

| Env | Purpose |
|-----|---------|
| `RUST_LOG` | `tracing` filter; default `bsnext_fs=trace`. Same variable on every OS. |
| `FS_AUDIT_ROOT` | Watched directory (default `target/fs-notify-audit-root`). |
| `FS_AUDIT_OUTSIDE` | Directory **outside** the watch root for move-in / move-out steps (default `target/fs-notify-audit-outside`). |
| `FS_AUDIT_LOG` | Log file (default `audit/fs-notify/logs/raw-<uname -s lower>.log`). |
| `FS_AUDIT_WATCH_ONLY` | `1` = prepare `FS_AUDIT_ROOT` + `FS_AUDIT_OUTSIDE`, then run the watcher in the **foreground** with stderr **tee’d** to the log and your terminal—**no** automated `AUDIT_STEP` sequence. For manual editor testing. |

**Narrow platform decision lines only:**

```bash
RUST_LOG=bsnext_fs::platform_accepts=trace,bsnext_fs::watcher=trace ./scripts/fs-notify-audit.sh
```

**Manual / long-running (same `RUST_LOG` knobs):**

```bash
FS_AUDIT_WATCH_ONLY=1 RUST_LOG=bsnext_fs=trace ./scripts/fs-notify-audit.sh
```

Stop with Ctrl+C; stderr continues to append to `FS_AUDIT_LOG`.

**Windows:** use Git Bash, WSL, or translate the script; env vars and `RUST_LOG` behave the same. Some steps (`chmod`, `touch -a`) may no-op or log `AUDIT_NOTE`—that is still useful signal.

## What the automated script does

1. Recreates **`FS_AUDIT_ROOT`** and **`FS_AUDIT_OUTSIDE`** (initial files for move/symlink tests).
2. Runs `cargo run -p bsnext_fs --example fs_watcher_audit <FS_AUDIT_ROOT>` (recursive watch on that root only).
3. Inserts **`AUDIT_STEP …`** markers into the log, then runs steps in order:

| Step | Intent (UX-level) |
|------|-------------------|
| **content-changed** | Append bytes to an existing file |
| **touched** | `touch` existing file |
| **file-added** | Create new file |
| **file-removed** | Delete file |
| **folder-added** | `mkdir` |
| **folder-removed** | `rmdir` |
| **file-renamed-within-watch** | `mv` file inside watch root |
| **folder-renamed-within-watch** | `mv` directory inside watch root |
| **symlink-file-created** | `ln -s` to a path inside the watch root |
| **symlink-write-through** | Append via symlink (same inode story as editor writes) |
| **symlink-target-outside-watch** | Symlink into watch root pointing at **`FS_AUDIT_OUTSIDE`** |
| **symlink-outside-target-modified** | Change the outside file (resolution / monorepo style) |
| **chmod-executable-bit** | `chmod +x` / `chmod -x` (metadata-focused); failures logged on odd FS |
| **touch-access-time-only** | `touch -a` when available (atime vs mtime); skip note if unsupported |
| **move-file-into-watch-root** | `mv` from **`FS_AUDIT_OUTSIDE` → inside** watch root |
| **move-file-out-of-watch-root** | `mv` from inside watch root → **`FS_AUDIT_OUTSIDE`** |

4. Writes the raw trace log to **`FS_AUDIT_LOG`**.

Atomic save (temp + rename onto original) is intentionally **not** scripted yet.

## Interpreting the log

- **`AUDIT_STEP …`** — human/script anchor for “what we meant to do” vs the raw burst of events that follows.
- **`bsnext_fs::platform_accepts`** — which branch ran and `accept=true|false` for the current OS label (`platform="macos"`, `platform="unix"`, or `platform="windows"`).
- **`[accept]` / `[not-accepted]`** — after filters, whether an `InnerChangeEvent` was sent (see `watcher.rs` callback).

POSIX policy today still **rejects** top-level `Create` and `Remove`; many real-world creates/removes may show up as **`Modify`** on some backends—use the raw `Event { kind: … }` lines to see what the OS sent.

Verbose logs from **git checkouts**, **installs**, or **manual edits** in `FS_AUDIT_WATCH_ONLY` mode are expected to be **large**; that volume is the point for later UX-level grouping.

## Reports

| File | Purpose |
|------|---------|
| `report-darwin-vs-linux.md` | Side-by-side summary of two captured runs (Darwin + Linux); **start here** for cross-OS policy gaps. |
| `report-darwin.md` | macOS-only step table from an earlier script revision (re-run script and refresh if needed). |

Raw logs under `logs/` are **gitignored** (regenerate on each machine). After `./scripts/fs-notify-audit.sh`, open `logs/raw-<kernel>.log` for analysis.
