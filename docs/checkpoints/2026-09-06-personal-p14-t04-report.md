# P14-T04 member join Dual Track — running report

- Task / slice: `P14-T04/D01` Dual Track member join on real PlanRevision slots
- Lease: `lease/personal/P14-T04/member-join`
- Branch: `personal/P14-T04-member-join`
- Draft PR: pending first coherent commit
- Change class: `implementation-only` (Dual Track process-ring ids become PlanRevision `responsible_slot`; write join seats those slots; no `core/specs`; no numbered migration — v41 remains reserved). Handbook: `dev.store-migrations` + `dev.daemon-http-surface` + regenerated `ref.http-api` (both locales).
- Claim ceiling: `hypothesis`
- Product origin: daemon `/ui/` (`http://127.0.0.1:48681/ui/`) — Vite is not the product source
- Evaluation routing: **OFF**
- Do not claim T05/T06/T07/T08. T02/T03/T07/T08 remain **done**.

## Failure-first (D01)

Observed fail on current `main@ed893951` (T03 Dual Track collapsed `rights=owner` into one slot): Dual Track activation minted three `owner` slots, so joining `collect`/`analyze`/`draft` was `roster missing slot coverage`. Fake `manager` join and G1-without-plan already fail-closed.

Fix: `parse_dual_track_stages` mints `responsible_slot = stage_id`. `rights=` stays Owner access in the objective text, not a seating slot. Charter blob now also records `slot=${ring.id}`.

## Local development evidence (MSVC override; not supported CI)

- `rustc -vV` host `x86_64-pc-windows-msvc`; `CARGO_PROFILE_DEV_DEBUG=0`
- `cargo test -p cognitive-store --test p14_t04_member_join --locked -- --test-threads=1` **5/5 pass** (`dual_track_activation_mints_responsible_slots`, `write_join_seats_members_on_plan_revision_slots`, `no_slot_fake_join_is_refused`, `surplus_slot_join_does_not_seat`, `chat_approve_must_not_join`)
- `cargo test -p kernel-server dual_track_http_join --locked -- --test-threads=1` **1/1 pass** (`dual_track_http_join_seats_ring_slots_and_refuses_chat`: axis collect/analyze/draft, fake `manager` 422, task-channel register 403, management register+seat 3 seated)
- `cargo test -p cognitive-store --test p14_t03_write_live_project --locked -- --test-threads=1` **4/4 pass** (T03 regression)
- Dual Track TS `pnpm test` in `clients/pc/web` **73 files / 529 tests** (includes Dual Track ring-slot Write join + `uniqueResponsibleSlots` collect/analyze/draft)
- `cargo fmt --all -- --check` **pass**

## JOURNEY (`JOURNEY-BROWSER-SYNC-01`)

D02 not started. Product origin remains guest daemon `/ui/`.

| Journey | Result |
|---|---|
| J4 member join | **not-run** (D02) |
| J1 regression | **not-run** (D02) |
| J0 / J10 / J18 / J19 | **not-run** (D02) |
| Windows chrome | **not-run** |

## Unique next

Push D01 Draft PR, then `P14-T04/D02` guest `/ui/` J4 + `JOURNEY-BROWSER-SYNC-01` (include J1 regression). Do not claim T05/T06.
