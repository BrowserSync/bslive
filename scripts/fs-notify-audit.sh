#!/usr/bin/env bash
# Reproducible filesystem notify audit for bsnext_fs (notify-rs + platform_accepts).
#
# Cross-platform env (same on macOS, Linux, Git Bash on Windows):
#   RUST_LOG          tracing filter (default: bsnext_fs=trace)
#                     Narrow: RUST_LOG=bsnext_fs::platform_accepts=trace,bsnext_fs::watcher=trace
#   FS_AUDIT_ROOT     directory to create and watch (default: <repo>/target/fs-notify-audit-root)
#   FS_AUDIT_LOG      output log path (default: <repo>/audit/fs-notify/logs/raw-<kernel>.log)
#
# Usage:
#   ./scripts/fs-notify-audit.sh
#   FS_AUDIT_LOG=/tmp/audit.log RUST_LOG=bsnext_fs=trace ./scripts/fs-notify-audit.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

KERNEL="$(uname -s | tr '[:upper:]' '[:lower:]')"
AUDIT_ROOT="${FS_AUDIT_ROOT:-$REPO_ROOT/target/fs-notify-audit-root}"
LOG="${FS_AUDIT_LOG:-$REPO_ROOT/audit/fs-notify/logs/raw-${KERNEL}.log}"
export RUST_LOG="${RUST_LOG:-bsnext_fs=trace}"

mkdir -p "$(dirname "$LOG")"
rm -f "$LOG"

{
  echo "======== fs-notify-audit started $(date -u +%Y-%m-%dT%H:%M:%SZ) ========"
  echo "uname: $(uname -a)"
  echo "RUST_LOG=$RUST_LOG"
  echo "FS_AUDIT_ROOT=$AUDIT_ROOT"
  echo "FS_AUDIT_LOG=$LOG"
  echo "=============================================================="
} >>"$LOG"

rm -rf "$AUDIT_ROOT"
mkdir -p "$AUDIT_ROOT"
printf '%s\n' 'seed line' >"$AUDIT_ROOT/existing.txt"

cargo build -q -p bsnext_fs --example fs_watcher_audit

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

audit_step() {
  {
    echo ""
    echo "======== AUDIT_STEP $1 $(date -u +%Y-%m-%dT%H:%M:%SZ) ========"
  } >>"$LOG"
}

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

{
  echo ""
  echo "======== fs-notify-audit finished $(date -u +%Y-%m-%dT%H:%M:%SZ) ========"
} >>"$LOG"

kill "$PID" 2>/dev/null || true
wait "$PID" 2>/dev/null || true

echo "Wrote $LOG"
