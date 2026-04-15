#!/usr/bin/env bash
# Reproducible filesystem notify audit for bsnext_fs (notify-rs + platform_accepts).
#
# Cross-platform env (same on macOS, Linux, Git Bash on Windows):
#   RUST_LOG tracing filter (default: bsnext_fs=trace)
#   FS_AUDIT_ROOT         watched directory (default: <repo>/target/fs-notify-audit-root)
#   FS_AUDIT_OUTSIDE      sibling area *outside* the watch root, for move-in / move-out
#                         (default: <repo>/target/fs-notify-audit-outside)
#   FS_AUDIT_LOG          output log path (default: <repo>/audit/fs-notify/logs/raw-<kernel>.log)
#   FS_AUDIT_WATCH_ONLY   if1: set up trees, then run the watcher in the foreground with stderr
#                         copied to FS_AUDIT_LOG and the terminal (for manual editor experiments).
#                         No automated AUDIT_STEP sequence.
#
# Usage:
#   ./scripts/fs-notify-audit.sh
#   FS_AUDIT_LOG=/tmp/audit.log RUST_LOG=bsnext_fs=trace ./scripts/fs-notify-audit.sh
#   FS_AUDIT_WATCH_ONLY=1 ./scripts/fs-notify-audit.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

KERNEL="$(uname -s | tr '[:upper:]' '[:lower:]')"
AUDIT_ROOT="${FS_AUDIT_ROOT:-$REPO_ROOT/target/fs-notify-audit-root}"
AUDIT_OUTSIDE="${FS_AUDIT_OUTSIDE:-$REPO_ROOT/target/fs-notify-audit-outside}"
LOG="${FS_AUDIT_LOG:-$REPO_ROOT/audit/fs-notify/logs/raw-${KERNEL}.log}"
export RUST_LOG="${RUST_LOG:-bsnext_fs=trace}"
WATCH_ONLY="${FS_AUDIT_WATCH_ONLY:-0}"

prepare_audit_trees() {
  rm -rf "$AUDIT_ROOT" "$AUDIT_OUTSIDE"
  mkdir -p "$AUDIT_ROOT" "$AUDIT_OUTSIDE"
  printf '%s\n' 'seed line' >"$AUDIT_ROOT/existing.txt"
  printf '%s\n' 'outside-target' >"$AUDIT_OUTSIDE/outside-target.txt"
  printf '%s\n' 'payload-for-move-in' >"$AUDIT_OUTSIDE/brought-in.txt"
}

write_log_header() {
  {
    echo "======== fs-notify-audit started $(date -u +%Y-%m-%dT%H:%M:%SZ) ========"
    echo "uname: $(uname -a)"
    echo "RUST_LOG=$RUST_LOG"
    echo "FS_AUDIT_ROOT=$AUDIT_ROOT"
    echo "FS_AUDIT_OUTSIDE=$AUDIT_OUTSIDE"
    echo "FS_AUDIT_LOG=$LOG"
    echo "FS_AUDIT_WATCH_ONLY=$WATCH_ONLY"
    echo "=============================================================="
  } >>"$LOG"
}

audit_step() {
  {
    echo ""
    echo "======== AUDIT_STEP $1 $(date -u +%Y-%m-%dT%H:%M:%SZ) ========"
  } >>"$LOG"
}

mkdir -p "$(dirname "$LOG")"
rm -f "$LOG"
prepare_audit_trees
write_log_header

cargo build -q -p bsnext_fs --example fs_watcher_audit

if [ "$WATCH_ONLY" = 1 ]; then
  {
    echo ""
    echo "======== FS_AUDIT_WATCH_ONLY manual session — use your editor on:"
    echo "    WATCH_ROOT=$AUDIT_ROOT"
    echo "    OUTSIDE_STASH=$AUDIT_OUTSIDE"
    echo "======== stderr is also appended to: $LOG"
    echo ""
  } | tee -a "$LOG"
  exec cargo run -p bsnext_fs --example fs_watcher_audit "$AUDIT_ROOT" 2> >(tee -a "$LOG" >&2)
fi

cargo run -q -p bsnext_fs --example fs_watcher_audit "$AUDIT_ROOT" >>"$LOG" 2>&1 &
PID=$!

for _ in $(seq 1 100); do
  if grep -q FS_AUDIT_READY "$LOG" 2>/dev/null; then
    break
  fi
  sleep 0.05
done

if ! grep -q FS_AUDIT_READY "$LOG" 2>/dev/null; then
  echo "timed out waiting for FS_AUDIT_READY; see $LOG" >&2
  kill "$PID" 2>/dev/null || true
  exit 1
fi

sleep 0.4

audit_step "content-changed"
printf '\nextra-bytes\n' >>"$AUDIT_ROOT/existing.txt"
sleep 0.6

audit_step "touched"
touch "$AUDIT_ROOT/existing.txt"
sleep 0.6

audit_step "file-added"
printf 'new\n' >"$AUDIT_ROOT/audit-new-file.txt"
sleep 0.6

audit_step "file-removed"
rm -f "$AUDIT_ROOT/audit-new-file.txt"
sleep 0.6

audit_step "folder-added"
mkdir "$AUDIT_ROOT/audit-new-dir"
sleep 0.6

audit_step "folder-removed"
rmdir "$AUDIT_ROOT/audit-new-dir"
sleep 0.6

audit_step "file-renamed-within-watch"
printf 'rename-me\n' >"$AUDIT_ROOT/rename-me.txt"
sleep 0.25
mv "$AUDIT_ROOT/rename-me.txt" "$AUDIT_ROOT/renamed-file.txt"
sleep 0.6

audit_step "folder-renamed-within-watch"
mkdir "$AUDIT_ROOT/old-name-dir"
printf 'nested\n' >"$AUDIT_ROOT/old-name-dir/inner.txt"
sleep 0.25
mv "$AUDIT_ROOT/old-name-dir" "$AUDIT_ROOT/new-name-dir"
sleep 0.6

audit_step "symlink-file-created"
ln -s existing.txt "$AUDIT_ROOT/sym-existing.txt"
sleep 0.6

audit_step "symlink-write-through"
printf '\nvia-symlink\n' >>"$AUDIT_ROOT/sym-existing.txt"
sleep 0.6

audit_step "symlink-target-outside-watch"
ln -s "$AUDIT_OUTSIDE/outside-target.txt" "$AUDIT_ROOT/sym-outside.txt"
sleep 0.6

audit_step "symlink-outside-target-modified"
printf '\noutside-patch\n' >>"$AUDIT_OUTSIDE/outside-target.txt"
sleep 0.6

audit_step "chmod-executable-bit"
printf '#!/bin/sh\necho noop\n' >"$AUDIT_ROOT/meta-chmod.sh"
sleep 0.2
chmod +x "$AUDIT_ROOT/meta-chmod.sh" 2>>"$LOG" || echo "AUDIT_NOTE chmod +x failed (ignored)" >>"$LOG"
sleep 0.4
chmod -x "$AUDIT_ROOT/meta-chmod.sh" 2>>"$LOG" || echo "AUDIT_NOTE chmod -x failed (ignored)" >>"$LOG"
sleep 0.6

audit_step "touch-access-time-only"
touch -a "$AUDIT_ROOT/existing.txt" 2>>"$LOG" || {
  echo "AUDIT_NOTE touch -a not supported; skipped" >>"$LOG"
}
sleep 0.6

audit_step "move-file-into-watch-root"
mv "$AUDIT_OUTSIDE/brought-in.txt" "$AUDIT_ROOT/moved-in-from-outside.txt"
sleep 0.6

audit_step "move-file-out-of-watch-root"
printf 'leaving\n' >"$AUDIT_ROOT/moved-out-source.txt"
sleep 0.2
mv "$AUDIT_ROOT/moved-out-source.txt" "$AUDIT_OUTSIDE/moved-out-dest.txt"
sleep 0.6

{
  echo ""
  echo "======== fs-notify-audit finished $(date -u +%Y-%m-%dT%H:%M:%SZ) ========"
} >>"$LOG"

kill "$PID" 2>/dev/null || true
wait "$PID" 2>/dev/null || true

echo "Wrote $LOG"
