use super::context::Context;
use super::snapshot::Snapshot;
use super::trace::{Trace, decode, unhex};
use super::{Result, ensure, number};
use bonsai_contracts::bonsai::adapter::v1::{self as wire, adapter_frame::Message as Kind};
use prost::Message;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(super) struct Outcome {
    pub(super) events: u64,
    pub(super) steps: u64,
    pub(super) episodes: u64,
    pub(super) reward: i64,
    pub(super) work: u64,
}

pub(super) fn reconstruct(snapshot: &Snapshot, context: &Context) -> Result<Outcome> {
    let mut trace = Trace::load(snapshot, context.steps)?;
    ensure(
        trace.last_time < number(&context.manifest["resource_profile"], "wall_time_limit_ns")?,
        "RUN_TRACE_WALL_VIOLATION",
    )?;
    let controls = trace.take("run.resource", Some("controls_before_launch"))?;
    super::resources::controls(snapshot, context, &controls)?;
    let launch = trace.take("run.status", Some("agent_launch_policy"))?;
    super::reference::launch(&launch["audit"], &context.manifest)?;
    initialize(&mut trace, snapshot, context, true)?;
    initialize(&mut trace, snapshot, context, false)?;
    let mut outcome = Outcome {
        events: trace.count,
        steps: context.steps,
        episodes: 0,
        reward: 0,
        work: context.steps * context.per_step_work(),
    };
    let mut observation = None;
    let mut prior_cpu = 0;
    let mut prior_touches = 0_u64;
    for total in 0..context.steps {
        if observation.is_none() {
            observation = Some(reset(&mut trace, context, outcome.episodes)?);
        }
        let observed = observation.take().ok_or("RUN_OBSERVATION_MISSING")?;
        for (class, amount) in context.tariffs() {
            let work = trace.take("run.work", Some("admission"))?;
            super::resources::work(context, &work, total, class, amount)?;
        }
        grant(&mut trace, context, total)?;
        let transition = step(&mut trace, context, &observed, total, outcome.episodes)?;
        let accounting = feedback(&mut trace, context, total, &transition)?;
        let touches = accounting
            .parameter_touches
            .checked_sub(prior_touches)
            .ok_or("RUN_ACCOUNTING_REGRESSION")?;
        ensure(
            context
                .accounting
                .maximum_parameter_touches_per_update()
                .is_none_or(|cap| touches <= cap),
            "RUN_PARAMETER_TOUCH_BOUND_EXCEEDED",
        )?;
        prior_touches = accounting.parameter_touches;
        for (class, amount) in context.tariffs() {
            let work = trace.take("run.work", Some("measured_charge"))?;
            super::resources::work(context, &work, total, class, amount)?;
            ensure(
                unhex(super::text(&work, "accounting_hex")?)? == accounting.encode_to_vec(),
                "RUN_ACCOUNTING_LINK_MISMATCH",
            )?;
        }
        let usage = trace.take("run.resource", Some("step"))?;
        prior_cpu = super::resources::step(context, &usage, total, prior_cpu)?;
        outcome.reward = outcome
            .reward
            .checked_add(transition.reward)
            .ok_or("RUN_REWARD_OVERFLOW")?;
        let next = transition.next.ok_or("RUN_NEXT_OBSERVATION_MISSING")?;
        if transition.terminated || transition.truncated {
            ensure(
                next.allowed_actions.is_empty(),
                "RUN_TERMINAL_ACTIONS_INVALID",
            )?;
            outcome.episodes += 1;
        } else {
            observation = Some(next);
        }
    }
    finish(&mut trace, &outcome)?;
    trace.finish()?;
    metadata(snapshot, &outcome)?;
    Ok(outcome)
}

fn initialize(
    trace: &mut Trace,
    snapshot: &Snapshot,
    context: &Context,
    agent: bool,
) -> Result<()> {
    let (request, response) = trace.exchange(agent, 5_000_000_000)?;
    let (Kind::Start(start), Kind::Handshake(handshake)) = (request, response) else {
        return Err("RUN_INITIALIZATION_INVALID");
    };
    ensure(
        start.run_id == unhex(&snapshot.run_id.replace('-', ""))?
            && start.deterministic_seed == context.seed,
        "RUN_START_IDENTITY_INVALID",
    )?;
    let capabilities = handshake.capabilities.ok_or("RUN_CAPABILITIES_MISSING")?;
    if agent {
        let input_types = if context.accounting.is_transition_feedback() {
            vec![
                "bonsai.agent.observation/v1",
                "bonsai.agent.causal-transition/v1",
                "bonsai.agent.accounting/v1",
                "bonsai.agent.admission/v2",
            ]
        } else {
            vec![
                "bonsai.agent.observation/v1",
                "bonsai.agent.reward/v1",
                "bonsai.agent.accounting/v1",
            ]
        };
        ensure(
            capabilities
                == wire::CapabilityDeclaration {
                    reset: Some(true),
                    work: Some(true),
                    feedback: Some(true),
                    asynchronous_events: Some(false),
                    accepted_input_types: input_types.into_iter().map(str::to_owned).collect(),
                    emitted_event_types: vec![],
                    retains_transitions: Some(false),
                    offline_updates: Some(false),
                    observer_data_access: Some(false),
                    privileged_state_access: Some(false),
                    filesystem_read: Some(true),
                    filesystem_write: Some(false),
                    network_access: Some(false),
                    required_capabilities: Vec::new(),
                    optional_capabilities: Vec::new(),
                },
            "RUN_AGENT_CAPABILITIES_UNSUPPORTED",
        )?;
    }
    let (request, response) = trace.exchange(agent, 1_000_000_000)?;
    let (Kind::Configure(configure), Kind::Ack(ack)) = (request, response) else {
        return Err("RUN_CONFIGURATION_PROTOCOL_INVALID");
    };
    let (file, key) = if agent {
        ("agent-configuration.json", "adapter")
    } else {
        ("environment-configuration.json", "environment")
    };
    ensure(
        snapshot.json(file)? == context.manifest[key]["config"]
            && configure.configuration_sha256 == Sha256::digest(snapshot.bytes(file)?).as_slice()
            && configure.accepted_capability_fingerprint_sha256
                == Sha256::digest(capabilities.encode_to_vec()).as_slice()
            && ack.operation == wire::Operation::Configure as i32,
        "RUN_CONFIGURATION_HASH_MISMATCH",
    )
}

fn reset(trace: &mut Trace, context: &Context, episode: u64) -> Result<wire::CausalObservation> {
    for agent in [true, false] {
        let (request, response) = trace.exchange(agent, 1_000_000_000)?;
        let (Kind::Reset(reset), Kind::Ack(ack)) = (request, response) else {
            return Err("RUN_RESET_PROTOCOL_INVALID");
        };
        ensure(
            reset.episode_id == identity(episode)
                && reset.deterministic_seed == context.seed + episode
                && ack.operation == wire::Operation::Reset as i32,
            "RUN_EPISODE_IDENTITY_INVALID",
        )?;
    }
    let bytes = exchange_step(
        trace,
        false,
        0,
        "bonsai.environment.observe/v1",
        &[],
        1_000_000_000,
    )?;
    let observation: wire::CausalObservation = decode(&bytes)?;
    ensure(observation.step == 0, "RUN_EPISODE_STEP_INVALID")?;
    valid_observation(context, &observation, false)?;
    if context.manifest["environment"]["component_id"] == "bonsai-feature-chain" {
        let mut spec = context.manifest["environment"]["config"].clone();
        spec["seed"] = json!(context.seed + episode);
        let identity = serde_json::to_vec(&spec).map_err(|_| "RUN_CHAIN_IDENTITY_INVALID")?;
        ensure(
            observation.observation == vec![(context.seed + episode) % 4]
                && observation.stream_id == super::snapshot::digest(&identity),
            "RUN_CHAIN_RESET_MISMATCH",
        )?;
    }
    Ok(observation)
}

fn step(
    trace: &mut Trace,
    context: &Context,
    observed: &wire::CausalObservation,
    total: u64,
    episode: u64,
) -> Result<wire::CausalTransition> {
    valid_observation(context, observed, false)?;
    let index = observed.step;
    let observation_bytes = observed.encode_to_vec();
    let action_bytes = exchange_step(
        trace,
        true,
        index,
        "bonsai.agent.observation/v1",
        &observation_bytes,
        number(&context.manifest["resource_profile"], "action_deadline_ns")?,
    )?;
    let action: wire::CausalAction = decode(&action_bytes)?;
    ensure(
        action.step == index && observed.allowed_actions.contains(&action.action),
        "RUN_ACTION_INVALID",
    )?;
    let event = trace.take("run.action", None)?;
    ensure(
        event["total_step"] == total
            && event["episode"] == episode
            && event["step"] == index
            && event["action"] == action.action
            && unhex(super::text(&event, "observation_hex")?)? == observation_bytes,
        "RUN_ACTION_LINK_MISMATCH",
    )?;
    let bytes = exchange_step(
        trace,
        false,
        index + 1,
        "bonsai.environment.action/v1",
        &action_bytes,
        1_000_000_000,
    )?;
    let transition: wire::CausalTransition = decode(&bytes)?;
    let next = transition
        .next
        .as_ref()
        .ok_or("RUN_NEXT_OBSERVATION_MISSING")?;
    ensure(
        transition.step == index
            && transition.action == action.action
            && (context.manifest["environment"]["component_id"] == "bonsai-gymnasium-frozen-lake"
                || !(transition.terminated && transition.truncated))
            && next.step == index + 1
            && next.stream_id == observed.stream_id,
        "RUN_TRANSITION_INVALID",
    )?;
    valid_observation(context, next, transition.terminated || transition.truncated)?;
    if context.manifest["environment"]["component_id"] == "bonsai-feature-chain" {
        let state = observed
            .observation
            .first()
            .copied()
            .ok_or("RUN_CHAIN_STATE_MISSING")?;
        let after = (state + if action.action == 0 { 1 } else { 4 }) % 5;
        let attained = after == 4;
        let terminal =
            attained && context.manifest["environment"]["config"]["goal_terminates"] == true;
        let truncated = !terminal
            && index + 1 == number(&context.manifest["environment"]["config"], "horizon")?;
        ensure(
            state < 5
                && next.observation == vec![after]
                && transition.reward == if attained { 4 } else { -1 }
                && transition.terminated == terminal
                && transition.truncated == truncated,
            "RUN_CHAIN_DYNAMICS_MISMATCH",
        )?;
    }
    let event = trace.take("run.reward", None)?;
    ensure(
        event["total_step"] == total
            && event["episode"] == episode
            && event["step"] == index
            && event["reward"] == transition.reward
            && event["terminated"] == transition.terminated
            && event["truncated"] == transition.truncated
            && unhex(super::text(&event, "transition_hex")?)? == bytes,
        "RUN_REWARD_LINK_MISMATCH",
    )?;
    Ok(transition)
}

fn exchange_step(
    trace: &mut Trace,
    agent: bool,
    index: u64,
    input_type: &str,
    input: &[u8],
    timeout: u64,
) -> Result<Vec<u8>> {
    let (request, response) = trace.exchange(agent, timeout)?;
    let (Kind::Step(step), Kind::StepResult(result)) = (request, response) else {
        return Err("RUN_STEP_PROTOCOL_INVALID");
    };
    ensure(
        step.step_index == index
            && step.input_type == input_type
            && step.input == input
            && step.input_sha256 == Sha256::digest(input).as_slice()
            && result.step_index == index
            && result.action_sha256 == Sha256::digest(&result.action).as_slice(),
        "RUN_CAUSAL_INPUT_MISMATCH",
    )?;
    Ok(result.action)
}

fn valid_observation(
    context: &Context,
    value: &wire::CausalObservation,
    terminal: bool,
) -> Result<()> {
    let width = number(
        &context.manifest["environment"]["config"],
        "observation_width",
    )?;
    let actions = if terminal {
        vec![]
    } else {
        (0..u32::try_from(context.actions).map_err(|_| "RUN_ACTION_COUNT_INVALID")?).collect()
    };
    ensure(
        !value.stream_id.is_empty()
            && value.stream_id.len() <= 128
            && u64::try_from(value.observation.len()).map_err(|_| "RUN_SIZE_OVERFLOW")? == width
            && value.allowed_actions == actions,
        "RUN_PUBLIC_OBSERVATION_INVALID",
    )
}

fn grant(trace: &mut Trace, context: &Context, total: u64) -> Result<()> {
    let Some(expected) = context.accounting.admission_payload(total) else {
        return Ok(());
    };
    let timeout = number(&context.manifest["resource_profile"], "action_deadline_ns")?;
    let (request, response) = trace.exchange(true, timeout)?;
    let (Kind::Work(work), Kind::WorkResult(result)) = (request, response) else {
        return Err("RUN_ADMISSION_PROTOCOL_INVALID");
    };
    let payload = serde_json::to_vec(&expected).map_err(|_| "RUN_ADMISSION_INVALID")?;
    ensure(
        work.work_item_id == identity(total)
            && work.work_class == "bonsai.agent.admission/v2"
            && work.payload == payload
            && work.payload_sha256 == Sha256::digest(&payload).as_slice()
            && result.work_item_id == identity(total)
            && result.outcome == "ADMITTED"
            && result.result.is_empty()
            && result.result_sha256 == Sha256::digest([]).as_slice(),
        "RUN_ADMISSION_LINK_MISMATCH",
    )
}

fn feedback(
    trace: &mut Trace,
    context: &Context,
    total: u64,
    transition: &wire::CausalTransition,
) -> Result<wire::PrimitiveAccounting> {
    let timeout = number(&context.manifest["resource_profile"], "action_deadline_ns")?;
    let (request, response) = trace.exchange(true, timeout)?;
    let (Kind::Feedback(feedback), Kind::Ack(ack)) = (request, response) else {
        return Err("RUN_FEEDBACK_PROTOCOL_INVALID");
    };
    let (signal_type, signal) = if context.accounting.is_transition_feedback() {
        (
            "bonsai.agent.causal-transition/v1",
            transition.encode_to_vec(),
        )
    } else {
        (
            "bonsai.agent.reward/v1",
            wire::PrimitiveReward {
                step: transition.step,
                reward: transition.reward,
            }
            .encode_to_vec(),
        )
    };
    ensure(
        feedback.feedback_id == identity(total)
            && feedback.signal_type == signal_type
            && feedback.signal == signal
            && feedback.signal_sha256 == Sha256::digest(&signal).as_slice()
            && ack.operation == wire::Operation::Feedback as i32,
        "RUN_FEEDBACK_LINK_MISMATCH",
    )?;
    let (request, response) = trace.exchange(true, timeout)?;
    let (Kind::Work(work), Kind::WorkResult(result)) = (request, response) else {
        return Err("RUN_ACCOUNTING_PROTOCOL_INVALID");
    };
    ensure(
        work.work_item_id == identity(total)
            && work.work_class == "bonsai.agent.accounting/v1"
            && work.payload.is_empty()
            && work.payload_sha256 == Sha256::digest([]).as_slice()
            && result.work_item_id == identity(total)
            && result.outcome == "MEASURED"
            && result.result_sha256 == Sha256::digest(&result.result).as_slice(),
        "RUN_OFFLINE_WORK_UNSUPPORTED",
    )?;
    let measured: wire::PrimitiveAccounting = decode(&result.result)?;
    context
        .accounting
        .validate(&measured, total + 1, transition.action, transition.reward)?;
    ensure(
        measured.work_items == context.per_step_work() * (total + 1),
        "RUN_ACCOUNTING_INCONSISTENT",
    )?;
    Ok(measured)
}

fn finish(trace: &mut Trace, outcome: &Outcome) -> Result<()> {
    for agent in [true, false] {
        let (request, response) = trace.exchange(agent, 1_000_000_000)?;
        let (Kind::Stop(stop), Kind::Stopped(_)) = (request, response) else {
            return Err("RUN_STOP_PROTOCOL_INVALID");
        };
        ensure(
            stop.reason_code == "NORMAL_COMPLETION",
            "RUN_STOP_REASON_INVALID",
        )?;
    }
    for name in ["agent", "environment"] {
        let event = trace.take("run.status", Some("child_reaped"))?;
        ensure(
            event["adapter"] == name
                && event["exit_code"] == 0
                && event["transport_failures"] == json!([]),
            "RUN_CHILD_FAILURE",
        )?;
    }
    for name in ["agent", "environment"] {
        let event = trace.take("run.resource", Some("cleanup"))?;
        ensure(
            event["adapter"] == name
                && event["usage"]["populated"] == false
                && event["usage"]["member_pids"] == json!([])
                && event["usage"]["pids_current"] == 0,
            "RUN_CLEANUP_INCOMPLETE",
        )?;
        super::resources::healthy_usage(&event["usage"])?;
    }
    let event = trace.take("run.resource", Some("before_cleanup"))?;
    for name in ["agent", "environment"] {
        super::resources::healthy_usage(&event[name])?;
    }
    let event = trace.take("run.status", None)?;
    ensure(
        event["status"] == "COMPLETE"
            && event["status_scope"] == "execution_only"
            && event["details"] == details(outcome),
        "RUN_TERMINAL_SUMMARY_MISMATCH",
    )
}

fn metadata(snapshot: &Snapshot, outcome: &Outcome) -> Result<()> {
    let status = snapshot.json("run-status.json")?;
    ensure(
        status["events"] == outcome.events && status["details"] == details(outcome),
        "RUN_FINAL_SUMMARY_MISMATCH",
    )?;
    let metric = snapshot.json("metric-estimate.json")?;
    let hash = super::snapshot::digest(snapshot.bytes(super::trace::SEGMENT)?);
    ensure(
        metric["run_id"] == snapshot.run_id
            && metric["population"]["eligible_count"] == outcome.steps
            && metric["population"]["observed_count"] == outcome.steps
            && metric["population"]["selection_sha256"] == hash
            && metric["result"]["value"] == outcome.reward
            && metric["result"]["availability"] == "measured"
            && metric["missingness"]["missing_count"] == 0
            && metric["missingness"]["invalid_count"] == 0
            && metric["missingness"]["coverage_ratio"].as_f64() == Some(1.0)
            && metric["missingness"]["disposition"] == "complete",
        "RUN_METRIC_RECONSTRUCTION_MISMATCH",
    )?;
    metric_provenance(snapshot, outcome, &metric, &hash)
}

fn details(outcome: &Outcome) -> Value {
    json!({"environment_steps":outcome.steps,"finished_episodes":outcome.episodes,
        "reward_sum":outcome.reward,"work_items":outcome.work})
}

fn identity(sequence: u64) -> Vec<u8> {
    let mut bytes = vec![1; 16];
    bytes[..8].copy_from_slice(&sequence.to_le_bytes());
    bytes
}

fn metric_provenance(
    snapshot: &Snapshot,
    outcome: &Outcome,
    metric: &Value,
    hash: &str,
) -> Result<()> {
    let manifest = snapshot.json("experiment-manifest.json")?;
    let definition = snapshot.json("reward-metric-definition.json")?;
    ensure(
        definition
            == json!({"metric_id":"behavior.reward","version":"1.0",
        "aggregation":"sum","population":"accepted_reward_events"}),
        "RUN_METRIC_DEFINITION_UNSUPPORTED",
    )?;
    let canonical = serde_json::to_vec(&definition).map_err(|_| "RUN_JSON_INVALID")?;
    ensure(
        metric["metric_spec"]
            == json!({"metric_id":"behavior.reward","metric_version":"1.0",
        "canonical_sha256":super::snapshot::digest(&canonical)})
            && metric["population"]["population_id"] == "accepted_reward_events"
            && metric["result"]["kind"] == "scalar"
            && metric["result"]["unit"] == manifest["scenario"]["reward_unit_id"]
            && metric["window"] == json!({"basis":"step_count","start":0,"end":outcome.steps})
            && metric["inputs"]
                == json!([{"evidence_id":"event-segment-0","evidence_type":"event",
            "sha256":hash,"role":"accepted reward events"}]),
        "RUN_METRIC_PROVENANCE_INVALID",
    )?;
    let uncertainty = snapshot.json("metric-uncertainty.json")?;
    ensure(
        uncertainty["estimate_id"] == metric["estimate_id"]
            && metric["uncertainty_ids"] == json!([uncertainty["uncertainty_id"]])
            && uncertainty["result"]
                == json!({"kind":"unavailable","reason_code":"single_run_no_inference"})
            && uncertainty["sample_count"] == 1
            && uncertainty["confidence_level"].is_null(),
        "RUN_UNCERTAINTY_UNSUPPORTED",
    )?;
    let report = snapshot.json("reports/report.json")?;
    ensure(
        report["behavior"]["status"] == "COMPLETE"
            && report["behavior"]["events"] == outcome.events
            && report["behavior"]["details"] == details(outcome)
            && report["behavior"]["observed_rewards"] == outcome.steps
            && report["behavior"]["reward_sum"] == outcome.reward,
        "RUN_REPORT_SUMMARY_MISMATCH",
    )
}
