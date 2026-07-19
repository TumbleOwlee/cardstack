# Storage — Edge Cases

Behavior that is ugly on purpose: documented deliberately rather than hidden.

---

**Stale lock file after a forced kill (`ST-R-013`).** The instance lock
(`ST-R-013`) is released via cleanup on normal exit, error exit, and the panic
hook. A `SIGKILL` (or an OS crash) bypasses all of that, so the lock file can
outlive its process. The next launch then refuses to start, reporting the lock
as held, even though nothing is actually running. There is no staleness
detection (e.g. checking whether the recorded PID is still alive) — recovery
is manual: remove the lock file from the config directory.
