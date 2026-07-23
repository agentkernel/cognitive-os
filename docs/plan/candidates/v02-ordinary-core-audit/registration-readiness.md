# Core-only registration readiness

| Item | Candidate status | Real feedback | Still needed only at registration |
|---|---|---|---|
| decision schema | frozen for review | internal type + success/denied/error tests | independent final review and machine placement |
| receipt schema | frozen for review | release gate + durable adapter | independent final review and binding generation |
| port responsibility | frozen for review | `ManagementAuditPort` consumer and `admin-cli` product route | registered port identity/version |
| digest rules | frozen for review | repository canonical implementation computes all values | final asset domain selection/review |
| error mapping | candidate reuse ruling | zero-result failure behavior | reviewer confirmation; no new error proposed |
| sequence/epoch/order | frozen for review | file adapter tests and release gate | no checkpoint/export requirement for Core |

No row is machine registered, published, selected, a conformance behavior pass,
CA-0 GO, or Profile implementation.
