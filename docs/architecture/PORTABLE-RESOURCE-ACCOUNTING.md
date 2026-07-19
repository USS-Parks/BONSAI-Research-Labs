# Portable resource accounting

BM-03 combines three independently scoped accounting surfaces:

- a live process-tree snapshot rooted at the supervised agent process;
- byte-exact, non-symlink-following traversal of agent-owned and observer-owned storage roots;
- checked external counters for environment steps, updates, touches, work items, model calls, and planning backups.

The process collector uses refreshed public cross-platform process information and recursively includes descendants by parent PID. It sums accumulated CPU milliseconds into canonical nanoseconds, resident memory bytes, platform-qualified virtual-memory bytes, and cumulative process I/O bytes. RSS and virtual-memory meanings are not treated as cross-OS equivalents. Windows process I/O includes all process I/O; other supported systems expose process disk I/O subject to caching. Those distinctions remain in every snapshot.

Storage traversal never follows symbolic links and fails closed on entry, depth, I/O, or arithmetic bounds. Agent and observer storage are emitted as different root classes. Operation counters are exact external charges and reject overflow rather than saturating.

The hosted gate uses a real parent process that launches a real child and requires both PIDs in one scoped aggregate. Separate live filesystem fixtures prove exact known byte totals. Platform enforcement, Job Objects, cgroups, energy, and claims remain later prompts.
