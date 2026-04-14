# FS notify audit — **Darwin vs Linux (side-by-side)**

This document compares two runs of **`scripts/fs-notify-audit.sh`** with the same tracing defaults (`RUST_LOG=bsnext_fs=trace`) and the **current** `platform_accepts` behaviour in `crates/bsnext_fs/src/watcher.rs`. Raw logs are gitignored; paths below are where those runs lived when this file was written.

| | **Darwin** | **Linux** |
|--|------------|-----------|
| **Log** | `logs/raw-darwin.log` | `logs/raw-linux.log` |
| **Host (from log header)** | `aarch64-apple-darwin`, Darwin 24.6.0 (arm64) | `x86_64-unknown-linux-gnu`, Ubuntu 6.8.0-1044-azure |
| **`platform_accepts` label** | `platform="macos"` | `platform="unix"` |

## Forwarding summary (what actually became `InnerChangeEvent`)

| Metric | Darwin | Linux |
|--------|--------|-------|
| **`[accept]` by `EventKind`** | `Modify(Data(Content))` ×13, `Modify(Metadata(Any))` ×2 | `Modify(Metadata(Any))` ×3 only |
| **`InnerChangeEvent` lines** | 15 | 3 |

**Linux:** no `Modify(Data(Content))` appeared in the `[accept]` stream at all. inotify delivered **`Modify(Data(Any))`** for essentially every “bytes changed” scenario; that arm is **`accept=false`** today because only **`DataChange::Content`** is accepted on POSIX.

## Headline finding

- **Darwin:** the current gate is a plausible stand-in for “file work happened”: most scripted edits show up as **`Modify(Data(Content))`** and get forwarded, with **`Modify(Metadata(Any))`** for things like `touch`.
- **Linux:** the same gate **drops almost all data edits** in this capture (**`Data(Any)`** rejected) while still forwarding some **`Metadata(Any)`** (here: **`touched`** on `existing.txt`, and **two** metadata accepts on **`meta-chmod.sh`** during the chmod step).

So identical UX steps produce **very different forwarded counts** between these two logs—not because the script differed, but because **notify’s classification differs** and **`Content` vs `Any`** is decisive for the current matcher.

## Linux: where the three forwards came from

| Order | Step (approx.) | Forwarded `EventKind` | Path |
|-------|----------------|------------------------|------|
| 1 | **touched** | `Modify(Metadata(Any))` | `existing.txt` |
| 2–3 | **chmod-executable-bit** | `Modify(Metadata(Any))` (twice) | `meta-chmod.sh` |

Steps such as **content-changed**, **file-added**, **symlink-write-through**, **file-renamed**, **move-out**, etc. still produced **`Modify(Data(Any))`** lines in the trace, but **none** were `[accept]`’d in this run.

## Shapes Linux showed more of (still all `accept=false` with current rules)

- **`Access(Close(Write))`** — appears around writes; not accepted.
- **Renames:** **`Modify(Name(From))`**, **`Modify(Name(To))`**, **`Modify(Name(Both))`** — richer than Darwin’s **`Modify(Name(Any))`** in the other log, but still rejected by policy today.

Darwin still logged plenty of **`Create` / `Remove` / `Modify(Metadata(Extended))`** as `[not-accepted]`; that part is qualitatively similar.

## Regenerate and refresh this comparison

```bash
./scripts/fs-notify-audit.sh
# Darwin: audit/fs-notify/logs/raw-darwin.log
# Linux:  audit/fs-notify/logs/raw-linux.log
```

Re-count `[accept]` / `InnerChangeEvent` / `platform_accepts` lines from those files and adjust the numbers in this doc if kernels or notify backends change.

See also **`report-darwin.md`** for a step-by-step Darwin-only table from an earlier scripted layout (fewer `AUDIT_STEP`s than the current script).
