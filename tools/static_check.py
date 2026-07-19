"""CognitiveOS static conformance checks (Review-Conclusions section 8 checklist).

Read-only verification:
1. Parse all schemas, vectors, registries, transition tables.
2. Validate every schema against JSON Schema draft 2020-12 and resolve all $refs.
3. Transition tables validate against state-transition-table.schema.json; states closed,
   terminal states have no outgoing edges, initial state declared.
4. REQ <-> vector bidirectional closure; vector error codes registered; owner_spec files exist.
5. Legacy shape ban: no spec schema references common-defs metadata/strongRef.
6. Counterexamples must be rejected and positive fixtures accepted by current schemas.

Exit code 0 only if every check passes.
"""
import glob
import json
import os
import sys

import yaml
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012

FAILURES = []


def check(name, ok, detail=''):
    status = 'PASS' if ok else 'FAIL'
    print('%s  %s%s' % (status, name, (' -- ' + detail) if detail else ''))
    if not ok:
        FAILURES.append(name)


# ---------- 1. Parse everything ----------
schemas = {}
for path in sorted(glob.glob('specs/schemas/*.json')):
    schemas[os.path.basename(path)] = json.load(open(path, encoding='utf-8'))
check('parse schemas (%d)' % len(schemas), len(schemas) == 56, 'expected 56')

vectors = {}
for path in sorted(glob.glob('conformance/vectors/*.json')):
    vectors[os.path.basename(path)] = json.load(open(path, encoding='utf-8'))
check('parse vectors (%d)' % len(vectors), len(vectors) == 76, 'expected 76')

transitions = {}
for path in sorted(glob.glob('specs/transitions/*.json')):
    transitions[os.path.basename(path)] = json.load(open(path, encoding='utf-8'))
check('parse transition tables (%d)' % len(transitions), len(transitions) == 5, 'expected 5')

registries = {}
for path in sorted(glob.glob('specs/registry/*.yaml')):
    registries[os.path.basename(path)] = yaml.safe_load(open(path, encoding='utf-8'))
check('parse registries (%d)' % len(registries), len(registries) == 3, 'expected 3')

# ---------- 2. Meta-schema validation and $ref resolution ----------
resources = []
for name, doc in schemas.items():
    res = Resource.from_contents(doc, default_specification=DRAFT202012)
    resources.append((name, res))
    if '$id' in doc:
        resources.append((doc['$id'], res))
    resources.append(('https://schemas.cognitiveos.dev/governance/' + name, res))
registry = Registry().with_resources(resources)

meta_ok = True
for name, doc in schemas.items():
    try:
        Draft202012Validator.check_schema(doc)
    except Exception as exc:
        meta_ok = False
        print('  meta-schema failure in %s: %s' % (name, exc))
check('all schemas valid against draft 2020-12 meta-schema', meta_ok)


def walk_refs(node, base_name, errors):
    if isinstance(node, dict):
        ref = node.get('$ref')
        if isinstance(ref, str) and not ref.startswith('#'):
            target = ref.split('#')[0]
            frag = ref.split('#')[1] if '#' in ref else ''
            fname = os.path.basename(target)
            if fname not in schemas:
                errors.append('%s -> missing file %s' % (base_name, ref))
            elif frag:
                doc = schemas[fname]
                cur = doc
                ok = True
                for part in [p for p in frag.split('/') if p]:
                    part = part.replace('~1', '/').replace('~0', '~')
                    if isinstance(cur, dict) and part in cur:
                        cur = cur[part]
                    else:
                        ok = False
                        break
                if not ok:
                    errors.append('%s -> unresolvable pointer %s' % (base_name, ref))
        for v in node.values():
            walk_refs(v, base_name, errors)
    elif isinstance(node, list):
        for v in node:
            walk_refs(v, base_name, errors)


ref_errors = []
for name, doc in schemas.items():
    walk_refs(doc, name, ref_errors)
for e in ref_errors:
    print('  ref error:', e)
check('all relative $refs resolve (file + pointer)', not ref_errors)

# ---------- 3. Transition table closure ----------
table_schema = schemas['state-transition-table.schema.json']
tt_validator = Draft202012Validator(table_schema, registry=registry)
tt_ok = True
for name, table in transitions.items():
    errs = list(tt_validator.iter_errors(table))
    if errs:
        tt_ok = False
        print('  %s fails table schema: %s' % (name, errs[0].message))
    states = set(table['states'])
    terminals = set(table['terminal_states'])
    if not terminals <= states:
        tt_ok = False
        print('  %s: terminal states not subset' % name)
    if table['initial_state'] not in states:
        tt_ok = False
        print('  %s: initial state undeclared' % name)
    for t in table['transitions']:
        if t['from'] not in states or t['to'] not in states:
            tt_ok = False
            print('  %s: undeclared state in edge %s->%s' % (name, t['from'], t['to']))
        if t['from'] in terminals:
            tt_ok = False
            print('  %s: terminal %s has outgoing edge' % (name, t['from']))
check('transition tables closed (declared states, no terminal exits)', tt_ok)

# ---------- 4. Registry closure ----------
reqs = registries['requirements.yaml']['requirements']
req_ids = {r['id'] for r in reqs}
check('requirement count = 273, unique ids', len(reqs) == 273 and len(req_ids) == 273,
      'count=%d unique=%d' % (len(reqs), len(req_ids)))

errors_reg = registries['errors.yaml']
error_codes = {e['code'] for e in errors_reg['errors']} if isinstance(errors_reg, dict) else {e['code'] for e in errors_reg}
check('error code count = 55', len(error_codes) == 55, 'count=%d' % len(error_codes))

owner_missing = [r['id'] for r in reqs if not os.path.exists(r['owner_spec'].split('#')[0])]
check('all owner_spec files exist', not owner_missing, str(owner_missing[:5]))

informative_owner = [r['id'] for r in reqs
                     if 'CognitiveOS-Architecture.md' in r['owner_spec']
                     or r['owner_spec'].startswith('History/')]
check('no informative/historical owner_spec', not informative_owner, str(informative_owner[:5]))

vec_ids = {v['id'] for v in vectors.values()}
check('vector ids unique (%d)' % len(vec_ids), len(vec_ids) == len(vectors))

req_test_refs = {t for r in reqs for t in r.get('tests', [])}
orphan_tests = sorted(req_test_refs - vec_ids)
check('registry tests all point at existing vectors', not orphan_tests, str(orphan_tests[:5]))

vec_req_refs = {rid for v in vectors.values() for rid in v.get('requirement_ids', [])}
orphan_vec_reqs = sorted(vec_req_refs - req_ids)
check('vector requirement_ids all registered', not orphan_vec_reqs, str(orphan_vec_reqs[:5]))

unref_vectors = sorted(vec_ids - req_test_refs)
check('no orphan vectors (unreferenced by any REQ)', not unref_vectors, str(unref_vectors[:5]))


def collect_error_codes(node, out):
    if isinstance(node, dict):
        for k, v in node.items():
            if k == 'code' and isinstance(v, str) and v.isupper() and '_' in v:
                out.add(v)
            else:
                collect_error_codes(v, out)
    elif isinstance(node, list):
        for v in node:
            collect_error_codes(v, out)


vec_error_refs = set()
for v in vectors.values():
    collect_error_codes(v.get('input', {}), vec_error_refs)
    collect_error_codes(v.get('expected', {}), vec_error_refs)
unregistered = sorted(vec_error_refs - error_codes)
check('vector error codes all registered', not unregistered, str(unregistered[:8]))

# ---------- 5. Legacy shape ban ----------
legacy_hits = []
for name, doc in schemas.items():
    if name == 'common-defs.schema.json':
        continue
    raw = json.dumps(doc)
    if 'common-defs.schema.json#/$defs/metadata' in raw or 'common-defs.schema.json#/$defs/strongRef' in raw:
        legacy_hits.append(name)
check('no schema references legacy metadata/strongRef', not legacy_hits, str(legacy_hits[:8]))

MIGRATED_HEADER_FILES = [
    'agent-compatibility-report.schema.json', 'agent-installation.schema.json',
    'agent-package-manifest.schema.json', 'authorization-capability.schema.json',
    'cognitive-allocation-decision.schema.json', 'cognitive-resource-manifest.schema.json',
    'context-request-admission.schema.json', 'context-request.schema.json',
    'context-view-delta.schema.json', 'context-view.schema.json', 'delegation.schema.json',
    'effect.schema.json', 'event.schema.json', 'information-gap.schema.json',
    'intent-interpretation.schema.json', 'intent.schema.json', 'loop-checkpoint.schema.json',
    'memory-admission-decision.schema.json', 'memory-candidate.schema.json',
    'memory-object.schema.json', 'operation-catalog-snapshot.schema.json',
    'operation-match-report.schema.json', 'operation-summary.schema.json',
    'placement-manifest.schema.json', 'resource-graph.schema.json',
    'semantic-service-manifest.schema.json', 'task-contract.schema.json',
    'user-intent-record.schema.json', 'verification-report.schema.json',
    'world-state.schema.json',
]
not_migrated = [n for n in MIGRATED_HEADER_FILES
                if 'governed-object-header.schema.json' not in json.dumps(schemas[n])
                or '"header"' not in json.dumps(schemas[n])]
check('all 30 previously-legacy schemas carry GovernedObjectHeader under `header`',
      not not_migrated, str(not_migrated[:8]))

# ---------- 6. Fixtures: positives accepted, counterexamples rejected ----------
HEADER = {
    'id': '01890a5d-ac96-774b-bcce-b302099a8057',
    'type': 'Effect',
    'schema_version': 'cognitiveos.effect/0.2',
    'object_version': 3,
    'scope_domain': 'tenant',
    'tenant_id': '01890a5d-ac96-774b-bcce-b302099a8058',
    'resource_scope_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a8059',
                           'object_version': 1, 'content_digest': 'sha256:' + 'a' * 64},
    'owner_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a805a',
                  'object_version': 1, 'content_digest': 'sha256:' + 'b' * 64},
    'authority_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a805b',
                      'object_version': 1, 'content_digest': 'sha256:' + 'c' * 64},
    'policy_refs': [{'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a805c',
                     'object_version': 1, 'content_digest': 'sha256:' + 'd' * 64}],
    'purpose_constraints': ['task-execution'],
    'sensitivity': 'internal',
    'compartments': [],
    'retention': {'policy': 'default-90d', 'expires_at': None, 'legal_hold': False},
    'provenance': {'created_by': 'kernel://effect-authority', 'source_refs': []},
    'lineage': {'parents': [], 'transform': 'created'},
    'content_digest': 'sha256:' + 'e' * 64,
    'created_at': '2026-07-20T00:00:00Z',
    'valid_time': {'from': '2026-07-20T00:00:00Z', 'until': None},
}


def effect_base(**overrides):
    doc = {
        'header': dict(HEADER),
        'intent_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a805d',
                       'object_version': 1, 'content_digest': 'sha256:' + 'f' * 64},
        'state': 'PROPOSED',
        'executor': 'executor://payments',
        'attempt': 1,
        'idempotency_key': 'idem-0001-key',
        'parameters_digest': 'sha256:' + '1' * 64,
        'authorization_digest': 'sha256:' + '2' * 64,
        'observed_outcome': 'not_observed',
        'verification': {'status': 'pending'},
        'decision': 'pending',
        'event_refs': [],
    }
    doc.update(overrides)
    return doc


effect_validator = Draft202012Validator(schemas['effect.schema.json'], registry=registry)


def valid(validator, instance):
    return not list(validator.iter_errors(instance))


# positive: proposed effect
check('POSITIVE proposed Effect accepted', valid(effect_validator, effect_base()))

# positive: committed via reconciled-executed path with unknown observation preserved
committed_unknown = effect_base(
    state='COMMITTED', observed_outcome='unknown', reconciliation_result='executed',
    reconciliation_report_ref='report://reconcile/1',
    verification={'status': 'passed', 'report_ref': 'report://verify/1'}, decision='commit')
check('POSITIVE COMMITTED with observed unknown + reconciled executed accepted',
      valid(effect_validator, committed_unknown))

# positive: reconciled still_unknown
rec_unknown = effect_base(state='RECONCILED', observed_outcome='unknown',
                          reconciliation_result='still_unknown',
                          reconciliation_report_ref='report://reconcile/2',
                          verification={'status': 'unresolved'})
check('POSITIVE RECONCILED still_unknown accepted', valid(effect_validator, rec_unknown))

# counterexample 1: COMMITTED + unknown outcome + pending verification (historic)
ce1 = effect_base(state='COMMITTED', observed_outcome='unknown',
                  reconciliation_result='executed', reconciliation_report_ref='report://r/3',
                  verification={'status': 'pending'}, decision='pending')
check('NEGATIVE COMMITTED+pending verification rejected', not valid(effect_validator, ce1))

# counterexample 2: COMMITTED with reconciliation still_unknown
ce2 = effect_base(state='COMMITTED', observed_outcome='unknown',
                  reconciliation_result='still_unknown', reconciliation_report_ref='report://r/4',
                  verification={'status': 'passed', 'report_ref': 'report://verify/2'},
                  decision='commit')
check('NEGATIVE COMMITTED with still_unknown reconciliation rejected',
      not valid(effect_validator, ce2))

# counterexample 3: COMMITTED without reconciliation_result
ce3 = effect_base(state='COMMITTED', observed_outcome='succeeded',
                  verification={'status': 'passed', 'report_ref': 'report://verify/3'},
                  decision='commit')
check('NEGATIVE COMMITTED without reconciliation_result rejected',
      not valid(effect_validator, ce3))

# counterexample 4: RECONCILED without result fields
ce4 = effect_base(state='RECONCILED', observed_outcome='succeeded',
                  verification={'status': 'pending'})
check('NEGATIVE RECONCILED without reconciliation fields rejected',
      not valid(effect_validator, ce4))

# counterexample 5: pre-reconciliation state carrying reconciliation_result
ce5 = effect_base(state='EXECUTING', reconciliation_result='executed')
check('NEGATIVE EXECUTING carrying reconciliation_result rejected',
      not valid(effect_validator, ce5))

# counterexample 6: effect without governance header (legacy shape)
ce6 = effect_base()
del ce6['header']
check('NEGATIVE Effect without GovernedObjectHeader rejected', not valid(effect_validator, ce6))

# counterexample 7: tenant-scoped header missing tenant_id
ce7 = effect_base()
ce7['header'] = dict(HEADER)
del ce7['header']['tenant_id']
check('NEGATIVE tenant-scoped header missing tenant_id rejected', not valid(effect_validator, ce7))

# TaskContract: wait-only contract must be rejected
tc_validator = Draft202012Validator(schemas['task-contract.schema.json'], registry=registry)
tc_header = dict(HEADER, type='TaskContract', schema_version='cognitiveos.task-contract/0.2')
tc = {
    'header': tc_header,
    'task_ref': 'task://t1',
    'contract_epoch': 1,
    'intent_acceptance_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a8061',
                              'object_version': 1, 'content_digest': 'sha256:' + '3' * 64},
    'intent_interpretation_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a8062',
                                  'object_version': 1, 'content_digest': 'sha256:' + '4' * 64},
    'user_intent_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a8063',
                        'object_version': 1, 'content_digest': 'sha256:' + '5' * 64},
    'objective': 'ship the fix',
    'scope': {'in_scope': ['repo'], 'out_of_scope': []},
    'conditions': [{'id': 'c1', 'kind': 'wait', 'description': 'wait for CI'}],
    'budget': {'wall_time_ms': 60000},
    'max_iterations': 5,
    'max_retries': 1,
    'allowed_state_domains': ['task'],
    'allowed_tools': [],
}
check('NEGATIVE TaskContract without acceptance condition rejected', not valid(tc_validator, tc))
tc_ok = dict(tc)
tc_ok['conditions'] = tc['conditions'] + [
    {'id': 'c2', 'kind': 'acceptance', 'description': 'verifier passes', 'verifier_ref': 'verifier://v1'}]
check('POSITIVE TaskContract with acceptance condition accepted', valid(tc_validator, tc_ok))

# ContextView: untrusted+control item must be rejected
cv_validator = Draft202012Validator(schemas['context-view.schema.json'], registry=registry)
cv_header = dict(HEADER, type='ContextView', schema_version='cognitiveos.context-view/0.2')
item = {
    'item_id': 'item://1',
    'object_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a8064',
                   'object_version': 1, 'content_digest': 'sha256:' + '6' * 64},
    'representation': 'text',
    'trust_level': 'untrusted',
    'role': 'control',
    'cost': {'bytes': 10},
}
cv = {
    'header': cv_header,
    'request_ref': {'kind': 'strong', 'id': '01890a5d-ac96-774b-bcce-b302099a8065',
                    'object_version': 1, 'content_digest': 'sha256:' + '7' * 64},
    'complete': True,
    'loaded': [item],
    'rejected': [],
    'loss_declaration': [],
    'pinned_versions': {'world': 4},
    'cost': {'bytes': 10, 'resolve_ms': 5},
    'activity_bound': 'activity://a1',
}
check('NEGATIVE ContextView untrusted+control item rejected', not valid(cv_validator, cv))
cv_ok = json.loads(json.dumps(cv))
cv_ok['loaded'][0]['role'] = 'untrusted_input'
check('POSITIVE ContextView untrusted+untrusted_input accepted', valid(cv_validator, cv_ok))

# performance-report vector instance still validates against schema
perf_vec = vectors['performance-report-contract.json']
perf_validator = Draft202012Validator(schemas['performance-report.schema.json'], registry=registry)
report_instance = perf_vec.get('input', {}).get('report')
if report_instance is None:
    check('performance report vector embeds report instance', False)
else:
    check('performance report vector instance schema-valid', valid(perf_validator, report_instance))

print()
if FAILURES:
    print('%d CHECK(S) FAILED:' % len(FAILURES))
    for f in FAILURES:
        print(' -', f)
    sys.exit(1)
print('ALL CHECKS PASSED')
