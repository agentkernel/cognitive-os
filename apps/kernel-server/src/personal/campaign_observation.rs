//! P2-T17 A7 评测用外部变更观察：回环幂等 fixture、persist-before-dispatch、
//! 原键对账与恰好一次证据。本地/fixture 结果不得升格为 Gate/release/Profile。

use cognitive_contracts::{
    canonical,
    generated::{
        common_defs::Budget,
        governed_object_header::GovernedObjectHeaderSensitivity,
        task_contract::{ContractCondition, ContractConditionKind, TaskContract, TaskScope},
    },
};
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, StateName, UriRef, Version, WallTimestamp,
    capability::{CapabilityConstraints, LeaseWindow},
};
use cognitive_kernel::{
    authz::{
        AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, MembershipFacts,
        ObjectGovernance, PrincipalFacts, authorize,
    },
    budget::BudgetState,
    effects::{EffectProtocol, GovernanceCurrency, WriterLease},
    executor::{
        DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    },
    intent_chain::{
        GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
        strong_reference_to,
    },
    ports::{
        AuthorityStore, Clock, EventDraft, IdGenerator, IntentChainStore, IntentRow,
        ObjectAdmission, PortFailure, ProtocolStore, StoredObject, TaskBinding, TaskContractRow,
    },
};
use cognitive_store::{ArtifactStore, SqliteAuthorityStore, SystemClock, UuidV7Generator};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use thiserror::Error;

const CAMPAIGN_ID: &str = "PERSONAL-PERF-EVAL-003";
const FIXTURE_STATE_SCHEMA: &str = "cognitiveos.personal.a7-fixture/0.1";
const RUN_STATE_SCHEMA: &str = "cognitiveos.personal.a7-observation/0.1";
const MUTATION_DIGEST_DOMAIN: &str = "personal-a7-external-mutation/0.1";
const POST_STATE_DIGEST_DOMAIN: &str = "personal-a7-external-post-state/0.1";
const MAXIMUM_HTTP_BYTES: usize = 16 * 1024;
const ARTIFACT_MAXIMUM_BYTES: usize = 8 * 1024 * 1024;
const FIXED_EFFECT_VERIFIER_REF: &str = "verifier://personal/fixed-effect";
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static ACTIVE_RUNS: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FixtureBounds {
    pub maximum_records: usize,
    pub maximum_absolute_delta: i64,
}

/// 评测授权的故障注入点。默认关闭；未授权请求不得触达外部变更。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CampaignFaultPoint {
    DispatchBefore,
    MutationAfterReceiptBefore,
    ReceiptAfterEffectCloseBefore,
    VerificationBefore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FixtureQueryFault {
    Normal,
    Timeout,
    Ambiguous,
    TamperedPostStateDigest,
    DuplicateMutationCount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CampaignAuthorization {
    campaign_id: String,
    case_ref: String,
    faults_enabled: bool,
}

impl CampaignAuthorization {
    pub(crate) fn authorized(
        campaign_id: &str,
        case_ref: &str,
    ) -> Result<Self, CampaignObservationError> {
        if !cfg!(test) {
            return Err(CampaignObservationError::FaultHooksUnavailable);
        }
        if campaign_id != CAMPAIGN_ID || !valid_case_ref(case_ref) {
            return Err(CampaignObservationError::CampaignUnauthorized);
        }
        Ok(Self {
            campaign_id: campaign_id.to_owned(),
            case_ref: case_ref.to_owned(),
            faults_enabled: true,
        })
    }

    /// 评测授权存在，但故障注入默认关闭。
    pub(crate) fn authorized_faults_disabled(
        campaign_id: &str,
        case_ref: &str,
    ) -> Result<Self, CampaignObservationError> {
        let mut authorization = Self::authorized(campaign_id, case_ref)?;
        authorization.faults_enabled = false;
        Ok(authorization)
    }

    fn verify(&self) -> Result<(), CampaignObservationError> {
        if self.campaign_id != CAMPAIGN_ID || !valid_case_ref(&self.case_ref) {
            return Err(CampaignObservationError::CampaignUnauthorized);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CampaignMutationRequest {
    pub task_ref: String,
    pub expected_fixture_version: i64,
    pub delta: i64,
    pub scheduler_lease_epoch: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PreparedCampaignMutation {
    pub run_ref: String,
    pub effect_ref: String,
    pub idempotency_key_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CampaignOutcomeClass {
    Prepared,
    Indeterminate,
    NotExecuted,
    ReconciledExecuted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CampaignStageTiming {
    pub stage: String,
    pub elapsed_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CampaignMutationObservation {
    pub schema_version: &'static str,
    pub run_ref: String,
    pub outcome_class: CampaignOutcomeClass,
    pub idempotency_key_digest: String,
    pub idempotency_key_ref: String,
    pub mutation_count: u64,
    pub post_state_digest: Option<String>,
    pub stage_timings: Vec<CampaignStageTiming>,
    pub effect_ref: String,
    pub verification_report_ref: Option<String>,
    pub acceptance_ref: Option<String>,
    pub cleanup: CampaignCleanupObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CampaignCleanupObservation {
    pub fixture_removed: bool,
    pub residue_count: usize,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum CampaignObservationError {
    #[error("campaign authorization is absent or invalid")]
    CampaignUnauthorized,
    #[error("campaign fault hooks are not compiled into this binary")]
    FaultHooksUnavailable,
    #[error("campaign fault hook was requested without authorization")]
    FaultUnauthorized,
    #[error("fault injected at {0:?}")]
    InjectedCrash(CampaignFaultPoint),
    #[error("writer fencing epoch is stale")]
    StaleEpoch,
    #[error("scheduler lease epoch is stale")]
    StaleLease,
    #[error("another restart worker owns this run")]
    DuplicateRestartWorker,
    #[error("external outcome remains indeterminate")]
    Indeterminate,
    #[error("external receipt does not match the persisted request or post-state")]
    ReceiptMismatch,
    #[error("external mutation count exceeded one")]
    DuplicateMutation,
    #[error("duplicate Effect or original idempotency key was rejected")]
    DuplicateEffect,
    #[error("cleanup found campaign-owned residue")]
    CleanupResidue,
    #[error("campaign observation infrastructure failed: {0}")]
    Infrastructure(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixtureMutationRecord {
    idempotency_key_digest: String,
    parameters_digest: String,
    version: i64,
    value: i64,
    post_state_digest: String,
    mutation_count: u64,
    receipt_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixtureState {
    schema: String,
    version: i64,
    value: i64,
    records: BTreeMap<String, FixtureMutationRecord>,
    mutation_count: u64,
    query_count: u64,
    last_query_key_digest: Option<String>,
}

impl FixtureState {
    fn empty() -> Self {
        Self {
            schema: FIXTURE_STATE_SCHEMA.to_owned(),
            version: 0,
            value: 0,
            records: BTreeMap::new(),
            mutation_count: 0,
            query_count: 0,
            last_query_key_digest: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FixtureMutationBody {
    expected_version: i64,
    delta: i64,
    parameters_digest: String,
}

struct FixtureRuntime {
    root: PathBuf,
    bounds: FixtureBounds,
    state: Mutex<FixtureState>,
    query_fault: Mutex<FixtureQueryFault>,
    stop: AtomicBool,
}

pub(crate) struct CampaignExternalStateFixture {
    root: PathBuf,
    endpoint: String,
    runtime: Arc<FixtureRuntime>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl CampaignExternalStateFixture {
    pub(crate) fn open(
        root: &Path,
        bounds: FixtureBounds,
    ) -> Result<Self, CampaignObservationError> {
        if bounds.maximum_records == 0 || bounds.maximum_absolute_delta <= 0 {
            return Err(CampaignObservationError::Infrastructure(
                "fixture bounds must be positive".to_owned(),
            ));
        }
        ensure_safe_directory(root)?;
        let state_path = root.join("state.json");
        let state = if state_path.exists() {
            read_json(&state_path)?
        } else {
            let state = FixtureState::empty();
            write_json_durable(&state_path, &state)?;
            state
        };
        if state.schema != FIXTURE_STATE_SCHEMA
            || state.records.len() > bounds.maximum_records
            || state.mutation_count > bounds.maximum_records as u64
        {
            return Err(CampaignObservationError::Infrastructure(
                "fixture state violates its frozen bounds".to_owned(),
            ));
        }
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|error| infrastructure("bind A7 fixture", error))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| infrastructure("configure A7 fixture", error))?;
        let address = listener
            .local_addr()
            .map_err(|error| infrastructure("read A7 fixture address", error))?;
        let runtime = Arc::new(FixtureRuntime {
            root: root.to_path_buf(),
            bounds,
            state: Mutex::new(state),
            query_fault: Mutex::new(FixtureQueryFault::Normal),
            stop: AtomicBool::new(false),
        });
        let server_runtime = Arc::clone(&runtime);
        let thread = std::thread::spawn(move || serve_fixture(listener, &server_runtime));
        Ok(Self {
            root: root.to_path_buf(),
            endpoint: format!("http://127.0.0.1:{}", address.port()),
            runtime,
            thread: Mutex::new(Some(thread)),
        })
    }

    pub(crate) fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub(crate) fn mutation_count(&self) -> Result<u64, CampaignObservationError> {
        Ok(lock(&self.runtime.state, "fixture state")?.mutation_count)
    }

    pub(crate) fn query_count(&self) -> Result<u64, CampaignObservationError> {
        Ok(lock(&self.runtime.state, "fixture state")?.query_count)
    }

    pub(crate) fn last_query_key_digest(&self) -> Result<Option<String>, CampaignObservationError> {
        Ok(lock(&self.runtime.state, "fixture state")?
            .last_query_key_digest
            .clone())
    }

    pub(crate) fn set_query_fault(
        &self,
        fault: FixtureQueryFault,
    ) -> Result<(), CampaignObservationError> {
        *lock(&self.runtime.query_fault, "fixture query fault")? = fault;
        Ok(())
    }

    pub(crate) fn reset(
        &self,
        maximum_existing_records: usize,
    ) -> Result<(), CampaignObservationError> {
        let mut state = lock(&self.runtime.state, "fixture state")?;
        if state.records.len() > maximum_existing_records
            || state.records.len() > self.runtime.bounds.maximum_records
        {
            return Err(CampaignObservationError::CleanupResidue);
        }
        let reset = FixtureState::empty();
        write_json_durable(&self.root.join("state.json"), &reset)?;
        *state = reset;
        *lock(&self.runtime.query_fault, "fixture query fault")? = FixtureQueryFault::Normal;
        Ok(())
    }

    /// 对已记录的原键再发一次相同参数，fixture 必须幂等返回且不增加 mutation_count。
    pub(crate) fn replay_first_recorded_key(&self) -> Result<u16, CampaignObservationError> {
        let (key, parameters_digest) = {
            let state = lock(&self.runtime.state, "fixture state")?;
            let (key, record) = state.records.iter().next().ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "fixture has no recorded key to replay".to_owned(),
                )
            })?;
            (key.clone(), record.parameters_digest.clone())
        };
        let payload = FixtureMutationBody {
            expected_version: 0,
            delta: 1,
            parameters_digest,
        };
        let (status, _) = mutate_fixture(&self.runtime, &key, &payload)?;
        Ok(status)
    }

    /// 用冲突参数重放原键，fixture 必须 409 且不写入第二次变更。
    pub(crate) fn conflict_first_recorded_key(&self) -> Result<u16, CampaignObservationError> {
        let key = {
            let state = lock(&self.runtime.state, "fixture state")?;
            state.records.keys().next().cloned().ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "fixture has no recorded key to conflict".to_owned(),
                )
            })?
        };
        let payload = FixtureMutationBody {
            expected_version: 0,
            delta: 1,
            parameters_digest: "sha256:conflict-parameters".to_owned(),
        };
        let (status, _) = mutate_fixture(&self.runtime, &key, &payload)?;
        Ok(status)
    }

    pub(crate) fn apply_bounded_mutation(
        &self,
        key: &str,
        expected_version: i64,
        delta: i64,
        parameters_digest: &str,
    ) -> Result<u16, CampaignObservationError> {
        let payload = FixtureMutationBody {
            expected_version,
            delta,
            parameters_digest: parameters_digest.to_owned(),
        };
        let (status, _) = mutate_fixture(&self.runtime, key, &payload)?;
        Ok(status)
    }

    pub(crate) fn cleanup(&self) -> Result<(), CampaignObservationError> {
        self.stop_server()?;
        if !self.root.exists() {
            return Ok(());
        }
        let residues = fs::read_dir(&self.root)
            .map_err(|error| infrastructure("enumerate fixture cleanup", error))?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name() != "state.json")
            .count();
        if residues != 0 {
            return Err(CampaignObservationError::CleanupResidue);
        }
        let state_path = self.root.join("state.json");
        if state_path.exists() {
            fs::remove_file(&state_path)
                .map_err(|error| infrastructure("remove fixture state", error))?;
        }
        fs::remove_dir(&self.root).map_err(|error| infrastructure("remove fixture root", error))
    }

    fn stop_server(&self) -> Result<(), CampaignObservationError> {
        self.runtime.stop.store(true, Ordering::SeqCst);
        if let Ok(address) = endpoint_address(&self.endpoint) {
            let _ = TcpStream::connect(address);
        }
        if let Some(thread) = lock(&self.thread, "fixture thread")?.take() {
            thread.join().map_err(|_| {
                CampaignObservationError::Infrastructure(
                    "fixture server thread panicked".to_owned(),
                )
            })?;
        }
        Ok(())
    }
}

impl Drop for CampaignExternalStateFixture {
    fn drop(&mut self) {
        let _ = self.stop_server();
    }
}

fn serve_fixture(listener: TcpListener, runtime: &Arc<FixtureRuntime>) {
    while !runtime.stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = handle_fixture_connection(stream, runtime);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
    }
}

fn handle_fixture_connection(
    mut stream: TcpStream,
    runtime: &Arc<FixtureRuntime>,
) -> Result<(), CampaignObservationError> {
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|error| infrastructure("configure fixture read timeout", error))?;
    let request = read_http_message(&mut stream)?;
    let (request_line, headers, body) = split_http_request(&request)?;
    if request_line == "POST /v1/mutations HTTP/1.1" {
        let key = headers
            .get("idempotency-key")
            .filter(|key| valid_idempotency_key(key))
            .ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "fixture mutation requires a bounded idempotency key".to_owned(),
                )
            })?;
        let payload: FixtureMutationBody = serde_json::from_slice(body).map_err(|error| {
            CampaignObservationError::Infrastructure(format!(
                "decode fixture mutation payload: {error}"
            ))
        })?;
        let (status, response) = mutate_fixture(runtime, key, &payload)?;
        return write_http_json(&mut stream, status, &response);
    }
    if let Some(key) = request_line
        .strip_prefix("GET /v1/mutations/")
        .and_then(|value| value.strip_suffix(" HTTP/1.1"))
        .filter(|key| valid_idempotency_key(key))
    {
        let fault = *lock(&runtime.query_fault, "fixture query fault")?;
        if fault == FixtureQueryFault::Timeout {
            std::thread::sleep(Duration::from_millis(700));
            return Ok(());
        }
        if fault == FixtureQueryFault::Ambiguous {
            return write_http_json(&mut stream, 503, &json!({"code": "ambiguous"}));
        }
        let (status, response) = query_fixture(runtime, key, fault)?;
        return write_http_json(&mut stream, status, &response);
    }
    write_http_json(&mut stream, 404, &json!({"code": "route_not_found"}))
}

fn mutate_fixture(
    runtime: &FixtureRuntime,
    key: &str,
    payload: &FixtureMutationBody,
) -> Result<(u16, serde_json::Value), CampaignObservationError> {
    let mut state = lock(&runtime.state, "fixture state")?;
    if let Some(existing) = state.records.get(key) {
        if existing.parameters_digest != payload.parameters_digest {
            return Ok((409, json!({"code": "idempotency_conflict"})));
        }
        return Ok((200, serde_json::to_value(existing).map_err(json_error)?));
    }
    if state.records.len() >= runtime.bounds.maximum_records {
        return Ok((409, json!({"code": "record_bound"})));
    }
    if payload.expected_version != state.version {
        return Ok((409, json!({"code": "expected_version_mismatch"})));
    }
    if payload.delta == 0 || payload.delta.abs() > runtime.bounds.maximum_absolute_delta {
        return Ok((409, json!({"code": "delta_out_of_bounds"})));
    }
    let mut next = state.clone();
    next.version += 1;
    next.value = next.value.checked_add(payload.delta).ok_or_else(|| {
        CampaignObservationError::Infrastructure("fixture value overflow".to_owned())
    })?;
    next.mutation_count += 1;
    let key_digest = digest_text(key, "personal-a7-idempotency-key/0.1")?;
    let record = FixtureMutationRecord {
        idempotency_key_digest: key_digest.clone(),
        parameters_digest: payload.parameters_digest.clone(),
        version: next.version,
        value: next.value,
        post_state_digest: post_state_digest(next.version, next.value)?,
        mutation_count: 1,
        receipt_ref: format!("fixture-receipt://{key_digest}"),
    };
    next.records.insert(key.to_owned(), record.clone());
    write_json_durable(&runtime.root.join("state.json"), &next)?;
    *state = next;
    Ok((201, serde_json::to_value(record).map_err(json_error)?))
}

fn query_fixture(
    runtime: &FixtureRuntime,
    key: &str,
    fault: FixtureQueryFault,
) -> Result<(u16, serde_json::Value), CampaignObservationError> {
    let mut state = lock(&runtime.state, "fixture state")?;
    let mut next = state.clone();
    next.query_count += 1;
    next.last_query_key_digest = Some(digest_text(key, "personal-a7-idempotency-key/0.1")?);
    let mut record = next.records.get(key).cloned();
    if let Some(record) = record.as_mut() {
        match fault {
            FixtureQueryFault::TamperedPostStateDigest => {
                record.post_state_digest = format!("sha256:{}", "0".repeat(64));
            }
            FixtureQueryFault::DuplicateMutationCount => {
                record.mutation_count = 2;
            }
            FixtureQueryFault::Normal
            | FixtureQueryFault::Timeout
            | FixtureQueryFault::Ambiguous => {}
        }
    }
    write_json_durable(&runtime.root.join("state.json"), &next)?;
    *state = next;
    match record {
        Some(record) => Ok((200, serde_json::to_value(record).map_err(json_error)?)),
        None => Ok((404, json!({"code": "not_found"}))),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunState {
    schema: String,
    campaign_id: String,
    case_ref: String,
    run_ref: String,
    requested_task_ref: String,
    authority_task_ref: String,
    effect_object_id: ObjectId,
    intent_object_id: ObjectId,
    loop_object_id: ObjectId,
    budget_id: BudgetId,
    contract_id: ObjectId,
    idempotency_key_digest: String,
    parameters_digest: String,
    fixture_endpoint: String,
    expected_fixture_version: i64,
    delta: i64,
    writer_fencing_epoch: i64,
    scheduler_lease_epoch: i64,
    dispatch_count: u64,
    stage: RunStage,
    stage_timings: Vec<CampaignStageTiming>,
    receipt_ref: Option<String>,
    post_state_digest: Option<String>,
    mutation_count: u64,
    verification_report_ref: Option<String>,
    cleanup: CampaignCleanupObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RunStage {
    Prepared,
    Authorized,
    DispatchStarted,
    MutationSucceededReceiptUnpersisted,
    ReceiptPersisted,
    Reconciled,
    Indeterminate,
    Verified,
}

/// Daemon 私有 A7 观察服务：先持久化 Intent/Effect，再派发；重启只查询原键。
pub(crate) struct CampaignMutationObservationService {
    runs_root: PathBuf,
    store: Arc<SqliteAuthorityStore>,
    artifact_store: Arc<ArtifactStore>,
    fixture_endpoint: String,
    authorization: CampaignAuthorization,
    writer_fencing_epoch: i64,
}

impl CampaignMutationObservationService {
    pub(crate) fn open(
        authority_root: &Path,
        fixture_endpoint: String,
        authorization: CampaignAuthorization,
        writer_fencing_epoch: i64,
    ) -> Result<Self, CampaignObservationError> {
        authorization.verify()?;
        validate_fixture_endpoint(&fixture_endpoint)?;
        ensure_safe_directory(authority_root)?;
        let runs_root = authority_root.join("runs");
        ensure_safe_directory(&runs_root)?;
        let store = Arc::new(
            SqliteAuthorityStore::open(&authority_root.join("authority.sqlite"))
                .map_err(|error| infrastructure("open campaign authority store", error))?,
        );
        let mut current_epoch = store
            .current_fencing_epoch()
            .map_err(|error| infrastructure("read campaign fencing epoch", error))?;
        if current_epoch > writer_fencing_epoch {
            return Err(CampaignObservationError::StaleEpoch);
        }
        while current_epoch < writer_fencing_epoch {
            current_epoch = store
                .advance_fencing_epoch()
                .map_err(|error| infrastructure("advance campaign fencing epoch", error))?;
        }
        let artifact_store = Arc::new(
            ArtifactStore::open(authority_root.join("artifacts"), ARTIFACT_MAXIMUM_BYTES)
                .map_err(|error| infrastructure("open campaign ArtifactStore", error))?,
        );
        Ok(Self {
            runs_root,
            store,
            artifact_store,
            fixture_endpoint,
            authorization,
            writer_fencing_epoch,
        })
    }

    pub(crate) fn persist_effect_before_dispatch(
        &self,
        request: CampaignMutationRequest,
    ) -> Result<PreparedCampaignMutation, CampaignObservationError> {
        self.authorization.verify()?;
        if request.task_ref.trim().is_empty()
            || request.expected_fixture_version < 0
            || request.delta == 0
            || request.delta.abs() > 10
            || request.scheduler_lease_epoch <= 0
        {
            return Err(CampaignObservationError::Infrastructure(
                "campaign mutation request violates fixed bounds".to_owned(),
            ));
        }
        self.verify_current_epoch()?;
        let started = Instant::now();
        let identifiers = UuidV7Generator;
        let run_id = identifiers
            .next_uuid_v7()
            .map_err(|error| infrastructure("mint campaign run id", error))?;
        let run_ref = format!("campaign-run://{run_id}");
        let authority_task_ref = format!("{}#campaign-run={run_id}", request.task_ref);
        let effect_object_id = next_object_id(&identifiers)?;
        let intent_object_id = next_object_id(&identifiers)?;
        let loop_object_id = next_object_id(&identifiers)?;
        let budget_id = next_budget_id(&identifiers)?;
        let contract_id = next_object_id(&identifiers)?;
        let idempotency_key = format!(
            "a7-{}",
            identifiers
                .next_uuid_v7()
                .map_err(|error| infrastructure("mint idempotency key", error))?
        );
        let mutation_document = json!({
            "delta": request.delta,
            "expected_version": request.expected_fixture_version,
            "operation": "increment",
        });
        let mutation_bytes = serde_json::to_vec(&mutation_document).map_err(json_error)?;
        let parameters_digest = canonical::digest(&mutation_bytes, MUTATION_DIGEST_DOMAIN)
            .map_err(|error| infrastructure("digest campaign mutation", error))?;
        let idempotency_key_digest =
            digest_text(&idempotency_key, "personal-a7-idempotency-key/0.1")?;
        let clock = SystemClock;
        let admitted_at = clock
            .now()
            .map_err(|error| infrastructure("read campaign clock", error))?;
        let (contract_row, budget_state_json) = build_campaign_contract(
            &identifiers,
            &authority_task_ref,
            contract_id.clone(),
            loop_object_id.clone(),
            budget_id.clone(),
            &admitted_at,
        )?;
        self.store
            .insert_task_contract(
                &contract_row,
                &event(
                    &identifiers,
                    &contract_id,
                    LifecycleDomain::Task,
                    "task-contract.minted",
                    &admitted_at,
                )?,
                0,
            )
            .map_err(|error| infrastructure("persist campaign TaskContract", error))?;
        self.store
            .create_budget(&budget_id, &budget_state_json, &admitted_at)
            .map_err(|error| infrastructure("persist campaign budget", error))?;
        self.store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    state: StateName::parse("ACT")
                        .map_err(|error| infrastructure("parse Loop state", error))?,
                    version: Version::INITIAL,
                    body: json!({"campaign_id": CAMPAIGN_ID, "run_ref": run_ref}),
                },
                admitted_at: admitted_at.clone(),
                event: event(
                    &identifiers,
                    &loop_object_id,
                    LifecycleDomain::Loop,
                    "campaign-loop.admitted",
                    &admitted_at,
                )?,
                outbox: Vec::new(),
                fencing_epoch: Some(self.writer_fencing_epoch),
            })
            .map_err(|error| infrastructure("persist campaign Loop", error))?;
        self.store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: effect_object_id.clone(),
                    domain: LifecycleDomain::Effect,
                    state: StateName::parse("PROPOSED")
                        .map_err(|error| infrastructure("parse Effect state", error))?,
                    version: Version::INITIAL,
                    body: json!({
                        "campaign_id": CAMPAIGN_ID,
                        "case_ref": self.authorization.case_ref,
                        "expected_fixture_version": request.expected_fixture_version,
                        "delta": request.delta,
                        "idempotency_key_digest": idempotency_key_digest,
                        "parameters_digest": parameters_digest,
                    }),
                },
                admitted_at: admitted_at.clone(),
                event: event(
                    &identifiers,
                    &effect_object_id,
                    LifecycleDomain::Effect,
                    "campaign-effect.admitted",
                    &admitted_at,
                )?,
                outbox: Vec::new(),
                fencing_epoch: Some(self.writer_fencing_epoch),
            })
            .map_err(|error| infrastructure("persist campaign Effect", error))?;
        self.store
            .insert_intent(
                &IntentRow {
                    intent_id: intent_object_id.clone(),
                    idempotency_key,
                    parameters_digest: parameters_digest.clone(),
                    action: "external.mutate".to_owned(),
                    target: format!("{}/v1/mutations", self.fixture_endpoint),
                    effect_object_id: effect_object_id.clone(),
                    expected_state_version: Version::INITIAL,
                    grant_epoch: 1,
                    capability_set_version: 1,
                    task_binding: Some(TaskBinding {
                        task_ref: authority_task_ref.clone(),
                        contract_epoch: 1,
                    }),
                    canonical_json: json!({
                        "campaign_id": CAMPAIGN_ID,
                        "case_ref": self.authorization.case_ref,
                        "effect_object_id": effect_object_id.as_str(),
                        "parameters_digest": parameters_digest,
                    })
                    .to_string(),
                },
                &event(
                    &identifiers,
                    &intent_object_id,
                    LifecycleDomain::Effect,
                    "campaign-intent.persisted",
                    &admitted_at,
                )?,
            )
            .map_err(duplicate_or_infrastructure("persist campaign Intent"))?;
        let state = RunState {
            schema: RUN_STATE_SCHEMA.to_owned(),
            campaign_id: CAMPAIGN_ID.to_owned(),
            case_ref: self.authorization.case_ref.clone(),
            run_ref: run_ref.clone(),
            requested_task_ref: request.task_ref,
            authority_task_ref,
            effect_object_id: effect_object_id.clone(),
            intent_object_id,
            loop_object_id,
            budget_id,
            contract_id,
            idempotency_key_digest: idempotency_key_digest.clone(),
            parameters_digest,
            fixture_endpoint: self.fixture_endpoint.clone(),
            expected_fixture_version: request.expected_fixture_version,
            delta: request.delta,
            writer_fencing_epoch: self.writer_fencing_epoch,
            scheduler_lease_epoch: request.scheduler_lease_epoch,
            dispatch_count: 0,
            stage: RunStage::Prepared,
            stage_timings: vec![stage_timing("persist_effect_before_dispatch", started)],
            receipt_ref: None,
            post_state_digest: None,
            mutation_count: 0,
            verification_report_ref: None,
            cleanup: CampaignCleanupObservation {
                fixture_removed: false,
                residue_count: 0,
            },
        };
        self.write_run_state(&state)?;
        Ok(PreparedCampaignMutation {
            run_ref,
            effect_ref: effect_ref(&effect_object_id),
            idempotency_key_digest,
        })
    }

    pub(crate) fn dispatch(
        &self,
        run_ref: &str,
        fault: CampaignFaultPoint,
        scheduler_lease_epoch: i64,
    ) -> Result<CampaignMutationObservation, CampaignObservationError> {
        if !self.authorization.faults_enabled {
            return Err(CampaignObservationError::FaultUnauthorized);
        }
        self.dispatch_internal(run_ref, Some(fault), scheduler_lease_epoch)
    }

    pub(crate) fn dispatch_without_fault(
        &self,
        run_ref: &str,
        scheduler_lease_epoch: i64,
    ) -> Result<CampaignMutationObservation, CampaignObservationError> {
        self.dispatch_internal(run_ref, None, scheduler_lease_epoch)
    }

    fn dispatch_internal(
        &self,
        run_ref: &str,
        fault: Option<CampaignFaultPoint>,
        scheduler_lease_epoch: i64,
    ) -> Result<CampaignMutationObservation, CampaignObservationError> {
        self.authorization.verify()?;
        self.verify_current_epoch()?;
        let _run_guard = ActiveRunGuard::acquire(run_ref)?;
        let mut state = self.read_run_state(run_ref)?;
        self.verify_run_binding(&state, scheduler_lease_epoch)?;
        if state.stage != RunStage::Prepared {
            return Err(CampaignObservationError::Infrastructure(
                "campaign run is not dispatchable".to_owned(),
            ));
        }
        let intent = self
            .store
            .load_intent_for_effect(&state.effect_object_id)
            .map_err(|error| infrastructure("reload original campaign Intent", error))?
            .ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "campaign Effect has no original Intent".to_owned(),
                )
            })?;
        if digest_text(&intent.idempotency_key, "personal-a7-idempotency-key/0.1")?
            != state.idempotency_key_digest
            || intent.parameters_digest != state.parameters_digest
        {
            return Err(CampaignObservationError::ReceiptMismatch);
        }
        let clock = SystemClock;
        let identifiers = UuidV7Generator;
        let lease = WriterLease {
            epoch: self.writer_fencing_epoch,
        };
        let protocol = effect_protocol(&self.store, &clock, &identifiers, run_ref)?;
        let grant = effect_grant(&clock)?;
        let currency = GovernanceCurrency {
            revocation_epoch: 1,
            capability_set_version: 1,
        };
        let authorized = protocol
            .authorize_effect(
                &state.effect_object_id,
                Version::INITIAL,
                &grant,
                &currency,
                &lease,
            )
            .map_err(|error| infrastructure("authorize campaign Effect", error))?;
        state.stage = RunStage::Authorized;
        state
            .stage_timings
            .push(stage_timing("effect_authorized", Instant::now()));
        self.write_run_state(&state)?;
        if fault == Some(CampaignFaultPoint::DispatchBefore) {
            return Err(CampaignObservationError::InjectedCrash(
                CampaignFaultPoint::DispatchBefore,
            ));
        }
        state.dispatch_count = state.dispatch_count.checked_add(1).ok_or_else(|| {
            CampaignObservationError::Infrastructure("dispatch counter overflow".to_owned())
        })?;
        if state.dispatch_count != 1 {
            return Err(CampaignObservationError::DuplicateMutation);
        }
        state.stage = RunStage::DispatchStarted;
        self.write_run_state(&state)?;
        let executor = FixtureEffectExecutor::new(
            state.fixture_endpoint.clone(),
            self.writer_fencing_epoch,
            intent.idempotency_key.clone(),
            state.parameters_digest.clone(),
            state.expected_fixture_version,
            state.delta,
        );
        let (executing, outcome) = protocol
            .dispatch_effect(
                &state.effect_object_id,
                authorized.after_version,
                &grant,
                &currency,
                &executor,
                &lease,
            )
            .map_err(|error| infrastructure("dispatch campaign Effect", error))?;
        if fault == Some(CampaignFaultPoint::MutationAfterReceiptBefore)
            && matches!(outcome, DispatchOutcome::Executed { .. })
        {
            state.stage = RunStage::MutationSucceededReceiptUnpersisted;
            state.stage_timings.push(stage_timing(
                "mutation_succeeded_receipt_unpersisted",
                Instant::now(),
            ));
            self.write_run_state(&state)?;
            return Err(CampaignObservationError::InjectedCrash(
                CampaignFaultPoint::MutationAfterReceiptBefore,
            ));
        }
        let recorded = protocol
            .record_outcome(
                &state.effect_object_id,
                executing.after_version,
                &outcome,
                &lease,
            )
            .map_err(|error| infrastructure("record campaign Effect outcome", error))?;
        match outcome {
            DispatchOutcome::Executed { receipt_ref } => {
                state.receipt_ref = Some(receipt_ref);
                state.stage = RunStage::ReceiptPersisted;
                self.write_run_state(&state)?;
                if fault == Some(CampaignFaultPoint::ReceiptAfterEffectCloseBefore) {
                    return Err(CampaignObservationError::InjectedCrash(
                        CampaignFaultPoint::ReceiptAfterEffectCloseBefore,
                    ));
                }
                let (_, query) = protocol
                    .reconcile(
                        &state.effect_object_id,
                        "EXECUTED",
                        recorded.after_version,
                        &executor,
                        &lease,
                    )
                    .map_err(|error| infrastructure("reconcile executed campaign Effect", error))?;
                if query != ExecutorQueryResult::ExecutedWithOriginalKey {
                    return Err(CampaignObservationError::ReceiptMismatch);
                }
                state.stage = RunStage::Reconciled;
                self.write_run_state(&state)?;
            }
            DispatchOutcome::Unknown { .. } => {
                state.stage = RunStage::MutationSucceededReceiptUnpersisted;
                self.write_run_state(&state)?;
                return Err(CampaignObservationError::Indeterminate);
            }
            DispatchOutcome::NotExecuted { .. } => {
                state.stage = RunStage::Indeterminate;
                self.write_run_state(&state)?;
                return Ok(self.observation_from_state(&state, CampaignOutcomeClass::NotExecuted));
            }
            DispatchOutcome::FencedStaleEpoch { .. } => {
                return Err(CampaignObservationError::StaleEpoch);
            }
        }
        let record = executor
            .last_record()?
            .ok_or(CampaignObservationError::ReceiptMismatch)?;
        self.validate_fixture_record(&state, &record)?;
        state.post_state_digest = Some(record.post_state_digest);
        state.mutation_count = record.mutation_count;
        if fault == Some(CampaignFaultPoint::VerificationBefore) {
            self.write_run_state(&state)?;
            return Err(CampaignObservationError::InjectedCrash(
                CampaignFaultPoint::VerificationBefore,
            ));
        }
        self.verify_reconciled_run(&mut state)?;
        self.write_run_state(&state)?;
        Ok(self.observation_from_state(&state, CampaignOutcomeClass::ReconciledExecuted))
    }

    pub(crate) fn reconcile_after_restart(
        &self,
        run_ref: &str,
        scheduler_lease_epoch: i64,
    ) -> Result<CampaignMutationObservation, CampaignObservationError> {
        self.authorization.verify()?;
        self.verify_current_epoch()?;
        let _run_guard = ActiveRunGuard::acquire(run_ref)?;
        let mut state = self.read_run_state(run_ref)?;
        self.verify_run_binding(&state, scheduler_lease_epoch)?;
        let intent = self
            .store
            .load_intent_for_effect(&state.effect_object_id)
            .map_err(|error| infrastructure("reload restart Intent", error))?
            .ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "restart has no original Intent".to_owned(),
                )
            })?;
        if digest_text(&intent.idempotency_key, "personal-a7-idempotency-key/0.1")?
            != state.idempotency_key_digest
        {
            return Err(CampaignObservationError::ReceiptMismatch);
        }
        let executor = FixtureEffectExecutor::new(
            state.fixture_endpoint.clone(),
            self.writer_fencing_epoch,
            intent.idempotency_key,
            state.parameters_digest.clone(),
            state.expected_fixture_version,
            state.delta,
        );
        let effect = self
            .store
            .load_object(LifecycleDomain::Effect, &state.effect_object_id)
            .map_err(|error| infrastructure("reload restart Effect", error))?
            .ok_or_else(|| {
                CampaignObservationError::Infrastructure("restart Effect is unavailable".to_owned())
            })?;
        if matches!(effect.state.as_str(), "PROPOSED" | "AUTHORIZED") {
            state.stage = RunStage::Indeterminate;
            self.write_run_state(&state)?;
            return Ok(self.observation_from_state(&state, CampaignOutcomeClass::Indeterminate));
        }
        let clock = SystemClock;
        let identifiers = UuidV7Generator;
        let lease = WriterLease {
            epoch: self.writer_fencing_epoch,
        };
        let protocol = effect_protocol(&self.store, &clock, &identifiers, run_ref)?;
        let query = match effect.state.as_str() {
            "EXECUTING" => {
                let unknown = protocol
                    .record_outcome(
                        &state.effect_object_id,
                        effect.version,
                        &DispatchOutcome::Unknown {
                            detail: "daemon restarted after external dispatch".to_owned(),
                        },
                        &lease,
                    )
                    .map_err(|error| infrastructure("record restart uncertainty", error))?;
                protocol
                    .reconcile(
                        &state.effect_object_id,
                        "OUTCOME_UNKNOWN",
                        unknown.after_version,
                        &executor,
                        &lease,
                    )
                    .map_err(|error| infrastructure("query original restart key", error))?
                    .1
            }
            "OUTCOME_UNKNOWN" => {
                protocol
                    .reconcile(
                        &state.effect_object_id,
                        "OUTCOME_UNKNOWN",
                        effect.version,
                        &executor,
                        &lease,
                    )
                    .map_err(|error| infrastructure("query original unknown key", error))?
                    .1
            }
            "EXECUTED" => {
                let query = executor.query_outcome_for_service()?;
                if query == ExecutorQueryResult::ExecutedWithOriginalKey {
                    protocol
                        .reconcile(
                            &state.effect_object_id,
                            "EXECUTED",
                            effect.version,
                            &executor,
                            &lease,
                        )
                        .map_err(|error| infrastructure("close receipt-confirmed Effect", error))?;
                }
                query
            }
            "RECONCILED" => executor.query_outcome_for_service()?,
            "QUARANTINED" | "NOT_EXECUTED" => ExecutorQueryResult::Indeterminate,
            "VERIFIED" | "VERIFY_FAILED" => ExecutorQueryResult::ExecutedWithOriginalKey,
            other => {
                return Err(CampaignObservationError::Infrastructure(format!(
                    "unsupported restart Effect state {other}"
                )));
            }
        };
        match query {
            ExecutorQueryResult::ExecutedWithOriginalKey => {
                let record = executor
                    .last_record()?
                    .ok_or(CampaignObservationError::ReceiptMismatch)?;
                self.validate_fixture_record(&state, &record)?;
                state.receipt_ref = Some(record.receipt_ref);
                state.post_state_digest = Some(record.post_state_digest);
                state.mutation_count = record.mutation_count;
                state.stage = RunStage::Reconciled;
                state.stage_timings.push(stage_timing(
                    "restart_original_key_reconciled",
                    Instant::now(),
                ));
                self.write_run_state(&state)?;
                self.verify_reconciled_run(&mut state)?;
                self.write_run_state(&state)?;
                Ok(self.observation_from_state(&state, CampaignOutcomeClass::ReconciledExecuted))
            }
            ExecutorQueryResult::NotExecuted => {
                let effect = self
                    .store
                    .load_object(LifecycleDomain::Effect, &state.effect_object_id)
                    .map_err(|error| infrastructure("reload not-executed Effect", error))?
                    .ok_or_else(|| {
                        CampaignObservationError::Infrastructure(
                            "not-executed Effect disappeared".to_owned(),
                        )
                    })?;
                if effect.state.as_str() == "RECONCILED" {
                    protocol
                        .close_not_executed(&state.effect_object_id, effect.version, &lease)
                        .map_err(|error| infrastructure("close non-executed Effect", error))?;
                }
                state.stage = RunStage::Indeterminate;
                self.write_run_state(&state)?;
                Ok(self.observation_from_state(&state, CampaignOutcomeClass::NotExecuted))
            }
            ExecutorQueryResult::Indeterminate => {
                let effect = self
                    .store
                    .load_object(LifecycleDomain::Effect, &state.effect_object_id)
                    .map_err(|error| infrastructure("reload indeterminate Effect", error))?
                    .ok_or_else(|| {
                        CampaignObservationError::Infrastructure(
                            "indeterminate Effect disappeared".to_owned(),
                        )
                    })?;
                if effect.state.as_str() == "RECONCILED" {
                    protocol
                        .quarantine_still_unknown(&state.effect_object_id, effect.version, &lease)
                        .map_err(|error| infrastructure("quarantine unknown Effect", error))?;
                }
                state.stage = RunStage::Indeterminate;
                self.write_run_state(&state)?;
                Ok(self.observation_from_state(&state, CampaignOutcomeClass::Indeterminate))
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn observation(
        &self,
        run_ref: &str,
    ) -> Result<CampaignMutationObservation, CampaignObservationError> {
        let state = self.read_run_state(run_ref)?;
        let outcome = match state.stage {
            RunStage::Verified => CampaignOutcomeClass::ReconciledExecuted,
            RunStage::Indeterminate => CampaignOutcomeClass::Indeterminate,
            RunStage::Prepared | RunStage::Authorized | RunStage::DispatchStarted => {
                CampaignOutcomeClass::Prepared
            }
            RunStage::MutationSucceededReceiptUnpersisted
            | RunStage::ReceiptPersisted
            | RunStage::Reconciled => CampaignOutcomeClass::Indeterminate,
        };
        Ok(self.observation_from_state(&state, outcome))
    }

    pub(crate) fn dispatch_count(&self, run_ref: &str) -> Result<u64, CampaignObservationError> {
        Ok(self.read_run_state(run_ref)?.dispatch_count)
    }

    /// 用原键再插入一条 Intent：SQLite UNIQUE 必须拒绝，禁止换键或双 Effect。
    pub(crate) fn reject_duplicate_original_key_intent(
        &self,
        run_ref: &str,
    ) -> Result<(), CampaignObservationError> {
        self.authorization.verify()?;
        let state = self.read_run_state(run_ref)?;
        let intent = self
            .store
            .load_intent_for_effect(&state.effect_object_id)
            .map_err(|error| infrastructure("reload original Intent for duplicate", error))?
            .ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "duplicate probe has no original Intent".to_owned(),
                )
            })?;
        let identifiers = UuidV7Generator;
        let clock = SystemClock;
        let recorded_at = clock
            .now()
            .map_err(|error| infrastructure("read duplicate-intent clock", error))?;
        let mut duplicate = intent;
        duplicate.intent_id = next_object_id(&identifiers)?;
        self.store
            .insert_intent(
                &duplicate,
                &event(
                    &identifiers,
                    &duplicate.intent_id,
                    LifecycleDomain::Effect,
                    "campaign-intent.duplicate-rejected",
                    &recorded_at,
                )?,
            )
            .map_err(duplicate_or_infrastructure(
                "reject duplicate original-key Intent",
            ))?;
        Err(CampaignObservationError::Infrastructure(
            "duplicate original-key Intent was accepted".to_owned(),
        ))
    }

    /// 同一 run 的第二个 restart worker 必须在取锁时失败。
    pub(crate) fn reject_duplicate_restart_worker(
        &self,
        run_ref: &str,
    ) -> Result<(), CampaignObservationError> {
        let _held = ActiveRunGuard::acquire(run_ref)?;
        ActiveRunGuard::acquire(run_ref).map(|_| ())
    }

    fn verify_reconciled_run(&self, state: &mut RunState) -> Result<(), CampaignObservationError> {
        let effect = self
            .store
            .load_object(LifecycleDomain::Effect, &state.effect_object_id)
            .map_err(|error| infrastructure("load Effect before verification", error))?
            .ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "Effect disappeared before verification".to_owned(),
                )
            })?;
        if effect.state.as_str() != "RECONCILED" {
            return Err(CampaignObservationError::Indeterminate);
        }
        if state.mutation_count != 1 || state.post_state_digest.is_none() {
            return Err(CampaignObservationError::DuplicateMutation);
        }
        let clock = SystemClock;
        let identifiers = UuidV7Generator;
        let lease = WriterLease {
            epoch: self.writer_fencing_epoch,
        };
        let verification_request =
            crate::personal::verification_executor::begin_verification_from_current_task_contract(
                self.store.as_ref(),
                &clock,
                &identifiers,
                &TaskBinding {
                    task_ref: state.authority_task_ref.clone(),
                    contract_epoch: 1,
                },
                &state.loop_object_id,
                Version::INITIAL,
                &state.effect_object_id,
                &lease,
            )
            .map_err(|error| infrastructure("begin P2-T13 verification", error))?;
        let outcome =
            crate::personal::verification_executor::run_production_independent_verification(
                self.store.as_ref(),
                self.artifact_store.as_ref(),
                &clock,
                &identifiers,
                &verification_request.verification_request_id,
                &lease,
            )
            .map_err(|error| infrastructure("run P2-T13 verifier", error))?;
        if outcome.report.status != "passed" {
            return Err(CampaignObservationError::Indeterminate);
        }
        state.verification_report_ref = Some(format!(
            "verification-report://{}",
            outcome.report.verification_report_id.as_str()
        ));
        state.stage = RunStage::Verified;
        state.stage_timings.push(stage_timing(
            "independent_verification_passed",
            Instant::now(),
        ));
        Ok(())
    }

    fn validate_fixture_record(
        &self,
        state: &RunState,
        record: &FixtureMutationRecord,
    ) -> Result<(), CampaignObservationError> {
        if record.idempotency_key_digest != state.idempotency_key_digest
            || record.parameters_digest != state.parameters_digest
            || record.post_state_digest != post_state_digest(record.version, record.value)?
        {
            return Err(CampaignObservationError::ReceiptMismatch);
        }
        if record.mutation_count != 1 {
            return Err(CampaignObservationError::DuplicateMutation);
        }
        Ok(())
    }

    fn verify_run_binding(
        &self,
        state: &RunState,
        scheduler_lease_epoch: i64,
    ) -> Result<(), CampaignObservationError> {
        if state.schema != RUN_STATE_SCHEMA
            || state.campaign_id != self.authorization.campaign_id
            || state.case_ref != self.authorization.case_ref
            || state.fixture_endpoint != self.fixture_endpoint
            || state.writer_fencing_epoch != self.writer_fencing_epoch
        {
            return Err(CampaignObservationError::CampaignUnauthorized);
        }
        if state.scheduler_lease_epoch != scheduler_lease_epoch {
            return Err(CampaignObservationError::StaleLease);
        }
        Ok(())
    }

    fn verify_current_epoch(&self) -> Result<(), CampaignObservationError> {
        let current = self
            .store
            .current_fencing_epoch()
            .map_err(|error| infrastructure("read current writer epoch", error))?;
        if current != self.writer_fencing_epoch {
            return Err(CampaignObservationError::StaleEpoch);
        }
        Ok(())
    }

    fn observation_from_state(
        &self,
        state: &RunState,
        outcome_class: CampaignOutcomeClass,
    ) -> CampaignMutationObservation {
        CampaignMutationObservation {
            schema_version: RUN_STATE_SCHEMA,
            run_ref: state.run_ref.clone(),
            outcome_class,
            idempotency_key_digest: state.idempotency_key_digest.clone(),
            idempotency_key_ref: format!("idempotency-key://{}", state.idempotency_key_digest),
            mutation_count: state.mutation_count,
            post_state_digest: state.post_state_digest.clone(),
            stage_timings: state.stage_timings.clone(),
            effect_ref: effect_ref(&state.effect_object_id),
            verification_report_ref: state.verification_report_ref.clone(),
            acceptance_ref: None,
            cleanup: state.cleanup.clone(),
        }
    }

    fn read_run_state(&self, run_ref: &str) -> Result<RunState, CampaignObservationError> {
        read_json(&self.run_path(run_ref)?)
    }

    fn write_run_state(&self, state: &RunState) -> Result<(), CampaignObservationError> {
        write_json_durable(&self.run_path(&state.run_ref)?, state)
    }

    fn run_path(&self, run_ref: &str) -> Result<PathBuf, CampaignObservationError> {
        let run_id = run_ref
            .strip_prefix("campaign-run://")
            .filter(|value| valid_uuid_text(value))
            .ok_or_else(|| {
                CampaignObservationError::Infrastructure(
                    "invalid campaign run reference".to_owned(),
                )
            })?;
        Ok(self.runs_root.join(format!("{run_id}.json")))
    }
}

struct ActiveRunGuard {
    run_ref: String,
}

impl ActiveRunGuard {
    fn acquire(run_ref: &str) -> Result<Self, CampaignObservationError> {
        let active = ACTIVE_RUNS.get_or_init(|| Mutex::new(BTreeSet::new()));
        let mut active = lock(active, "active campaign runs")?;
        if !active.insert(run_ref.to_owned()) {
            return Err(CampaignObservationError::DuplicateRestartWorker);
        }
        Ok(Self {
            run_ref: run_ref.to_owned(),
        })
    }
}

impl Drop for ActiveRunGuard {
    fn drop(&mut self) {
        if let Some(active) = ACTIVE_RUNS.get()
            && let Ok(mut active) = active.lock()
        {
            active.remove(&self.run_ref);
        }
    }
}

struct FixtureEffectExecutor {
    endpoint: String,
    trusted_fencing_epoch: i64,
    original_idempotency_key: String,
    parameters_digest: String,
    expected_version: i64,
    delta: i64,
    last_record: Mutex<Option<FixtureMutationRecord>>,
}

impl FixtureEffectExecutor {
    fn new(
        endpoint: String,
        trusted_fencing_epoch: i64,
        original_idempotency_key: String,
        parameters_digest: String,
        expected_version: i64,
        delta: i64,
    ) -> Self {
        Self {
            endpoint,
            trusted_fencing_epoch,
            original_idempotency_key,
            parameters_digest,
            expected_version,
            delta,
            last_record: Mutex::new(None),
        }
    }

    fn query_outcome_for_service(&self) -> Result<ExecutorQueryResult, CampaignObservationError> {
        self.query_original_key()
    }

    fn query_original_key(&self) -> Result<ExecutorQueryResult, CampaignObservationError> {
        let response = match send_fixture_http(
            &self.endpoint,
            "GET",
            &format!("/v1/mutations/{}", self.original_idempotency_key),
            &[],
            &[],
        ) {
            Ok(response) => response,
            Err(_) => return Ok(ExecutorQueryResult::Indeterminate),
        };
        match response.status {
            200 => {
                let record: FixtureMutationRecord = serde_json::from_slice(&response.body)
                    .map_err(|_| CampaignObservationError::Indeterminate)?;
                *lock(&self.last_record, "fixture query record")? = Some(record);
                Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
            }
            404 => Ok(ExecutorQueryResult::NotExecuted),
            _ => Ok(ExecutorQueryResult::Indeterminate),
        }
    }

    fn last_record(&self) -> Result<Option<FixtureMutationRecord>, CampaignObservationError> {
        Ok(lock(&self.last_record, "fixture record")?.clone())
    }
}

impl EffectExecutor for FixtureEffectExecutor {
    fn capabilities(&self) -> ExecutorCapabilities {
        ExecutorCapabilities {
            queryable: true,
            idempotent: true,
        }
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        if call.fencing_epoch != self.trusted_fencing_epoch {
            return Ok(DispatchOutcome::FencedStaleEpoch {
                sink_epoch: self.trusted_fencing_epoch,
            });
        }
        if call.idempotency_key != self.original_idempotency_key
            || call.parameters_digest != self.parameters_digest
            || call.action != "external.mutate"
        {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "campaign dispatch binding mismatch".to_owned(),
            });
        }
        let body = serde_json::to_vec(&json!({
            "delta": self.delta,
            "expected_version": self.expected_version,
            "operation": "increment",
            "parameters_digest": self.parameters_digest,
        }))
        .map_err(|error| PortFailure {
            detail: error.to_string(),
        })?;
        let response = match send_fixture_http(
            &self.endpoint,
            "POST",
            "/v1/mutations",
            &[("Idempotency-Key", self.original_idempotency_key.as_str())],
            &body,
        ) {
            Ok(response) => response,
            Err(error) => {
                return Ok(DispatchOutcome::Unknown { detail: error });
            }
        };
        match response.status {
            200 | 201 => {
                let record: FixtureMutationRecord = serde_json::from_slice(&response.body)
                    .map_err(|error| PortFailure {
                        detail: format!("decode fixture receipt: {error}"),
                    })?;
                let receipt_ref = record.receipt_ref.clone();
                *self.last_record.lock().map_err(|_| PortFailure {
                    detail: "fixture receipt lock poisoned".to_owned(),
                })? = Some(record);
                Ok(DispatchOutcome::Executed { receipt_ref })
            }
            409 => Ok(DispatchOutcome::NotExecuted {
                reason: "fixture rejected mutation before execution".to_owned(),
            }),
            _ => Ok(DispatchOutcome::Unknown {
                detail: format!("fixture returned ambiguous status {}", response.status),
            }),
        }
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        if idempotency_key != self.original_idempotency_key {
            return Ok(ExecutorQueryResult::Indeterminate);
        }
        self.query_original_key().map_err(|error| PortFailure {
            detail: error.to_string(),
        })
    }
}

struct HttpResponse {
    status: u16,
    body: Vec<u8>,
}

fn send_fixture_http(
    endpoint: &str,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Result<HttpResponse, String> {
    let address = endpoint_address(endpoint).map_err(|error| error.to_string())?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(300))
        .map_err(|error| format!("connect fixture: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(300)))
        .map_err(|error| format!("configure fixture read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(300)))
        .map_err(|error| format!("configure fixture write timeout: {error}"))?;
    let mut request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: {}\r\n",
        address,
        body.len()
    );
    for (name, value) in headers {
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");
    stream
        .write_all(request.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|error| format!("write fixture request: {error}"))?;
    let _ = stream.shutdown(Shutdown::Write);
    let mut response = Vec::new();
    stream
        .take(MAXIMUM_HTTP_BYTES as u64)
        .read_to_end(&mut response)
        .map_err(|error| format!("read fixture response: {error}"))?;
    let header_end =
        find_header_end(&response).ok_or_else(|| "malformed fixture response".to_owned())?;
    let head = std::str::from_utf8(&response[..header_end])
        .map_err(|_| "fixture response headers are not utf-8".to_owned())?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or_else(|| "fixture response status is invalid".to_owned())?;
    Ok(HttpResponse {
        status,
        body: response[header_end + 4..].to_vec(),
    })
}

fn read_http_message(stream: &mut TcpStream) -> Result<Vec<u8>, CampaignObservationError> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 2048];
    let mut expected_total = None;
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|error| infrastructure("read fixture request", error))?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.len() > MAXIMUM_HTTP_BYTES {
            return Err(CampaignObservationError::Infrastructure(
                "fixture request exceeds byte bound".to_owned(),
            ));
        }
        if expected_total.is_none()
            && let Some(header_end) = find_header_end(&bytes)
        {
            let headers = std::str::from_utf8(&bytes[..header_end]).map_err(|_| {
                CampaignObservationError::Infrastructure("fixture headers are not utf-8".to_owned())
            })?;
            let content_length = parse_content_length(headers)?;
            expected_total = Some(header_end + 4 + content_length);
        }
        if expected_total.is_some_and(|total| bytes.len() >= total) {
            break;
        }
    }
    Ok(bytes)
}

fn split_http_request(
    bytes: &[u8],
) -> Result<(&str, BTreeMap<String, String>, &[u8]), CampaignObservationError> {
    let header_end = find_header_end(bytes).ok_or_else(|| {
        CampaignObservationError::Infrastructure("fixture request is incomplete".to_owned())
    })?;
    let head = std::str::from_utf8(&bytes[..header_end]).map_err(|_| {
        CampaignObservationError::Infrastructure("fixture headers are not utf-8".to_owned())
    })?;
    let mut lines = head.lines();
    let request_line = lines.next().ok_or_else(|| {
        CampaignObservationError::Infrastructure("fixture request line is absent".to_owned())
    })?;
    let mut headers = BTreeMap::new();
    for line in lines {
        let (name, value) = line.split_once(':').ok_or_else(|| {
            CampaignObservationError::Infrastructure("fixture header is malformed".to_owned())
        })?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }
    Ok((request_line, headers, &bytes[header_end + 4..]))
}

fn parse_content_length(headers: &str) -> Result<usize, CampaignObservationError> {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0)
        .try_into()
        .map_err(|_| CampaignObservationError::Infrastructure("invalid content length".to_owned()))
}

fn write_http_json(
    stream: &mut TcpStream,
    status: u16,
    value: &serde_json::Value,
) -> Result<(), CampaignObservationError> {
    let body = serde_json::to_vec(value).map_err(json_error)?;
    let reason = match status {
        200 => "OK",
        201 => "Created",
        404 => "Not Found",
        409 => "Conflict",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .and_then(|_| stream.write_all(&body))
        .map_err(|error| infrastructure("write fixture response", error))
}

fn effect_protocol<'a>(
    store: &'a SqliteAuthorityStore,
    clock: &'a SystemClock,
    identifiers: &'a UuidV7Generator,
    run_ref: &str,
) -> Result<
    EffectProtocol<'a, SqliteAuthorityStore, SystemClock, UuidV7Generator>,
    CampaignObservationError,
> {
    Ok(EffectProtocol::new(
        store,
        clock,
        identifiers,
        UriRef::parse("principal://personal/daemon")
            .map_err(|error| infrastructure("parse campaign actor", error))?,
        UriRef::parse("authority://personal/effect-authority")
            .map_err(|error| infrastructure("parse campaign authority", error))?,
        UriRef::parse(&run_ref.replacen("campaign-run://", "correlation://personal/a7/", 1))
            .map_err(|error| infrastructure("parse campaign correlation", error))?,
    ))
}

fn effect_grant(clock: &SystemClock) -> Result<AuthorizationGrant, CampaignObservationError> {
    let decided_at = clock
        .now()
        .map_err(|error| infrastructure("read authorization clock", error))?;
    authorize(
        &AuthzSnapshot {
            tenant_id: "personal".to_owned(),
            principal: PrincipalFacts {
                principal_ref: UriRef::parse("principal://personal/daemon")
                    .map_err(|error| infrastructure("parse campaign principal", error))?,
                authenticated: true,
                active: true,
                tenant_id: Some("personal".to_owned()),
            },
            actor_chain: ActorChainFacts {
                chain_digest: format!("sha256:{}", "a".repeat(64)),
                resolved: true,
            },
            membership: Some(MembershipFacts {
                valid: true,
                roles: ["daemon".to_owned()].into(),
            }),
            capability_links: vec![CapabilityConstraints {
                subject: "principal://personal/daemon".to_owned(),
                audience: "authority://personal/effect-authority".to_owned(),
                resource: "scope://personal/campaign-external-mutation".to_owned(),
                purpose: "task_execution".to_owned(),
                actions: ["external.mutate".to_owned()].into(),
                parameter_bounds: BTreeMap::new(),
                lease: LeaseWindow {
                    not_before: WallTimestamp::parse("2020-01-01T00:00:00Z")
                        .map_err(|error| infrastructure("parse campaign lease start", error))?,
                    expires: WallTimestamp::parse("2099-01-01T00:00:00Z")
                        .map_err(|error| infrastructure("parse campaign lease end", error))?,
                },
                depth_remaining: 1,
                issued_epoch: 1,
            }],
            capability_set_version: 1,
            explicit_denies: Vec::new(),
            revocation_epoch: 1,
            decided_at,
        },
        &ObjectGovernance {
            object_ref: "effect://personal/campaign-external-mutation".to_owned(),
            tenant_id: Some("personal".to_owned()),
            owner_ref: "principal://personal/daemon".to_owned(),
            resource_scope: "scope://personal/campaign-external-mutation".to_owned(),
            conversation_ref: None,
        },
        &AccessRequest {
            action: "external.mutate".to_owned(),
            purpose: "task_execution".to_owned(),
        },
    )
    .map_err(|error| {
        CampaignObservationError::Infrastructure(format!(
            "authorize campaign external mutation: {error:?}"
        ))
    })
}

fn build_campaign_contract(
    identifiers: &UuidV7Generator,
    task_ref: &str,
    contract_id: ObjectId,
    loop_object_id: ObjectId,
    budget_id: BudgetId,
    issued_at: &WallTimestamp,
) -> Result<(TaskContractRow, String), CampaignObservationError> {
    let owner_id = next_object_id(identifiers)?;
    let authority_id = next_object_id(identifiers)?;
    let scope_id = next_object_id(identifiers)?;
    let acceptance_id = next_object_id(identifiers)?;
    let interpretation_id = next_object_id(identifiers)?;
    let intent_record_id = next_object_id(identifiers)?;
    let governance = GovernanceSeed {
        owner: strong_reference_to(&owner_id, &format!("sha256:{}", "a".repeat(64))),
        authority: strong_reference_to(&authority_id, &format!("sha256:{}", "b".repeat(64))),
        resource_scope: strong_reference_to(&scope_id, &format!("sha256:{}", "c".repeat(64))),
        tenant_id: Some("personal".to_owned()),
        created_by: "principal://personal/daemon".to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "campaign-bounded".to_owned(),
    };
    let header = compose_governed_header(
        &contract_id,
        "TaskContract",
        "cognitiveos.task-contract/0.4",
        &governance,
        Vec::new(),
        Vec::new(),
        "p2-t17-a7-external-mutation",
        issued_at,
    )
    .map_err(|error| infrastructure("compose campaign TaskContract", error))?;
    let contract = TaskContract {
        allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
        allowed_tools: vec!["campaign.external.mutate".to_owned()],
        budget: Budget {
            attention_slots: None,
            context_bytes: None,
            egress_bytes: Some(16 * 1024),
            input_tokens: None,
            money_microunits: None,
            output_tokens: None,
            semantic_calls: None,
            tool_calls: Some(1),
            wall_time_ms: Some(5_000),
        },
        budget_id: Some(budget_id.to_generated()),
        conditions: vec![ContractCondition {
            description: "reconciled external post-state is independently fixed".to_owned(),
            id: "a7-fixed-post-state".to_owned(),
            kind: ContractConditionKind::Acceptance,
            machine_expression: None,
            verifier_ref: Some(FIXED_EFFECT_VERIFIER_REF.to_owned()),
        }],
        context_request_ref: None,
        contract_epoch: 1,
        deadline: Some("2099-01-01T00:00:00Z".to_owned()),
        header,
        human_gates: None,
        intent_acceptance_ref: strong_reference_to(
            &acceptance_id,
            &format!("sha256:{}", "d".repeat(64)),
        ),
        intent_interpretation_ref: strong_reference_to(
            &interpretation_id,
            &format!("sha256:{}", "e".repeat(64)),
        ),
        loop_object_id: Some(loop_object_id.to_generated()),
        max_iterations: 2,
        max_retries: 0,
        objective: "observe and reconcile one external mutation".to_owned(),
        scope: TaskScope {
            in_scope: vec!["campaign-owned loopback fixture".to_owned()],
            out_of_scope: vec!["Task completion".to_owned()],
        },
        task_ref: task_ref.to_owned(),
        user_intent_ref: strong_reference_to(
            &intent_record_id,
            &format!("sha256:{}", "f".repeat(64)),
        ),
        worker_authorization_root_id: Some(contract_id.to_generated()),
    };
    let (sealed, digest) =
        seal_governed_object_content_digest(serde_json::to_value(&contract).map_err(json_error)?)
            .map_err(|error| infrastructure("seal campaign TaskContract", error))?;
    let budget_state = BudgetState::new(BTreeMap::from([
        ("egress_bytes".to_owned(), 16 * 1024),
        ("tool_calls".to_owned(), 1),
        ("wall_time_ms".to_owned(), 5_000),
    ]))
    .map_err(|error| infrastructure("build campaign budget", error))?;
    Ok((
        TaskContractRow {
            contract_id,
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
            user_intent_record_id: intent_record_id,
            interpretation_id,
            accepted_by: "principal://personal/daemon".to_owned(),
            contract_digest: digest,
            canonical_json: serde_json::to_string(&sealed).map_err(json_error)?,
        },
        serde_json::to_string(&budget_state).map_err(json_error)?,
    ))
}

fn event(
    identifiers: &UuidV7Generator,
    object_id: &ObjectId,
    domain: LifecycleDomain,
    event_type: &str,
    event_time: &WallTimestamp,
) -> Result<EventDraft, CampaignObservationError> {
    Ok(EventDraft {
        event_id: next_event_id(identifiers)?,
        object_id: object_id.clone(),
        domain,
        object_version: Version::INITIAL,
        event_type: event_type.to_owned(),
        canonical_json: json!({
            "event_type": event_type,
            "object_id": object_id.as_str(),
            "event_time": event_time.as_str(),
        })
        .to_string(),
    })
}

fn next_object_id(identifiers: &UuidV7Generator) -> Result<ObjectId, CampaignObservationError> {
    ObjectId::parse(
        &identifiers
            .next_uuid_v7()
            .map_err(|error| infrastructure("mint object id", error))?,
    )
    .map_err(|error| infrastructure("parse object id", error))
}

fn next_event_id(identifiers: &UuidV7Generator) -> Result<EventId, CampaignObservationError> {
    EventId::parse(
        &identifiers
            .next_uuid_v7()
            .map_err(|error| infrastructure("mint event id", error))?,
    )
    .map_err(|error| infrastructure("parse event id", error))
}

fn next_budget_id(identifiers: &UuidV7Generator) -> Result<BudgetId, CampaignObservationError> {
    BudgetId::parse(
        &identifiers
            .next_uuid_v7()
            .map_err(|error| infrastructure("mint budget id", error))?,
    )
    .map_err(|error| infrastructure("parse budget id", error))
}

fn post_state_digest(version: i64, value: i64) -> Result<String, CampaignObservationError> {
    let bytes =
        serde_json::to_vec(&json!({"value": value, "version": version})).map_err(json_error)?;
    canonical::digest(&bytes, POST_STATE_DIGEST_DOMAIN)
        .map_err(|error| infrastructure("digest fixture post-state", error))
}

fn digest_text(value: &str, domain: &str) -> Result<String, CampaignObservationError> {
    canonical::digest(value.as_bytes(), domain)
        .map_err(|error| infrastructure("digest bounded campaign value", error))
}

fn effect_ref(effect_id: &ObjectId) -> String {
    format!("effect://personal/{}", effect_id.as_str())
}

fn stage_timing(stage: &str, started: Instant) -> CampaignStageTiming {
    CampaignStageTiming {
        stage: stage.to_owned(),
        elapsed_micros: started.elapsed().as_micros().min(u64::MAX as u128) as u64,
    }
}

fn write_json_durable<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), CampaignObservationError> {
    let parent = path.parent().ok_or_else(|| {
        CampaignObservationError::Infrastructure("durable path has no parent".to_owned())
    })?;
    ensure_safe_directory(parent)?;
    let bytes = serde_json::to_vec(value).map_err(json_error)?;
    let temporary = parent.join(format!(
        ".a7-{}-{}",
        std::process::id(),
        TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        #[cfg(windows)]
        if path.exists() {
            fs::remove_file(path)?;
        }
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(infrastructure("persist campaign state", error));
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, CampaignObservationError> {
    let bytes = fs::read(path).map_err(|error| infrastructure("read campaign state", error))?;
    if bytes.len() > MAXIMUM_HTTP_BYTES * 4 {
        return Err(CampaignObservationError::Infrastructure(
            "campaign state exceeds byte bound".to_owned(),
        ));
    }
    serde_json::from_slice(&bytes).map_err(json_error)
}

fn ensure_safe_directory(path: &Path) -> Result<(), CampaignObservationError> {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.file_type().is_symlink()
    {
        return Err(CampaignObservationError::Infrastructure(
            "campaign state root must not be a symlink".to_owned(),
        ));
    }
    fs::create_dir_all(path).map_err(|error| infrastructure("create campaign directory", error))
}

fn endpoint_address(endpoint: &str) -> Result<std::net::SocketAddr, CampaignObservationError> {
    let port = endpoint
        .strip_prefix("http://127.0.0.1:")
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|port| *port != 0)
        .ok_or_else(|| {
            CampaignObservationError::Infrastructure(
                "fixture endpoint must be an explicit IPv4 loopback port".to_owned(),
            )
        })?;
    Ok(std::net::SocketAddr::from(([127, 0, 0, 1], port)))
}

fn validate_fixture_endpoint(endpoint: &str) -> Result<(), CampaignObservationError> {
    endpoint_address(endpoint).map(|_| ())
}

fn valid_case_ref(case_ref: &str) -> bool {
    case_ref
        .strip_prefix("A7-")
        .is_some_and(|suffix| suffix.len() == 3 && suffix.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_idempotency_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 128
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_uuid_text(value: &str) -> bool {
    value.len() == 36
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn lock<'a, T>(
    mutex: &'a Mutex<T>,
    label: &str,
) -> Result<std::sync::MutexGuard<'a, T>, CampaignObservationError> {
    mutex
        .lock()
        .map_err(|_| CampaignObservationError::Infrastructure(format!("{label} lock is poisoned")))
}

fn infrastructure(context: &str, error: impl std::fmt::Display) -> CampaignObservationError {
    CampaignObservationError::Infrastructure(format!("{context}: {error}"))
}

fn duplicate_or_infrastructure(
    context: &str,
) -> impl Fn(cognitive_kernel::ports::StorePortError) -> CampaignObservationError {
    move |error| match error {
        cognitive_kernel::ports::StorePortError::Conflict { .. } => {
            CampaignObservationError::DuplicateEffect
        }
        other => infrastructure(context, other),
    }
}

fn json_error(error: serde_json::Error) -> CampaignObservationError {
    infrastructure("serialize campaign JSON", error)
}
