use bonsai_contracts::adapter::{AdapterProtocolMachine, Peer, ProtocolState};
use bonsai_contracts::bonsai::adapter::v1::{
    AdapterFrame, CausalAction, CausalObservation, CausalTransition, adapter_frame,
};
use prost::Message;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedFrame {
    peer: String,
    hex: String,
}

#[test]
fn live_python_causal_transcript_obeys_existing_rust_protocol() {
    let frames: Vec<RecordedFrame> = serde_json::from_str(include_str!(
        "../../../fixtures/scenario/causal-v1/wire-transcript.json"
    ))
    .expect("live transcript");
    let mut machine = AdapterProtocolMachine::default();
    let mut input_type = String::new();
    let mut transitions = 0;
    for item in frames {
        let bytes = decode_hex(&item.hex);
        let frame = AdapterFrame::decode(bytes.as_slice()).expect("Python protobuf frame");
        assert_eq!(
            frame.encode_to_vec(),
            bytes,
            "canonical cross-language encoding"
        );
        let peer = match item.peer.as_str() {
            "supervisor" => Peer::Supervisor,
            "adapter" => Peer::Adapter,
            _ => panic!("unknown peer"),
        };
        machine
            .apply(peer, &frame)
            .expect("existing protocol accepts live exchange");
        match frame.message.expect("message") {
            adapter_frame::Message::Step(request) => {
                input_type = request.input_type;
                if input_type == "bonsai.environment.action/v1" {
                    let action =
                        CausalAction::decode(request.input.as_slice()).expect("action payload");
                    assert_eq!(action.step, transitions);
                    assert!(action.action < 3);
                } else {
                    assert_eq!(input_type, "bonsai.environment.observe/v1");
                    assert!(request.input.is_empty());
                }
            }
            adapter_frame::Message::StepResult(result) => {
                if input_type == "bonsai.environment.observe/v1" {
                    let observation =
                        CausalObservation::decode(result.action.as_slice()).expect("observation");
                    assert_eq!(observation.step, 0);
                    assert_eq!(observation.observation.len(), 4);
                    assert_eq!(observation.allowed_actions, [0, 1, 2]);
                } else {
                    let transition =
                        CausalTransition::decode(result.action.as_slice()).expect("transition");
                    assert_eq!(transition.step, transitions);
                    assert_eq!(
                        transition.next.as_ref().expect("next observation").step,
                        transitions + 1
                    );
                    assert!(!(transition.terminated && transition.truncated));
                    assert!((0..=1).contains(&transition.reward));
                    transitions += 1;
                }
            }
            _ => {}
        }
    }
    assert_eq!(transitions, 8);
    assert_eq!(machine.state(), ProtocolState::Stopped);
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16).expect("hex byte")
        })
        .collect()
}
