# Implement an adapter

Adapters are child processes behind the versioned length-delimited Protobuf protocol. They do not write canonical telemetry themselves.

Read:

- [Adapter protocol](../architecture/ADAPTER-PROTOCOL.md)
- [Process transport](../architecture/PROCESS-TRANSPORT.md)
- [Adapter conformance](../architecture/ADAPTER-CONFORMANCE.md)
- [Agent/observer isolation](../architecture/AGENT-OBSERVER-ISOLATION.md)

A conforming adapter:

- speaks only stdin/stdout protocol frames;
- writes work only under the granted `agent/work` tree;
- never receives observer paths in arguments, environment, or payloads;
- fails closed on oversized frames.

Conformance is a protocol verdict. It is not scientific quality, not a hostile-code sandbox, and not Track A certification by itself.
