<!--
Task: P8-T03
Slice: D04
Classification: MVP task closure
Status: in-progress pending required CI
-->

# P8-T03 First non-Pi Agent qualification (Codex) closure

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| Select mainstream CLI agent; fixture package identity independent of Pi | D01 Codex (`openai.codex.cli`); Linux at `12c8b3e`; Pi-evidence-transfer / public-listener / authority-writer negatives |
| Lifecycle activate/pause/stop/recover over digests with channel isolation | D02 management-channel lifecycle; Linux 3/3 + Clippy at `6847889`; Task-channel negative |
| Fixed-denominator independent qualification matrix / non-claim report | D03 `build_codex_qualification_report`; Linux 4/4 + Clippy at `b41f06f`; Gate/B09-shaped claim rejection |
| final acceptance / docs / PR / lease / branch closure | this checkpoint; required CI pending on Draft PR #191 HEAD `b41f06f` |

## Non-claims

No Gate, release, Profile, GMVP-LINUX, B09 transfer, live Codex production
install, or Pi evidence inheritance. Fixture qualification only.

## Closure

Required Ubuntu/Windows CI for HEAD will be recorded when green. PR #191 is the
task closure PR. Lease `lease/personal/P8-T03/non-pi-agent` closes with merge.
