# `@cognitiveos/dsh-akp-adapter`

This package is the client-side bridge for DeepSeek Harness (`dsh`). It keeps
the dsh plugin API at the edge and sends only bounded, candidate-only events to
the CognitiveOS AKP boundary. It does not hold a daemon bearer in plugin
payloads, a Provider key, an authority writer, or Task-completion capability.
HTTP bearers are supplied by the harness constructor and are never logged.

Pinned identity:

- dsh git revision: `528c682e061696f5a160f363f236ecbf53cbd006`
- AKP request-envelope schema digest:
  `sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b`
- bridge protocol: `cognitiveos.dsh-akp/0.1`

The wire format is snake_case JSON so the Rust daemon can parse it. Frames are
capped at 1 MiB. `JsonlAkpTransport` is the long-lived child-process transport;
`HttpAkpTransport` posts to authenticated `POST /task/akp/dsh`. HTTP sessions
must be activated after daemon start; a restart empties the process-local
session table and fails closed.

Workspace* candidates are mapped by the daemon onto the existing public
candidate admission path. Observation and lifecycle events are accepted without
authority writes. A dsh response is never Task completion.

The adapter records serialization, transport, and total durations separately.
It does not claim zero overhead. Measure paired dsh-direct versus
dsh → AKP → daemon runs with warm and cold process conditions, and report
p50/p95, TTFT, Provider network time, and retained failures. Live linux-002
results are implementation evidence, not a Gate, release, Profile, B01, or
Agent-benefit claim.
