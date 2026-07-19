# OaK Evidence and Traceability Register

Version: 0.1  
Date: 2026-07-18

## 1. Purpose

This register prevents BONSAI from presenting inference as published OaK specification. It separates:

- explicit statements in Richard Sutton's RLC 2025 OaK presentation;
- claims supported by public Oak Lab pages and primary papers;
- BONSAI interpretations that require validation through experimentation or future Oak publications.

## 2. Supplied source material

### Presentation screenshots

User-supplied Google Drive folder:

<https://drive.google.com/drive/folders/1achJJ38FCdjA-TkhS2Vt3wWyL4DRx5N8?usp=drive_link>

The folder contained 34 screenshots spanning `IMG_7999.PNG` through `IMG_8034.PNG`, with camera-number gaps at 8000 and 8005. The accompanying transcript supplies a continuous talk and no essential argumentative gap was identified.

### Transcript

Local supplied transcript:

`C:\Users\17076\.codex\attachments\9a5864e3-0839-479d-b357-3681fb37178f\pasted-text.txt`

Talk title: *Rich Sutton, The OaK Architecture: A Vision of SuperIntelligence from Experience — RLC 2025*.

## 3. Public primary sources

1. Oak Lab mission: <https://oaklab.ai/mission>
2. Oak Lab event-driven placeholder: <https://oaklab.ai/posts/event-driven-computation-with-batch-size-one>
3. Sutton et al., *Reward-Respecting Subtasks for Model-Based Reinforcement Learning*: <https://arxiv.org/abs/2202.03466>
4. Javed and Sutton, *The Need for a Big World Simulator: A Scientific Challenge for Continual Learning*: <https://arxiv.org/abs/2408.02930>
5. Dohare et al., *Loss of plasticity in deep continual learning*: <https://www.nature.com/articles/s41586-024-07711-7>

## 4. Explicit OaK claims used by BONSAI

| Claim | Presentation evidence | BONSAI consequence |
|---|---|---|
| OaK means Options and Knowledge | Name discussion; `IMG_8004.PNG`; transcript approximately 3:27–4:18 | Options and option-model knowledge require distinct identities and metrics. |
| An option is a policy plus a stopping rule | `IMG_8004.PNG`; transcript approximately 3:37–3:58 | Option lifecycle must record both policy and termination semantics. |
| The desired agent is domain general, experiential, and open-ended | `IMG_8006.PNG` and `IMG_8007.PNG`; transcript approximately 4:43–7:36 | Canonical track excludes domain labels, offline training, and undeclared built-in abstractions. |
| The world is larger and more complex than the agent | `IMG_8012.PNG` and `IMG_8013.PNG`; transcript approximately 13:38–18:48 | Approximation, apparent nonstationarity, limited resources, and lifelong adaptation must be tested. |
| Experience is streamed and never saved by the agent | `IMG_8015.PNG`; transcript problem formulation | Strict track requires single-pass agent access and no replay. |
| OaK adds auxiliary problems for attaining individual features | `IMG_8017.PNG` and `IMG_8018.PNG`; transcript approximately 25:48–29:30 | Features, subproblems, options, and value functions need traceable relationships. |
| Eight processes operate in parallel at runtime | `IMG_8019.PNG` and `IMG_8020.PNG`; transcript approximately 29:47–33:41 | Measurement must handle concurrent logical work and resource arbitration. |
| Feature generation has no specific complete proposal | `IMG_8020.PNG`, `IMG_8032.PNG`; transcript approximately 31:16–31:30 and 55:03–57:18 | Feature discovery is a measured experimental seam, not an assumed solved component. |
| Subproblems should be reward-respecting feature-attainment problems | `IMG_8022.PNG` and `IMG_8023.PNG`; transcript approximately 37:36–42:48 | Subproblem evidence includes the attained feature, intensity, real reward, and terminal-state value. |
| Solved subproblems produce options; option prediction produces models; models support planning | `IMG_8024.PNG`–`IMG_8026.PNG`; transcript approximately 42:51–47:55 | BONSAI observes the full STOMP-like progression and downstream contribution. |
| Planning uses temporally extended option models for larger jumps | `IMG_8027.PNG`–`IMG_8030.PNG`; transcript approximately 48:22–53:09 | Planning metrics must use primitive-time equivalents and model-error-by-jump-length. |
| Reliable continual deep learning is required and unresolved | `IMG_8031.PNG`; transcript approximately 53:40–55:03 | Lifelong plasticity and forgetting require distinct gates. |
| New state-feature discovery is required and unresolved | `IMG_8032.PNG`; transcript approximately 55:03–57:18 | Open-endedness cannot be inferred from a fixed representation. |
| Each forward dependency has a slower backward flow of utility credit | `IMG_8033.PNG`; transcript approximately 43:58–46:16 and 57:28–58:03 | Consumer-to-provider credit, latency, and retention decisions are first-class telemetry. |
| OaK is not fully specified | `IMG_8034.PNG`; transcript approximately 59:33–1:00:58 | BONSAI must remain algorithm-neutral and qualify claims. |

## 5. Claims supported by primary papers

### Reward-respecting subtasks

The public paper supports:

- the SubTask → Option → Model → Planning progression;
- use of original reward plus a stopping bonus based on a state feature;
- online and off-policy learning using general value functions;
- evidence in small tabular/linear settings that reward-respecting options can improve planning relative to selected alternatives;
- the conclusion that feature selection becomes central to option discovery.

Qualification: the paper does not establish a complete deep, nonlinear, continually changing, open-ended OaK agent. Its state-abstraction and nonlinear planning limitations remain material.

### Continual plasticity

The public loss-of-plasticity work supports:

- separating loss of plasticity from catastrophic forgetting;
- the finding that conventional deep learners can lose learning ability over long continual operation;
- continual backpropagation as one method that maintains variability by selectively replacing low-utility units.

Qualification: maintaining plasticity does not by itself solve semantic feature discovery, downstream utility assignment, option-model stability, or open-ended abstraction.

### Big-world evaluation

The big-world simulator proposal supports the need for long-running, nonstationary evaluation environments that expose limitations hidden by short, reset-heavy benchmarks.

Qualification: no single simulator can establish domain generality or open-ended intelligence.

## 6. BONSAI working inferences

The following are reasoned interpretations, not confirmed Oak Lab implementation details:

### 6.1 Event-driven computation as an architectural scaling requirement

Because OaK proposes many simultaneous learners while treating compute as the limiting resource, BONSAI hypothesizes that event-driven scheduling must prevent every experience from waking every feature, option, model, and planning process.

This is consistent with Oak Lab's mission statement but not yet specified by its coming-soon event-driven post.

### 6.2 Open-endedness as resource-positive recursive reuse

BONSAI interprets meaningful open-endedness as multiple generations of useful abstractions whose downstream benefits continue under controlled resources. Mere indefinite generation is insufficient.

### 6.3 Utility as marginal downstream contribution

Sutton's backward-credit language motivates measuring whether an artifact changes prediction, control, modeling, planning, or adaptation. The exact utility algorithm remains unspecified.

### 6.4 External resource governor

BONSAI places final enforcement authority outside the evaluated agent so learned scheduling can be tested without allowing self-reported compliance to determine the result. This is a BONSAI experimental-design decision, not an OaK claim.

### 6.5 Observer-retained telemetry is not agent memory

BONSAI permits audit telemetry only when it is inaccessible to the agent. This preserves scientific reproducibility while retaining the canonical no-replay semantics.

## 7. Questions awaiting future evidence

- What precisely constitutes an event in Oak Lab's event-driven networks?
- Is event routing based on activation, prediction error, eligibility, learned utility, parameter credit, or another mechanism?
- How does Network-IDBD interact with feature generation, feature replacement, and deep nonlinear credit assignment?
- How are feature intensities or stopping bonuses generated and governed?
- How is downstream utility propagated across changing representations?
- What prevents option populations from becoming redundant or semantically unstable?
- How are learned option models composed when underlying feature semantics change?
- What search-control mechanism allocates planning computation?
- What evidence would Oak Lab accept as demonstrating an operating cycle of discovery?

These questions should remain explicit seams in the PSPR rather than being answered by assumption.

