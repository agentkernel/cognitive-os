# Security negative tests (placeholder)

Security negatives land here starting M3 (tenant lateral movement, rank
before auth, revocation cache reuse, prompt-injection isolation — see the
`security-negative` vectors). Every security feature ships with at least one
negative test (`.cursor/rules/14-security-testing.mdc`); safety negatives
can never be degraded away (`conformance/README.md`).
