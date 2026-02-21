# Playwright Test Failures to Fix

Failures from the `defaults` branch PR: https://github.com/BrowserSync/bslive/actions/runs/22034914640/job/63666068435

All 9 failures are caused by two issues introduced during experiments on the `defaults` branch.

---

## Issue 1: `BsLiveRules` event not handled by test utils (causes test 1)

The server now emits a `BsLiveRules` event (`"bslive rules violated"`) on startup for watch-based configs. The test utility code doesn't recognise this event kind and logs `"cannot continue - probably an unsupported type"`, causing `watch-graceful` to timeout waiting for expected messages.

**Failing test:**
- `graceful_exit.spec.ts:15` — "graceful exit on timeout" (Error: 1000 timeout expired)

**Files:**
- [tests/utils.ts](tests/utils.ts) — line ~117-121, the `loose.safeParse` fallback logs but doesn't handle `BsLiveRules`
- [crates/bsnext_input/src/lib.rs](crates/bsnext_input/src/lib.rs) — `BsLiveRulesError` definition (~line 294)
- [crates/bsnext_system/src/start/start_kind/start_from_inputs.rs](crates/bsnext_system/src/start/start_kind/start_from_inputs.rs) — where the event is emitted

---

## Issue 2: Output format changed — `prefix` and `task_id` fields (causes tests 2-9)

The `OutputLine` payload shape changed. Previously each output line had `"prefix": "[run]"` and no `task_id`. Now:
- `prefix` is a random short ID like `"[M3CpiT]"` instead of the fixed `"[run]"`
- A new `task_id` field is present in the payload

This breaks all assertions that check the exact shape of `OutputLine` objects.

**Failing tests:**
- `watcher.before.spec.ts:15` — "running before" (expected `"[run]"`, got random prefix + extra `task_id` field)
- `watcher.cancellation.spec.ts:15` — "cancellation of sibling tasks" (can't find `"starting succeeding task"` line)
- `watcher.fail.spec.ts:15` — "ignoring failures in a task sequence" (extra `task_id` field)
- `watcher.patterns.spec.ts:15` — "patterns in globs" (extra `task_id` field)
- `watcher.spec.ts:95` — "custom output for index.html" (random prefix + `task_id`)
- `watcher.spec.ts:115` — "custom output for 01.txt" (extra `task_id`)
- `watcher.spec.ts:135` — "custom output for 02.txt" (extra `task_id`)
- `watcher.spec.ts:155` — "without prefix" (extra `task_id`)

**Files:**
- [tests/watcher.before.spec.ts](tests/watcher.before.spec.ts)
- [tests/watcher.cancellation.spec.ts](tests/watcher.cancellation.spec.ts)
- [tests/watcher.fail.spec.ts](tests/watcher.fail.spec.ts)
- [tests/watcher.patterns.spec.ts](tests/watcher.patterns.spec.ts)
- [tests/watcher.spec.ts](tests/watcher.spec.ts)
- [tests/graceful_exit.spec.ts](tests/graceful_exit.spec.ts)

---

## Summary

Both issues stem from changes to task execution on the `defaults` branch. To fix: either revert the output format changes (prefix/task_id) or update the test expectations and utils to handle the new shape.
