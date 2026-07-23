# Lane-CTR v0.2 WP-0 path-set algorithm owner-decision handoff

- Date: 2026-07-23
- Branch: `lane/ctr-v02-audit-privileged-read-registration`
- Classification: owner governance decision; docs-only
- Decision: **ordinal path sort → UTF-8 without BOM → LF join + trailing LF → SHA-256**

The owner confirmed the sole comparison procedure for the bypass path set. It
applies only to path names; it does not authorize reading content, staging,
modifying, cleaning, or otherwise processing bypass files. Future records may
compare count and this digest only when the same procedure is used.
