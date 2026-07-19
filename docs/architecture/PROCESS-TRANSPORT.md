# Bounded process transport v1

Status: BR-02 runtime authority for schema epoch 1.

The BONSAI supervisor launches an adapter as a child process with piped standard input, standard output, and standard error. Standard input and output carry only the BR-01 `AdapterFrame` protocol. Standard error is drained independently into a bounded diagnostic capture and can never be parsed as protocol data. The implementation does not open a socket, allocate a network identity, or claim hostile-code sandboxing.

## Framing and bounds

Each message is one unsigned 32-bit little-endian byte length followed by the exact deterministic Protobuf bytes. Empty frames, a zero/excessive configured maximum, a declared length above the configured maximum, a partial four-byte header, and a partial payload are distinct failures. The hard implementation ceiling is 16 MiB, and length is checked before payload allocation.

Rust and Python use the same framing rules and stable error codes. The Python runtime is standard-library-only and flushes after every frame. The cross-language process fixture proves Rust-to-Python and Python-to-Rust bytes through the real inherited pipes on every hosted CI operating system.

## Backpressure, deadlines, and containment

The stdout reader owns a fixed-capacity channel. It never grows the pending-frame queue. If the adapter fills that queue before the supervisor consumes it, the reader closes the protocol stream and records `TRANSPORT_BACKPRESSURE_EXCEEDED`; closing the pipe bounds the child sender through the OS pipe. A caller deadline uses a bounded receive timeout and records `TRANSPORT_READ_TIMEOUT` before killing and reaping the child.

Malformed, partial, excessive, stalled, and flood traffic all cause fail-closed containment. Each failure record contains a stable code plus no more than 256 characters of diagnostic detail. I/O details record only the portable error kind. The caller receives the same typed failure that was recorded.

## stderr and shutdown

Standard error is continuously drained on a separate thread. The capture records total bytes, retains only the configured prefix, and marks truncation when later bytes are discarded. Draining continues after the retention bound so a verbose adapter cannot deadlock on a full stderr pipe.

Clean shutdown closes protocol input, waits only for the caller-supplied duration, and joins the stdout/stderr drain threads. A child that does not exit is killed and reaped with `TRANSPORT_SHUTDOWN_TIMEOUT`. Dropping a live transport also closes input, kills, reaps, disconnects the bounded queue, and joins both drain threads.

## Boundary with later prompts

BR-02 owns process creation, bounded framing, deadlines, backpressure, diagnostic separation, and process cleanup. BR-03 validates event meaning before append. BR-05 owns run lifecycle and recovery. BR-06 constrains arguments, environment, handles, working directories, and filesystem visibility. Platform-specific descendant process controls and hard resource enforcement remain in BM/BQ. None of those later claims is implied by this transport.

The committed matrix at `fixtures/process-transport/v1/expected-outcomes.json` names the exact good, partial, oversized, stalled, and flood outcomes.
