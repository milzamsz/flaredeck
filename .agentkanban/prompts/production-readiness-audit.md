# Prompt — production-readiness audit

Standalone gate before moving a task or release to `done`. Built around the lesson: a board
"done" is worthless without evidence the behavior RUNS. Produces a PASS/FAIL report in the task file.

---

```markdown
# PRODUCTION READINESS AUDIT

## Target
- Task / release: `<name>`
- Capability spec: `.agentkanban/specs/<cap>/spec.md`
- Env exercised: `<local / staging / prod-like>`

## Audit (mark each PASS / FAIL / N/A with evidence — paste output)

### Correctness & "does it actually run"
- [ ] Checklist + spec acceptance criteria met
- [ ] `` · `` · `` green (output pasted)
- [ ] Route smoke / integration smoke green
- [ ] **Behavior proven to RUN**, not a DB row: the spec's Verification evidence (real workflow/job
      id, agent command executed on host, quota 429, S3 object, webhook state change). Quote it.
- [ ] No silent mock fallback left in the exercised path (fails closed without repo/orgID)

### Multi-tenant & security
- [ ] Every query org-scoped (`organization_id`); cross-org access tested negative
- [ ] Mutation writes an audit event; multi-statement mutation wrapped in a DB transaction
- [ ] Secrets reference-backed, masked, reveal audited, never logged
- [ ] Agent commands typed/allowlisted/signed (TTL + idempotency); no raw shell
- [ ] Entitlement/quota enforced where the task creates billable resources

### Reliability & ops
- [ ] Error handling + graceful failure; failed provisioning/command leaves no orphan (cleanup/compensation)
- [ ] Idempotent migration; verified on a clean DB
- [ ] Logging at useful levels, no secrets; health/metrics where applicable
- [ ] Backup/rollback path documented if the task touches data or deploy

### Performance
- [ ] No N+1 / unbatched hot paths; indexes present on lookup columns

### Docs (honest-state)
- [ ] README / architecture / spec updated to reflect what is now real vs still stubbed
- [ ] TECHNICAL.md / spec updated where behavior changed

## Output
Write a PASS/FAIL summary in the task file. ANY unresolved FAIL on Correctness / security /
reliability blocks `done` — list it, then fix or `block` with a reason. Mark untested checks
`not-run`; never imply coverage you don't have. Evidence over assertion.
```

