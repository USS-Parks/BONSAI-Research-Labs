from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Operation(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    OPERATION_UNSPECIFIED: _ClassVar[Operation]
    OPERATION_CONFIGURE: _ClassVar[Operation]
    OPERATION_RESET: _ClassVar[Operation]
    OPERATION_FEEDBACK: _ClassVar[Operation]
OPERATION_UNSPECIFIED: Operation
OPERATION_CONFIGURE: Operation
OPERATION_RESET: Operation
OPERATION_FEEDBACK: Operation

class AdapterFrame(_message.Message):
    __slots__ = ("sequence", "protocol_epoch", "protocol_minor", "capability_fingerprint_sha256", "start", "handshake", "configure", "reset", "step", "step_result", "work", "work_result", "feedback", "ack", "event", "stop", "stopped", "error")
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    PROTOCOL_EPOCH_FIELD_NUMBER: _ClassVar[int]
    PROTOCOL_MINOR_FIELD_NUMBER: _ClassVar[int]
    CAPABILITY_FINGERPRINT_SHA256_FIELD_NUMBER: _ClassVar[int]
    START_FIELD_NUMBER: _ClassVar[int]
    HANDSHAKE_FIELD_NUMBER: _ClassVar[int]
    CONFIGURE_FIELD_NUMBER: _ClassVar[int]
    RESET_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    STEP_RESULT_FIELD_NUMBER: _ClassVar[int]
    WORK_FIELD_NUMBER: _ClassVar[int]
    WORK_RESULT_FIELD_NUMBER: _ClassVar[int]
    FEEDBACK_FIELD_NUMBER: _ClassVar[int]
    ACK_FIELD_NUMBER: _ClassVar[int]
    EVENT_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STOPPED_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    sequence: int
    protocol_epoch: int
    protocol_minor: int
    capability_fingerprint_sha256: bytes
    start: Start
    handshake: Handshake
    configure: Configure
    reset: Reset
    step: Step
    step_result: StepResult
    work: Work
    work_result: WorkResult
    feedback: Feedback
    ack: Ack
    event: Event
    stop: Stop
    stopped: Stopped
    error: ProtocolError
    def __init__(self, sequence: _Optional[int] = ..., protocol_epoch: _Optional[int] = ..., protocol_minor: _Optional[int] = ..., capability_fingerprint_sha256: _Optional[bytes] = ..., start: _Optional[_Union[Start, _Mapping]] = ..., handshake: _Optional[_Union[Handshake, _Mapping]] = ..., configure: _Optional[_Union[Configure, _Mapping]] = ..., reset: _Optional[_Union[Reset, _Mapping]] = ..., step: _Optional[_Union[Step, _Mapping]] = ..., step_result: _Optional[_Union[StepResult, _Mapping]] = ..., work: _Optional[_Union[Work, _Mapping]] = ..., work_result: _Optional[_Union[WorkResult, _Mapping]] = ..., feedback: _Optional[_Union[Feedback, _Mapping]] = ..., ack: _Optional[_Union[Ack, _Mapping]] = ..., event: _Optional[_Union[Event, _Mapping]] = ..., stop: _Optional[_Union[Stop, _Mapping]] = ..., stopped: _Optional[_Union[Stopped, _Mapping]] = ..., error: _Optional[_Union[ProtocolError, _Mapping]] = ...) -> None: ...

class VersionRange(_message.Message):
    __slots__ = ("minimum_epoch", "minimum_minor", "maximum_epoch", "maximum_minor")
    MINIMUM_EPOCH_FIELD_NUMBER: _ClassVar[int]
    MINIMUM_MINOR_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_EPOCH_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_MINOR_FIELD_NUMBER: _ClassVar[int]
    minimum_epoch: int
    minimum_minor: int
    maximum_epoch: int
    maximum_minor: int
    def __init__(self, minimum_epoch: _Optional[int] = ..., minimum_minor: _Optional[int] = ..., maximum_epoch: _Optional[int] = ..., maximum_minor: _Optional[int] = ...) -> None: ...

class Start(_message.Message):
    __slots__ = ("run_id", "accepted_versions", "deterministic_seed", "deadline_monotonic_ns")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_VERSIONS_FIELD_NUMBER: _ClassVar[int]
    DETERMINISTIC_SEED_FIELD_NUMBER: _ClassVar[int]
    DEADLINE_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    run_id: bytes
    accepted_versions: VersionRange
    deterministic_seed: int
    deadline_monotonic_ns: int
    def __init__(self, run_id: _Optional[bytes] = ..., accepted_versions: _Optional[_Union[VersionRange, _Mapping]] = ..., deterministic_seed: _Optional[int] = ..., deadline_monotonic_ns: _Optional[int] = ...) -> None: ...

class CapabilityDeclaration(_message.Message):
    __slots__ = ("reset", "work", "feedback", "asynchronous_events", "accepted_input_types", "emitted_event_types", "retains_transitions", "offline_updates", "observer_data_access", "privileged_state_access", "filesystem_read", "filesystem_write", "network_access")
    RESET_FIELD_NUMBER: _ClassVar[int]
    WORK_FIELD_NUMBER: _ClassVar[int]
    FEEDBACK_FIELD_NUMBER: _ClassVar[int]
    ASYNCHRONOUS_EVENTS_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_INPUT_TYPES_FIELD_NUMBER: _ClassVar[int]
    EMITTED_EVENT_TYPES_FIELD_NUMBER: _ClassVar[int]
    RETAINS_TRANSITIONS_FIELD_NUMBER: _ClassVar[int]
    OFFLINE_UPDATES_FIELD_NUMBER: _ClassVar[int]
    OBSERVER_DATA_ACCESS_FIELD_NUMBER: _ClassVar[int]
    PRIVILEGED_STATE_ACCESS_FIELD_NUMBER: _ClassVar[int]
    FILESYSTEM_READ_FIELD_NUMBER: _ClassVar[int]
    FILESYSTEM_WRITE_FIELD_NUMBER: _ClassVar[int]
    NETWORK_ACCESS_FIELD_NUMBER: _ClassVar[int]
    reset: bool
    work: bool
    feedback: bool
    asynchronous_events: bool
    accepted_input_types: _containers.RepeatedScalarFieldContainer[str]
    emitted_event_types: _containers.RepeatedScalarFieldContainer[str]
    retains_transitions: bool
    offline_updates: bool
    observer_data_access: bool
    privileged_state_access: bool
    filesystem_read: bool
    filesystem_write: bool
    network_access: bool
    def __init__(self, reset: bool = ..., work: bool = ..., feedback: bool = ..., asynchronous_events: bool = ..., accepted_input_types: _Optional[_Iterable[str]] = ..., emitted_event_types: _Optional[_Iterable[str]] = ..., retains_transitions: bool = ..., offline_updates: bool = ..., observer_data_access: bool = ..., privileged_state_access: bool = ..., filesystem_read: bool = ..., filesystem_write: bool = ..., network_access: bool = ...) -> None: ...

class Handshake(_message.Message):
    __slots__ = ("selected_epoch", "selected_minor", "capabilities")
    SELECTED_EPOCH_FIELD_NUMBER: _ClassVar[int]
    SELECTED_MINOR_FIELD_NUMBER: _ClassVar[int]
    CAPABILITIES_FIELD_NUMBER: _ClassVar[int]
    selected_epoch: int
    selected_minor: int
    capabilities: CapabilityDeclaration
    def __init__(self, selected_epoch: _Optional[int] = ..., selected_minor: _Optional[int] = ..., capabilities: _Optional[_Union[CapabilityDeclaration, _Mapping]] = ...) -> None: ...

class Configure(_message.Message):
    __slots__ = ("configuration_sha256", "accepted_capability_fingerprint_sha256", "deadline_monotonic_ns")
    CONFIGURATION_SHA256_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_CAPABILITY_FINGERPRINT_SHA256_FIELD_NUMBER: _ClassVar[int]
    DEADLINE_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    configuration_sha256: bytes
    accepted_capability_fingerprint_sha256: bytes
    deadline_monotonic_ns: int
    def __init__(self, configuration_sha256: _Optional[bytes] = ..., accepted_capability_fingerprint_sha256: _Optional[bytes] = ..., deadline_monotonic_ns: _Optional[int] = ...) -> None: ...

class Reset(_message.Message):
    __slots__ = ("episode_id", "deterministic_seed", "deadline_monotonic_ns")
    EPISODE_ID_FIELD_NUMBER: _ClassVar[int]
    DETERMINISTIC_SEED_FIELD_NUMBER: _ClassVar[int]
    DEADLINE_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    episode_id: bytes
    deterministic_seed: int
    deadline_monotonic_ns: int
    def __init__(self, episode_id: _Optional[bytes] = ..., deterministic_seed: _Optional[int] = ..., deadline_monotonic_ns: _Optional[int] = ...) -> None: ...

class Step(_message.Message):
    __slots__ = ("step_index", "input_type", "input", "input_sha256", "deadline_monotonic_ns")
    STEP_INDEX_FIELD_NUMBER: _ClassVar[int]
    INPUT_TYPE_FIELD_NUMBER: _ClassVar[int]
    INPUT_FIELD_NUMBER: _ClassVar[int]
    INPUT_SHA256_FIELD_NUMBER: _ClassVar[int]
    DEADLINE_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    step_index: int
    input_type: str
    input: bytes
    input_sha256: bytes
    deadline_monotonic_ns: int
    def __init__(self, step_index: _Optional[int] = ..., input_type: _Optional[str] = ..., input: _Optional[bytes] = ..., input_sha256: _Optional[bytes] = ..., deadline_monotonic_ns: _Optional[int] = ...) -> None: ...

class StepResult(_message.Message):
    __slots__ = ("step_index", "action", "action_sha256")
    STEP_INDEX_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    ACTION_SHA256_FIELD_NUMBER: _ClassVar[int]
    step_index: int
    action: bytes
    action_sha256: bytes
    def __init__(self, step_index: _Optional[int] = ..., action: _Optional[bytes] = ..., action_sha256: _Optional[bytes] = ...) -> None: ...

class Work(_message.Message):
    __slots__ = ("work_item_id", "work_class", "payload", "payload_sha256", "deadline_monotonic_ns")
    WORK_ITEM_ID_FIELD_NUMBER: _ClassVar[int]
    WORK_CLASS_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_SHA256_FIELD_NUMBER: _ClassVar[int]
    DEADLINE_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    work_item_id: bytes
    work_class: str
    payload: bytes
    payload_sha256: bytes
    deadline_monotonic_ns: int
    def __init__(self, work_item_id: _Optional[bytes] = ..., work_class: _Optional[str] = ..., payload: _Optional[bytes] = ..., payload_sha256: _Optional[bytes] = ..., deadline_monotonic_ns: _Optional[int] = ...) -> None: ...

class WorkResult(_message.Message):
    __slots__ = ("work_item_id", "outcome", "result", "result_sha256")
    WORK_ITEM_ID_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    RESULT_SHA256_FIELD_NUMBER: _ClassVar[int]
    work_item_id: bytes
    outcome: str
    result: bytes
    result_sha256: bytes
    def __init__(self, work_item_id: _Optional[bytes] = ..., outcome: _Optional[str] = ..., result: _Optional[bytes] = ..., result_sha256: _Optional[bytes] = ...) -> None: ...

class Feedback(_message.Message):
    __slots__ = ("feedback_id", "signal_type", "signal", "signal_sha256", "deadline_monotonic_ns")
    FEEDBACK_ID_FIELD_NUMBER: _ClassVar[int]
    SIGNAL_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIGNAL_FIELD_NUMBER: _ClassVar[int]
    SIGNAL_SHA256_FIELD_NUMBER: _ClassVar[int]
    DEADLINE_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    feedback_id: bytes
    signal_type: str
    signal: bytes
    signal_sha256: bytes
    deadline_monotonic_ns: int
    def __init__(self, feedback_id: _Optional[bytes] = ..., signal_type: _Optional[str] = ..., signal: _Optional[bytes] = ..., signal_sha256: _Optional[bytes] = ..., deadline_monotonic_ns: _Optional[int] = ...) -> None: ...

class Ack(_message.Message):
    __slots__ = ("operation",)
    OPERATION_FIELD_NUMBER: _ClassVar[int]
    operation: Operation
    def __init__(self, operation: _Optional[_Union[Operation, str]] = ...) -> None: ...

class Event(_message.Message):
    __slots__ = ("event_envelope",)
    EVENT_ENVELOPE_FIELD_NUMBER: _ClassVar[int]
    event_envelope: bytes
    def __init__(self, event_envelope: _Optional[bytes] = ...) -> None: ...

class Stop(_message.Message):
    __slots__ = ("reason_code", "deadline_monotonic_ns")
    REASON_CODE_FIELD_NUMBER: _ClassVar[int]
    DEADLINE_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    reason_code: str
    deadline_monotonic_ns: int
    def __init__(self, reason_code: _Optional[str] = ..., deadline_monotonic_ns: _Optional[int] = ...) -> None: ...

class Stopped(_message.Message):
    __slots__ = ("outcome_code",)
    OUTCOME_CODE_FIELD_NUMBER: _ClassVar[int]
    outcome_code: str
    def __init__(self, outcome_code: _Optional[str] = ...) -> None: ...

class ProtocolError(_message.Message):
    __slots__ = ("reason_code", "bounded_detail")
    REASON_CODE_FIELD_NUMBER: _ClassVar[int]
    BOUNDED_DETAIL_FIELD_NUMBER: _ClassVar[int]
    reason_code: str
    bounded_detail: str
    def __init__(self, reason_code: _Optional[str] = ..., bounded_detail: _Optional[str] = ...) -> None: ...

class CausalAction(_message.Message):
    __slots__ = ("step", "action")
    STEP_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    step: int
    action: int
    def __init__(self, step: _Optional[int] = ..., action: _Optional[int] = ...) -> None: ...

class CausalObservation(_message.Message):
    __slots__ = ("stream_id", "step", "observation", "allowed_actions")
    STREAM_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    OBSERVATION_FIELD_NUMBER: _ClassVar[int]
    ALLOWED_ACTIONS_FIELD_NUMBER: _ClassVar[int]
    stream_id: str
    step: int
    observation: _containers.RepeatedScalarFieldContainer[int]
    allowed_actions: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, stream_id: _Optional[str] = ..., step: _Optional[int] = ..., observation: _Optional[_Iterable[int]] = ..., allowed_actions: _Optional[_Iterable[int]] = ...) -> None: ...

class CausalTransition(_message.Message):
    __slots__ = ("step", "action", "reward", "next", "terminated", "truncated")
    STEP_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    REWARD_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    TERMINATED_FIELD_NUMBER: _ClassVar[int]
    TRUNCATED_FIELD_NUMBER: _ClassVar[int]
    step: int
    action: int
    reward: int
    next: CausalObservation
    terminated: bool
    truncated: bool
    def __init__(self, step: _Optional[int] = ..., action: _Optional[int] = ..., reward: _Optional[int] = ..., next: _Optional[_Union[CausalObservation, _Mapping]] = ..., terminated: bool = ..., truncated: bool = ...) -> None: ...

class PrimitiveReward(_message.Message):
    __slots__ = ("step", "reward")
    STEP_FIELD_NUMBER: _ClassVar[int]
    REWARD_FIELD_NUMBER: _ClassVar[int]
    step: int
    reward: int
    def __init__(self, step: _Optional[int] = ..., reward: _Optional[int] = ...) -> None: ...

class PrimitiveAccounting(_message.Message):
    __slots__ = ("environment_steps", "updates", "parameter_touches", "work_items", "replay_items_retained")
    ENVIRONMENT_STEPS_FIELD_NUMBER: _ClassVar[int]
    UPDATES_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_TOUCHES_FIELD_NUMBER: _ClassVar[int]
    WORK_ITEMS_FIELD_NUMBER: _ClassVar[int]
    REPLAY_ITEMS_RETAINED_FIELD_NUMBER: _ClassVar[int]
    environment_steps: int
    updates: int
    parameter_touches: int
    work_items: int
    replay_items_retained: int
    def __init__(self, environment_steps: _Optional[int] = ..., updates: _Optional[int] = ..., parameter_touches: _Optional[int] = ..., work_items: _Optional[int] = ..., replay_items_retained: _Optional[int] = ...) -> None: ...
