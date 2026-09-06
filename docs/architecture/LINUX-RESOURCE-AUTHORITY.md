# Scoped Linux resource authority

BX-03 extends the existing platform backend and governor preflight. Discovery alone remains fail-closed: directory creation does not authorize hard-control claims. LinuxAuthority creates a fresh leaf, applies controls, and requires exact readbacks. The governor's preflight_linux_authority accepts this active object and re-reads its controls and counters before admission. Removal or external changes invalidate admission. Unsupported I/O limits remain unsupported.

## Delegation and scope

The operator must supply an already delegated cgroup v2 subtree. Its root must be the current process's group or immediate parent, below /sys/fs/cgroup, with no resident processes. A separate supervisor subgroup satisfies that constraint. Construction opens the delegated control file to check permission, requires cpu/memory/pids availability, and enables only those controllers inside that subtree. It never changes the global cgroup root or adopts an existing leaf name. No privileged setup is attempted.

The acceptance command uses the existing WSL2 Ubuntu systemd user manager with Delegate=yes and DelegateSubgroup=supervisor in a transient service. RuntimeMaxSec=50 bounds the entire probe. WSL2 evidence is virtualized Linux evidence; physical Linux remains a separate gate. The capability object uses unknown host class and does not certify physical acceptance.

## Control semantics

The implementation follows the [kernel cgroup v2 interface](https://docs.kernel.org/admin-guide/cgroup-v2.html): cpu.max constrains fair-scheduler bandwidth per period, not cumulative CPU time. memory.max bounds charged memory; swap is disabled for this scope and memory.oom.group requests a whole-workload OOM kill. pids.max limits kernel tasks, including threads. Counter snapshots retain usage, throttling, memory peaks/violations, task-limit violations, live membership, and populated state.

A small shell gate is attached and verified before exec of the workload. Subsequently created descendants inherit its membership. Previously created children are never claimed to migrate automatically. The gate's small startup footprint precedes attachment; executable initialization and workload allocations follow attachment. This is not hostile-code isolation: a cooperative adapter must not alter its scheduler or migrate out of its assigned scope.

Cleanup uses cgroup.kill, requires no populated descendants and zero current tasks, then removes only the newly owned leaf. It is distinct from process-group cancellation. The caller first invokes terminate and reaps direct-child handles, because kernel task accounting may retain zombies until reaping. Cleanup then verifies zero membership. The caller also owns child-pipe draining; the live gate also requires every recorded PID to disappear. Drop is bounded best effort; callers must use and check explicit cleanup for acceptance evidence. A kernel I/O failure is an error, never a successful cleanup claim.

## Live acceptance

Build the linux_authority_probe example for bonsai-governor, then run evidence/verification/bx-03/live-gate.sh from the repository root. Missing delegation fails the gate. The probe requires a real denied write-open outside delegation, verifies three-process membership, measures throttling while only a descendant burns CPU, crosses a 64 MiB memory limit using a bounded 128 MiB allocation, and observes fork denial at four tasks. It checks governor rejection for unsupported I/O and altered controller state, followed by empty cleanup and absent PIDs for each case. Each workload self-expires; an outer watchdog fails the probe after 40 seconds.

The CPU test uses 10,000 microseconds per 100,000-microsecond period. Its two-second sample must show throttled periods and positive CPU use no greater than the elapsed bandwidth allowance plus 70 ms for boundary/accounting tolerance. This is a bandwidth test; no cumulative-budget termination claim follows. Memory acceptance requires actual max events and OOM kills; a settings-only record cannot pass.
