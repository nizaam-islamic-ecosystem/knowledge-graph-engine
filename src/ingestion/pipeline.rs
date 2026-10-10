//! Fixed-core ingestion pipeline, run tracking, stage hooks, and reprocessing.
//!
//! This module coordinates phase-specific stage contracts; it is not a generic
//! workflow engine. Every ingestion run uses Core `OperationId`. Candidate states
//! are tracked separately so one candidate's failure does not fail an entire run.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use nizaam_core::identity::OperationId;

use crate::identity::SourceId;
use crate::temporal::Instant;

use super::mapping::{CandidateKey, MappedCandidate};
use super::raw::SourceClass;

/// Fixed Phase 5 stage sequence. Optional hooks may be inserted after a core stage.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PipelineStage {
    /// Raw artifact/source snapshot has been associated with this run.
    RawCapture,
    /// Envelope and source-record metadata checks before resolution.
    StructuralValidation,
    /// Deterministic generic/source-specific normalization.
    Normalization,
    /// Source-specific mapping into semantic candidates.
    Mapping,
    /// Deduplication by source record identity before resolution.
    SourceDeduplication,
    /// Existing Phase 3 deterministic entity resolution.
    EntityResolution,
    /// Semantic, ontology, reference, temporal, evidence, and provenance validation.
    SemanticValidation,
    /// Deduplication by canonical semantic identities after resolution.
    SemanticDeduplication,
    /// Human approval and source-class governance.
    Governance,
    /// Indexing readiness acknowledgement required before logical publication.
    IndexingReadiness,
    /// Per-candidate logical canonical publication.
    Publication,
    /// Post-publication Indexing synchronization status.
    IndexSynchronization,
}

impl PipelineStage {
    /// Fixed order used by the Phase 5 core pipeline.
    pub const CORE_ORDER: [Self; 12] = [
        Self::RawCapture,
        Self::StructuralValidation,
        Self::Normalization,
        Self::Mapping,
        Self::SourceDeduplication,
        Self::EntityResolution,
        Self::SemanticValidation,
        Self::SemanticDeduplication,
        Self::Governance,
        Self::IndexingReadiness,
        Self::Publication,
        Self::IndexSynchronization,
    ];

    /// Returns a stable stage label for logs and policy/configuration records.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawCapture => "raw-capture",
            Self::StructuralValidation => "structural-validation",
            Self::Normalization => "normalization",
            Self::Mapping => "mapping",
            Self::SourceDeduplication => "source-deduplication",
            Self::EntityResolution => "entity-resolution",
            Self::SemanticValidation => "semantic-validation",
            Self::SemanticDeduplication => "semantic-deduplication",
            Self::Governance => "governance",
            Self::IndexingReadiness => "indexing-readiness",
            Self::Publication => "publication",
            Self::IndexSynchronization => "index-synchronization",
        }
    }
}

/// Stage code/configuration versions frozen into a plan for deterministic replay.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PipelineStageVersion {
    stage: PipelineStage,
    stage_version: String,
    configuration_version: String,
}

impl PipelineStageVersion {
    /// Creates version metadata for a core stage.
    pub fn new(
        stage: PipelineStage,
        stage_version: impl Into<String>,
        configuration_version: impl Into<String>,
    ) -> Result<Self, PipelineError> {
        let stage_version = stage_version.into();
        let configuration_version = configuration_version.into();
        validate_label(&stage_version, "pipeline stage version")?;
        validate_label(&configuration_version, "pipeline configuration version")?;
        Ok(Self {
            stage,
            stage_version,
            configuration_version,
        })
    }

    /// Returns the fixed core stage.
    #[must_use]
    pub const fn stage(&self) -> PipelineStage {
        self.stage
    }
    /// Returns the stage implementation version.
    #[must_use]
    pub fn stage_version(&self) -> &str {
        &self.stage_version
    }
    /// Returns the configuration/policy version used by the stage.
    #[must_use]
    pub fn configuration_version(&self) -> &str {
        &self.configuration_version
    }
}

/// An optional source-specific hook inserted immediately after one core stage.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PipelineStageHook {
    name: String,
    after: PipelineStage,
    stage_version: String,
    configuration_version: String,
}

impl PipelineStageHook {
    /// Creates a named optional hook. Hooks supplement; they cannot replace the core stages.
    pub fn new(
        name: impl Into<String>,
        after: PipelineStage,
        stage_version: impl Into<String>,
        configuration_version: impl Into<String>,
    ) -> Result<Self, PipelineError> {
        let name = name.into();
        let stage_version = stage_version.into();
        let configuration_version = configuration_version.into();
        validate_label(&name, "pipeline hook name")?;
        validate_label(&stage_version, "pipeline hook version")?;
        validate_label(
            &configuration_version,
            "pipeline hook configuration version",
        )?;
        Ok(Self {
            name,
            after,
            stage_version,
            configuration_version,
        })
    }

    /// Returns the hook name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the core stage after which this hook runs.
    #[must_use]
    pub const fn after(&self) -> PipelineStage {
        self.after
    }
}

/// A planned core stage or optional hook with explicit versions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PipelineStepVersion {
    /// A stage from the fixed Phase 5 core pipeline.
    Core(PipelineStageVersion),
    /// An optional source-specific hook.
    Hook(PipelineStageHook),
}

impl PipelineStepVersion {
    /// Returns the stable step name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Core(value) => value.stage().as_str(),
            Self::Hook(value) => value.name(),
        }
    }

    /// Returns the implementation version.
    #[must_use]
    pub fn stage_version(&self) -> &str {
        match self {
            Self::Core(value) => value.stage_version(),
            Self::Hook(value) => &value.stage_version,
        }
    }

    /// Returns the configuration version.
    #[must_use]
    pub fn configuration_version(&self) -> &str {
        match self {
            Self::Core(value) => value.configuration_version(),
            Self::Hook(value) => &value.configuration_version,
        }
    }
}

/// Immutable core stage sequence plus optional source-specific hooks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipelinePlan {
    core_stages: BTreeMap<PipelineStage, PipelineStageVersion>,
    hooks: Vec<PipelineStageHook>,
}

impl PipelinePlan {
    /// Creates a plan that must specify each core stage exactly once.
    pub fn new<I, H>(core_stages: I, hooks: H) -> Result<Self, PipelineError>
    where
        I: IntoIterator<Item = PipelineStageVersion>,
        H: IntoIterator<Item = PipelineStageHook>,
    {
        let mut stage_map = BTreeMap::new();
        for stage in core_stages {
            let key = stage.stage();
            if stage_map.insert(key, stage).is_some() {
                return Err(PipelineError::DuplicateCoreStage { stage: key });
            }
        }
        for required in PipelineStage::CORE_ORDER {
            if !stage_map.contains_key(&required) {
                return Err(PipelineError::MissingCoreStage { stage: required });
            }
        }

        let mut hooks = hooks.into_iter().collect::<Vec<_>>();
        hooks.sort_by(|left, right| {
            left.after()
                .cmp(&right.after())
                .then_with(|| left.name().cmp(right.name()))
        });
        let mut hook_names = BTreeSet::new();
        for hook in &hooks {
            if !hook_names.insert(hook.name().to_owned()) {
                return Err(PipelineError::DuplicateHook {
                    name: hook.name().to_owned(),
                });
            }
        }
        Ok(Self {
            core_stages: stage_map,
            hooks,
        })
    }

    /// Creates the default fixed Phase 5 stage plan with version labels.
    #[must_use]
    pub fn phase5_default() -> Self {
        let stages = PipelineStage::CORE_ORDER.into_iter().map(|stage| {
            PipelineStageVersion::new(stage, "phase5-v1", "phase5-default-config-v1")
                .expect("fixed Phase 5 stage labels are valid")
        });
        Self::new(stages, std::iter::empty())
            .expect("default Phase 5 plan contains all core stages")
    }

    /// Adds a hook by returning a new validated plan.
    pub fn with_hook(&self, hook: PipelineStageHook) -> Result<Self, PipelineError> {
        let mut hooks = self.hooks.clone();
        hooks.push(hook);
        let stages = self.core_stages.values().cloned();
        Self::new(stages, hooks)
    }

    /// Returns the ordered core stages and hooks.
    #[must_use]
    pub fn ordered_steps(&self) -> Vec<PipelineStepVersion> {
        let mut steps = Vec::new();
        for stage in PipelineStage::CORE_ORDER {
            steps.push(PipelineStepVersion::Core(self.core_stages[&stage].clone()));
            for hook in self.hooks.iter().filter(|hook| hook.after() == stage) {
                steps.push(PipelineStepVersion::Hook(hook.clone()));
            }
        }
        steps
    }

    /// Returns the configured version for one core stage.
    #[must_use]
    pub fn stage_version(&self, stage: PipelineStage) -> &PipelineStageVersion {
        &self.core_stages[&stage]
    }
}

/// Overall status of one ingestion run.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IngestionRunState {
    /// Run metadata is initialized but execution has not started.
    Created,
    /// Stages are executing or awaiting per-candidate outcomes.
    Running,
    /// Every registered candidate reached a successful terminal state.
    Completed,
    /// The run completed with quarantined/rejected/blocked or unsynchronized candidates.
    CompletedWithIssues,
    /// A run-level failure prevented safe completion.
    Failed,
    /// Execution was cancelled through its owning operation.
    Cancelled,
}

impl IngestionRunState {
    /// Returns whether this run state is terminal.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::CompletedWithIssues | Self::Failed | Self::Cancelled
        )
    }
}

/// Per-candidate lifecycle. This is intentionally distinct from run-level state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CandidateState {
    /// The source record became an ingestion candidate.
    Received,
    /// The source record passed normalization.
    Normalized,
    /// Semantic mapping produced one or more candidates.
    Mapped,
    /// Source-level deduplication completed.
    SourceDeduplicated,
    /// Entity resolution completed or established that it was not needed.
    Resolved,
    /// Main semantic validation completed.
    SemanticValidated,
    /// A recoverable error requires correction/reprocessing.
    Quarantined,
    /// A fatal finding blocks publication.
    Blocked,
    /// Human approval is pending.
    AwaitingApproval,
    /// Human approval has been recorded.
    Approved,
    /// All semantic/publication preconditions passed.
    ReadyForPublication,
    /// The candidate was accepted into logical canonical KG state.
    Published,
    /// Publication succeeded and Indexing synchronization is pending.
    SynchronizationPending,
    /// Indexing synchronization completed.
    Synchronized,
    /// Publication succeeded but Indexing synchronization failed.
    SynchronizationFailed,
    /// A human reviewer rejected the candidate.
    Rejected,
    /// A published candidate was withdrawn through a governed operation.
    Withdrawn,
}

impl CandidateState {
    /// Returns whether the candidate has no normal path to more active processing.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Blocked | Self::Rejected | Self::Withdrawn | Self::Synchronized
        )
    }
}

/// One immutable candidate-state transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateTransition {
    from: CandidateState,
    to: CandidateState,
    at: Instant,
    reason: String,
}

impl CandidateTransition {
    /// Previous state.
    #[must_use]
    pub const fn from(&self) -> CandidateState {
        self.from
    }
    /// New state.
    #[must_use]
    pub const fn to(&self) -> CandidateState {
        self.to
    }
    /// Transition time.
    #[must_use]
    pub const fn at(&self) -> Instant {
        self.at
    }
    /// Transition reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Result recorded for an individual stage attempt.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StageExecutionOutcome {
    /// Stage completed successfully.
    Succeeded,
    /// Stage intentionally skipped the candidate under an explicit rule.
    Skipped,
    /// Recoverable failure requires quarantine/reprocessing.
    Quarantined,
    /// Fatal failure blocks this candidate.
    Blocked,
    /// Stage failed without classifying the candidate-level recovery yet.
    Failed,
    /// Stage failed in a way eligible for targeted retry.
    RetryableFailure,
}

/// Immutable record of one stage attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageExecution {
    step: PipelineStepVersion,
    attempt: u32,
    outcome: StageExecutionOutcome,
    started_at: Instant,
    finished_at: Instant,
    detail: Option<String>,
}

impl StageExecution {
    /// Executed plan step.
    #[must_use]
    pub fn step(&self) -> &PipelineStepVersion {
        &self.step
    }
    /// One-based attempt number.
    #[must_use]
    pub const fn attempt(&self) -> u32 {
        self.attempt
    }
    /// Attempt outcome.
    #[must_use]
    pub const fn outcome(&self) -> StageExecutionOutcome {
        self.outcome
    }
    /// Start time.
    #[must_use]
    pub const fn started_at(&self) -> Instant {
        self.started_at
    }
    /// Finish time.
    #[must_use]
    pub const fn finished_at(&self) -> Instant {
        self.finished_at
    }
    /// Optional stage detail.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

/// Input bundle for recording one pipeline-stage attempt.
///
/// Validation that depends on a run's state or frozen plan is performed by
/// `IngestionRun::record_stage_execution`; this value only groups the attempt
/// fields into one typed parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageExecutionInput {
    step: PipelineStepVersion,
    attempt: u32,
    outcome: StageExecutionOutcome,
    started_at: Instant,
    finished_at: Instant,
    detail: Option<String>,
}

impl StageExecutionInput {
    /// Creates an input describing one stage attempt.
    #[must_use]
    pub fn new(
        step: PipelineStepVersion,
        attempt: u32,
        outcome: StageExecutionOutcome,
        started_at: Instant,
        finished_at: Instant,
        detail: Option<String>,
    ) -> Self {
        Self {
            step,
            attempt,
            outcome,
            started_at,
            finished_at,
            detail,
        }
    }
}

/// Tracking for one candidate, independent of other candidates in its run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateProgress {
    key: CandidateKey,
    state: CandidateState,
    transitions: Vec<CandidateTransition>,
    stage_executions: Vec<StageExecution>,
    last_successful_step_index: Option<usize>,
}

impl CandidateProgress {
    fn new(key: CandidateKey) -> Self {
        Self {
            key,
            state: CandidateState::Received,
            transitions: Vec::new(),
            stage_executions: Vec::new(),
            last_successful_step_index: None,
        }
    }

    /// Candidate key.
    #[must_use]
    pub fn key(&self) -> &CandidateKey {
        &self.key
    }
    /// Current candidate state.
    #[must_use]
    pub const fn state(&self) -> CandidateState {
        self.state
    }
    /// Immutable candidate state-transition history.
    #[must_use]
    pub fn transitions(&self) -> &[CandidateTransition] {
        &self.transitions
    }
    /// Immutable stage-attempt history.
    #[must_use]
    pub fn stage_executions(&self) -> &[StageExecution] {
        &self.stage_executions
    }
}

/// One Core-operation-scoped ingestion run and its candidate ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngestionRun {
    operation_id: OperationId,
    source_id: SourceId,
    source_class: SourceClass,
    source_version: Option<String>,
    started_at: Instant,
    state: IngestionRunState,
    plan: PipelinePlan,
    candidates: BTreeMap<CandidateKey, CandidateProgress>,
    run_error: Option<String>,
}

impl IngestionRun {
    /// Creates run metadata using Core's existing operation identity.
    pub fn new(
        operation_id: OperationId,
        source_id: SourceId,
        source_class: SourceClass,
        source_version: Option<String>,
        started_at: Instant,
        plan: PipelinePlan,
    ) -> Result<Self, PipelineError> {
        if let Some(version) = &source_version {
            validate_label(version, "source version")?;
        }
        Ok(Self {
            operation_id,
            source_id,
            source_class,
            source_version,
            started_at,
            state: IngestionRunState::Created,
            plan,
            candidates: BTreeMap::new(),
            run_error: None,
        })
    }

    /// Starts this run. Candidate registration and stage recording require Running.
    pub fn start(&mut self) -> Result<(), PipelineError> {
        if self.state != IngestionRunState::Created {
            return Err(PipelineError::InvalidRunTransition {
                from: self.state,
                to: IngestionRunState::Running,
            });
        }
        self.state = IngestionRunState::Running;
        Ok(())
    }

    /// Registers a candidate from this run's source and snapshot.
    pub fn register_candidate<T>(
        &mut self,
        candidate: &MappedCandidate<T>,
    ) -> Result<(), PipelineError> {
        self.ensure_running()?;
        let key = candidate.key().clone();
        if key.source_id() != &self.source_id
            || key.source_version() != self.source_version.as_deref()
            || candidate.source().source_class() != &self.source_class
        {
            return Err(PipelineError::CandidateRunMismatch);
        }
        if self.candidates.contains_key(&key) {
            return Err(PipelineError::DuplicateCandidate { key });
        }
        self.candidates
            .insert(key.clone(), CandidateProgress::new(key));
        Ok(())
    }

    /// Applies a checked candidate-state transition, preserving a transition history.
    pub fn transition_candidate(
        &mut self,
        key: &CandidateKey,
        next: CandidateState,
        at: Instant,
        reason: impl Into<String>,
    ) -> Result<(), PipelineError> {
        self.ensure_running()?;
        let reason = reason.into();
        validate_label(&reason, "candidate transition reason")?;
        let progress = self
            .candidates
            .get_mut(key)
            .ok_or_else(|| PipelineError::CandidateNotFound { key: key.clone() })?;
        if !candidate_transition_allowed(progress.state, next) {
            return Err(PipelineError::InvalidCandidateTransition {
                from: progress.state,
                to: next,
            });
        }
        progress.transitions.push(CandidateTransition {
            from: progress.state,
            to: next,
            at,
            reason,
        });
        progress.state = next;
        Ok(())
    }

    /// Records one planned stage attempt and enforces the next required stage in the frozen plan.
    pub fn record_stage_execution(
        &mut self,
        key: &CandidateKey,
        input: StageExecutionInput,
    ) -> Result<(), PipelineError> {
        self.ensure_running()?;
        let StageExecutionInput {
            step,
            attempt,
            outcome,
            started_at,
            finished_at,
            detail,
        } = input;
        if attempt == 0 {
            return Err(PipelineError::InvalidAttemptNumber);
        }
        if finished_at < started_at {
            return Err(PipelineError::InvalidStageTimeRange);
        }
        if let Some(detail) = &detail {
            validate_label(detail, "stage execution detail")?;
        }
        let ordered = self.plan.ordered_steps();
        let index = ordered
            .iter()
            .position(|planned| planned == &step)
            .ok_or_else(|| PipelineError::UnplannedStage {
                name: step.name().to_owned(),
            })?;
        let progress = self
            .candidates
            .get_mut(key)
            .ok_or_else(|| PipelineError::CandidateNotFound { key: key.clone() })?;
        let expected_index = progress
            .last_successful_step_index
            .map_or(0, |last_successful| last_successful + 1);
        if index != expected_index {
            let expected_step = ordered
                .get(expected_index)
                .map_or("<no further stages>", PipelineStepVersion::name)
                .to_owned();
            return Err(PipelineError::StageOrderMismatch {
                expected_step,
                actual_step: step.name().to_owned(),
            });
        }
        progress.stage_executions.push(StageExecution {
            step,
            attempt,
            outcome,
            started_at,
            finished_at,
            detail,
        });
        if matches!(
            outcome,
            StageExecutionOutcome::Succeeded | StageExecutionOutcome::Skipped
        ) {
            progress.last_successful_step_index = Some(index);
        }
        Ok(())
    }

    /// Finishes the run, preserving candidate-level partial outcomes.
    pub fn finish(&mut self, outcome: RunCompletion) -> Result<(), PipelineError> {
        self.ensure_running()?;
        if outcome == RunCompletion::Completed
            && self
                .candidates
                .values()
                .any(|candidate| candidate.state != CandidateState::Synchronized)
        {
            return Err(PipelineError::RunHasIncompleteCandidates);
        }
        self.state = match outcome {
            RunCompletion::Completed => IngestionRunState::Completed,
            RunCompletion::CompletedWithIssues => IngestionRunState::CompletedWithIssues,
            RunCompletion::Failed => IngestionRunState::Failed,
            RunCompletion::Cancelled => IngestionRunState::Cancelled,
        };
        Ok(())
    }

    /// Records a run-level failure without modifying per-candidate results.
    pub fn fail(&mut self, reason: impl Into<String>) -> Result<(), PipelineError> {
        self.ensure_running()?;
        let reason = reason.into();
        validate_label(&reason, "run failure reason")?;
        self.run_error = Some(reason);
        self.state = IngestionRunState::Failed;
        Ok(())
    }

    fn ensure_running(&self) -> Result<(), PipelineError> {
        if self.state != IngestionRunState::Running {
            return Err(PipelineError::RunNotRunning { state: self.state });
        }
        Ok(())
    }

    /// Core OperationId is retained as the run identity.
    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    /// Source associated with this run.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }
    /// Source-class policy context.
    #[must_use]
    pub fn source_class(&self) -> &SourceClass {
        &self.source_class
    }
    /// Source snapshot version, if present.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }
    /// Run creation/start timestamp recorded by the KG caller.
    #[must_use]
    pub const fn started_at(&self) -> Instant {
        self.started_at
    }
    /// Current run-level state.
    #[must_use]
    pub const fn state(&self) -> IngestionRunState {
        self.state
    }
    /// Immutable pipeline plan used for this run.
    #[must_use]
    pub fn plan(&self) -> &PipelinePlan {
        &self.plan
    }
    /// One candidate progress record, if registered.
    #[must_use]
    pub fn candidate(&self, key: &CandidateKey) -> Option<&CandidateProgress> {
        self.candidates.get(key)
    }
    /// All candidate progress records, in deterministic key order.
    pub fn candidates(&self) -> impl Iterator<Item = (&CandidateKey, &CandidateProgress)> {
        self.candidates.iter()
    }
    /// Run-level failure detail, when the run failed.
    #[must_use]
    pub fn run_error(&self) -> Option<&str> {
        self.run_error.as_deref()
    }
}

/// Requested run completion outcome.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RunCompletion {
    /// Every candidate reached Synchronized.
    Completed,
    /// Run ended with some candidate blocked, quarantined, rejected, or unsynchronized.
    CompletedWithIssues,
    /// Run-level failure.
    Failed,
    /// Run was cancelled.
    Cancelled,
}

/// Explicit reprocessing scope. Targeted candidate retry is the default direction;
/// full-source re-ingestion is a separate, explicit scope value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReprocessingMode {
    /// Reprocess only listed candidates.
    TargetedCandidates(BTreeSet<CandidateKey>),
    /// Reprocess changed record keys between two source snapshots.
    SourceDelta(SourceDelta),
    /// Explicitly reprocess the complete source snapshot.
    FullSource,
}

/// Source snapshot delta; an empty change set is rejected so callers must use
/// `FullSource` when they intentionally request a complete re-ingestion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDelta {
    previous_version: String,
    current_version: String,
    changed_record_keys: BTreeSet<String>,
}

impl SourceDelta {
    /// Creates a nonempty, versioned delta.
    pub fn new<I>(
        previous_version: impl Into<String>,
        current_version: impl Into<String>,
        changed_record_keys: I,
    ) -> Result<Self, PipelineError>
    where
        I: IntoIterator<Item = String>,
    {
        let previous_version = previous_version.into();
        let current_version = current_version.into();
        validate_label(&previous_version, "previous source version")?;
        validate_label(&current_version, "current source version")?;
        if previous_version == current_version {
            return Err(PipelineError::UnchangedSourceVersion);
        }
        let changed_record_keys = changed_record_keys.into_iter().collect::<BTreeSet<_>>();
        if changed_record_keys.is_empty() {
            return Err(PipelineError::EmptySourceDelta);
        }
        for key in &changed_record_keys {
            validate_label(key, "changed source record key")?;
        }
        Ok(Self {
            previous_version,
            current_version,
            changed_record_keys,
        })
    }

    /// Previous source snapshot version.
    #[must_use]
    pub fn previous_version(&self) -> &str {
        &self.previous_version
    }
    /// Current source snapshot version.
    #[must_use]
    pub fn current_version(&self) -> &str {
        &self.current_version
    }
    /// Changed record keys in deterministic order.
    #[must_use]
    pub fn changed_record_keys(&self) -> &BTreeSet<String> {
        &self.changed_record_keys
    }
}

/// Request to reprocess a defined source scope under a new Core operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReprocessingRequest {
    operation_id: OperationId,
    source_id: SourceId,
    source_version: Option<String>,
    mode: ReprocessingMode,
    requested_at: Instant,
    reason: String,
}

impl ReprocessingRequest {
    /// Creates a validated targeted, delta, or explicit full-source request.
    pub fn new(
        operation_id: OperationId,
        source_id: SourceId,
        source_version: Option<String>,
        mode: ReprocessingMode,
        requested_at: Instant,
        reason: impl Into<String>,
    ) -> Result<Self, PipelineError> {
        if let Some(version) = &source_version {
            validate_label(version, "source version")?;
        }
        let reason = reason.into();
        validate_label(&reason, "reprocessing reason")?;
        match &mode {
            ReprocessingMode::TargetedCandidates(keys) => {
                if keys.is_empty() {
                    return Err(PipelineError::EmptyTargetedReprocessing);
                }
                if keys.iter().any(|key| {
                    key.source_id() != &source_id
                        || key.source_version() != source_version.as_deref()
                }) {
                    return Err(PipelineError::ReprocessingTargetMismatch);
                }
            }
            ReprocessingMode::SourceDelta(delta) => {
                if source_version.as_deref() != Some(delta.current_version()) {
                    return Err(PipelineError::ReprocessingTargetMismatch);
                }
            }
            ReprocessingMode::FullSource => {}
        }
        Ok(Self {
            operation_id,
            source_id,
            source_version,
            mode,
            requested_at,
            reason,
        })
    }

    /// Core operation identity used for this reprocessing attempt.
    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    /// Source being reprocessed.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }
    /// Target source snapshot.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }
    /// Requested scope.
    #[must_use]
    pub fn mode(&self) -> &ReprocessingMode {
        &self.mode
    }
    /// Request timestamp.
    #[must_use]
    pub const fn requested_at(&self) -> Instant {
        self.requested_at
    }
    /// Explicit reason supporting the reprocessing request.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Phase 5 pipeline plan/run constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngestionPipeline {
    plan: PipelinePlan,
}

impl IngestionPipeline {
    /// Creates a pipeline from a validated plan.
    #[must_use]
    pub fn new(plan: PipelinePlan) -> Self {
        Self { plan }
    }

    /// Creates the fixed default Phase 5 pipeline.
    #[must_use]
    pub fn phase5_default() -> Self {
        Self::new(PipelinePlan::phase5_default())
    }

    /// Returns the immutable configured plan.
    #[must_use]
    pub fn plan(&self) -> &PipelinePlan {
        &self.plan
    }

    /// Starts a new run envelope using the existing Core operation identity.
    pub fn begin_run(
        &self,
        operation_id: OperationId,
        source_id: SourceId,
        source_class: SourceClass,
        source_version: Option<String>,
        started_at: Instant,
    ) -> Result<IngestionRun, PipelineError> {
        IngestionRun::new(
            operation_id,
            source_id,
            source_class,
            source_version,
            started_at,
            self.plan.clone(),
        )
    }
}

fn candidate_transition_allowed(from: CandidateState, to: CandidateState) -> bool {
    use CandidateState as S;
    matches!(
        (from, to),
        (S::Received, S::Normalized | S::Quarantined | S::Blocked)
            | (S::Normalized, S::Mapped | S::Quarantined | S::Blocked)
            | (
                S::Mapped,
                S::SourceDeduplicated | S::Resolved | S::Quarantined | S::Blocked
            )
            | (
                S::SourceDeduplicated,
                S::Resolved | S::SemanticValidated | S::Quarantined | S::Blocked
            )
            | (
                S::Resolved,
                S::SemanticValidated | S::Quarantined | S::Blocked
            )
            | (
                S::SemanticValidated,
                S::AwaitingApproval | S::Quarantined | S::Blocked | S::Rejected
            )
            | (
                S::Quarantined,
                S::Received
                    | S::Normalized
                    | S::Mapped
                    | S::SemanticValidated
                    | S::AwaitingApproval
                    | S::Rejected
                    | S::Blocked
            )
            | (
                S::AwaitingApproval,
                S::Approved | S::Rejected | S::Quarantined | S::Blocked
            )
            | (
                S::Approved,
                S::ReadyForPublication | S::Rejected | S::Blocked
            )
            | (S::ReadyForPublication, S::Published | S::Blocked)
            | (
                S::Published,
                S::SynchronizationPending
                    | S::Synchronized
                    | S::SynchronizationFailed
                    | S::Withdrawn
            )
            | (
                S::SynchronizationPending,
                S::Synchronized | S::SynchronizationFailed
            )
            | (
                S::SynchronizationFailed,
                S::SynchronizationPending | S::Synchronized | S::Withdrawn
            )
            | (S::Synchronized, S::Withdrawn)
    )
}

/// Errors from pipeline planning, run tracking, and reprocessing validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PipelineError {
    /// A required stage version, hook, or reason label is invalid.
    InvalidLabel { field: &'static str },
    /// A core stage appears more than once in a plan.
    DuplicateCoreStage { stage: PipelineStage },
    /// A fixed core stage is missing from a plan.
    MissingCoreStage { stage: PipelineStage },
    /// A hook name is duplicated.
    DuplicateHook { name: String },
    /// Candidate source/source-version does not match the run source/source-version.
    CandidateRunMismatch,
    /// Candidate key is already registered in the run.
    DuplicateCandidate { key: CandidateKey },
    /// Candidate key was not registered in this run.
    CandidateNotFound { key: CandidateKey },
    /// Run transition is invalid.
    InvalidRunTransition {
        from: IngestionRunState,
        to: IngestionRunState,
    },
    /// An operation requiring a running ingestion run was called in another state.
    RunNotRunning { state: IngestionRunState },
    /// Candidate transition is invalid.
    InvalidCandidateTransition {
        from: CandidateState,
        to: CandidateState,
    },
    /// Candidate stage execution references a step absent from the frozen plan.
    UnplannedStage { name: String },
    /// A stage did not match the next expected step in the frozen plan.
    StageOrderMismatch {
        expected_step: String,
        actual_step: String,
    },
    /// Attempt numbers are one-based.
    InvalidAttemptNumber,
    /// Finish time is earlier than start time.
    InvalidStageTimeRange,
    /// A run declared success while one or more candidates were unfinished.
    RunHasIncompleteCandidates,
    /// A reprocessing request specifies no candidates.
    EmptyTargetedReprocessing,
    /// A source delta has no changed keys.
    EmptySourceDelta,
    /// Delta versions are equal.
    UnchangedSourceVersion,
    /// Reprocessing keys/source-version do not match its source target.
    ReprocessingTargetMismatch,
}

impl fmt::Display for PipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLabel { field } => write!(formatter, "{field} is invalid"),
            Self::DuplicateCoreStage { stage } => write!(formatter, "duplicate core stage {}", stage.as_str()),
            Self::MissingCoreStage { stage } => write!(formatter, "missing required core stage {}", stage.as_str()),
            Self::DuplicateHook { name } => write!(formatter, "duplicate pipeline hook {name}"),
            Self::CandidateRunMismatch => formatter.write_str("candidate source/snapshot does not match ingestion run"),
            Self::DuplicateCandidate { key } => write!(formatter, "candidate is already registered: {key:?}"),
            Self::CandidateNotFound { key } => write!(formatter, "candidate is not registered: {key:?}"),
            Self::InvalidRunTransition { from, to } => write!(formatter, "invalid ingestion run transition: {from:?} -> {to:?}"),
            Self::RunNotRunning { state } => write!(formatter, "ingestion operation requires a running run; current state is {state:?}"),
            Self::InvalidCandidateTransition { from, to } => write!(formatter, "invalid candidate transition: {from:?} -> {to:?}"),
            Self::UnplannedStage { name } => write!(formatter, "stage {name} is not in the frozen pipeline plan"),
            Self::StageOrderMismatch { expected_step, actual_step } => write!(formatter, "pipeline stage order mismatch: expected {expected_step}, got {actual_step}"),
            Self::InvalidAttemptNumber => formatter.write_str("stage attempt numbers start at one"),
            Self::InvalidStageTimeRange => formatter.write_str("stage finish time precedes its start time"),
            Self::RunHasIncompleteCandidates => formatter.write_str("run cannot complete successfully while candidates remain unsynchronized"),
            Self::EmptyTargetedReprocessing => formatter.write_str("targeted reprocessing requires at least one candidate"),
            Self::EmptySourceDelta => formatter.write_str("source delta requires at least one changed record key; use explicit full-source mode otherwise"),
            Self::UnchangedSourceVersion => formatter.write_str("source delta versions must differ"),
            Self::ReprocessingTargetMismatch => formatter.write_str("reprocessing scope does not match the source/snapshot target"),
        }
    }
}
impl std::error::Error for PipelineError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), PipelineError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(PipelineError::InvalidLabel { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateState, IngestionPipeline, IngestionRunState, PipelineError, PipelineStage,
        PipelineStageHook, PipelineStageVersion, PipelineStepVersion, ReprocessingMode,
        ReprocessingRequest, RunCompletion, SourceDelta, StageExecutionInput,
        StageExecutionOutcome,
    };
    use crate::identity::SourceId;
    use crate::ingestion::mapping::{CandidateKey, MappedCandidate};
    use crate::ingestion::raw::SourceClass;
    use crate::temporal::Instant;
    use nizaam_core::identity::OperationId;

    fn candidate_key() -> CandidateKey {
        CandidateKey::new(
            SourceId::new("source-pipeline").unwrap(),
            "record-1",
            Some("snapshot-1".to_owned()),
            0,
        )
        .unwrap()
    }

    fn mapped_candidate() -> MappedCandidate<()> {
        use crate::ingestion::mapping::MappingMetadata;
        use crate::ingestion::normalize::{NormalizationMetadata, NormalizedRecord};
        use crate::ingestion::raw::SourceRecordMetadata;

        let record = NormalizedRecord::new(
            SourceRecordMetadata::new(
                SourceId::new("source-pipeline").unwrap(),
                SourceClass::structured_data(),
                "record-1",
                Some("snapshot-1".to_owned()),
                None,
                [],
            )
            .unwrap(),
            (),
            NormalizationMetadata::new("normalize-v1", "config-v1", Instant::from_unix_seconds(1))
                .unwrap(),
        )
        .unwrap();
        MappedCandidate::from_normalized(
            record,
            0,
            (),
            MappingMetadata::new("mapping-v1", "config-v1").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn default_plan_has_fixed_core_order_and_supports_optional_hooks() {
        let base = super::PipelinePlan::phase5_default();
        let hook = PipelineStageHook::new(
            "source-enrichment",
            PipelineStage::Mapping,
            "hook-v1",
            "config-v1",
        )
        .unwrap();
        let plan = base.with_hook(hook).unwrap();
        let steps = plan.ordered_steps();
        let mapping_index = steps.iter().position(|step| matches!(step, PipelineStepVersion::Core(value) if value.stage() == PipelineStage::Mapping)).unwrap();
        assert!(
            matches!(&steps[mapping_index + 1], PipelineStepVersion::Hook(value) if value.name() == "source-enrichment")
        );
        assert_eq!(steps.first().unwrap().name(), "raw-capture");
        assert_eq!(steps.last().unwrap().name(), "index-synchronization");
    }

    #[test]
    fn run_tracks_candidate_states_separately_and_checks_transitions() {
        let pipeline = IngestionPipeline::phase5_default();
        let candidate = mapped_candidate();
        let key = candidate.key().clone();
        let mut run = pipeline
            .begin_run(
                OperationId::new("nizaam.kg.ingestion.pipeline-test").unwrap(),
                SourceId::new("source-pipeline").unwrap(),
                SourceClass::structured_data(),
                Some("snapshot-1".to_owned()),
                Instant::from_unix_seconds(1),
            )
            .unwrap();
        assert_eq!(run.state(), IngestionRunState::Created);
        assert_eq!(
            run.register_candidate(&candidate),
            Err(PipelineError::RunNotRunning {
                state: IngestionRunState::Created
            })
        );
        run.start().unwrap();
        run.register_candidate(&candidate).unwrap();
        run.transition_candidate(
            &key,
            CandidateState::Normalized,
            Instant::from_unix_seconds(2),
            "normalized",
        )
        .unwrap();
        assert!(matches!(
            run.transition_candidate(
                &key,
                CandidateState::Published,
                Instant::from_unix_seconds(3),
                "skip stages"
            ),
            Err(PipelineError::InvalidCandidateTransition { .. })
        ));
        assert_eq!(
            run.candidate(&key).unwrap().state(),
            CandidateState::Normalized
        );
        run.finish(RunCompletion::CompletedWithIssues).unwrap();
        assert_eq!(run.state(), IngestionRunState::CompletedWithIssues);
        assert_eq!(
            run.register_candidate(&candidate),
            Err(PipelineError::RunNotRunning {
                state: IngestionRunState::CompletedWithIssues
            })
        );
    }

    #[test]
    fn successful_run_cannot_finish_with_unsynchronized_candidates() {
        let pipeline = IngestionPipeline::phase5_default();
        let candidate = mapped_candidate();
        let mut run = pipeline
            .begin_run(
                OperationId::new("nizaam.kg.ingestion.pipeline-incomplete").unwrap(),
                SourceId::new("source-pipeline").unwrap(),
                SourceClass::structured_data(),
                Some("snapshot-1".to_owned()),
                Instant::from_unix_seconds(1),
            )
            .unwrap();
        run.start().unwrap();
        run.register_candidate(&candidate).unwrap();
        assert_eq!(
            run.finish(RunCompletion::Completed),
            Err(PipelineError::RunHasIncompleteCandidates)
        );
    }

    #[test]
    fn stage_execution_must_follow_the_frozen_plan() {
        let pipeline = IngestionPipeline::phase5_default();
        let candidate = mapped_candidate();
        let key = candidate.key().clone();
        let mut run = pipeline
            .begin_run(
                OperationId::new("nizaam.kg.ingestion.pipeline-order").unwrap(),
                SourceId::new("source-pipeline").unwrap(),
                SourceClass::structured_data(),
                Some("snapshot-1".to_owned()),
                Instant::from_unix_seconds(1),
            )
            .unwrap();
        run.start().unwrap();
        run.register_candidate(&candidate).unwrap();
        let steps = run.plan().ordered_steps();
        assert!(matches!(
            run.record_stage_execution(
                &key,
                StageExecutionInput::new(
                    steps[1].clone(),
                    1,
                    StageExecutionOutcome::Succeeded,
                    Instant::from_unix_seconds(2),
                    Instant::from_unix_seconds(3),
                    None,
                ),
            ),
            Err(PipelineError::StageOrderMismatch { .. })
        ));
        run.record_stage_execution(
            &key,
            StageExecutionInput::new(
                steps[0].clone(),
                1,
                StageExecutionOutcome::RetryableFailure,
                Instant::from_unix_seconds(3),
                Instant::from_unix_seconds(4),
                Some("transient raw capture failure".to_owned()),
            ),
        )
        .unwrap();
        run.record_stage_execution(
            &key,
            StageExecutionInput::new(
                steps[0].clone(),
                2,
                StageExecutionOutcome::Succeeded,
                Instant::from_unix_seconds(4),
                Instant::from_unix_seconds(5),
                None,
            ),
        )
        .unwrap();
        run.record_stage_execution(
            &key,
            StageExecutionInput::new(
                steps[1].clone(),
                1,
                StageExecutionOutcome::Succeeded,
                Instant::from_unix_seconds(5),
                Instant::from_unix_seconds(6),
                None,
            ),
        )
        .unwrap();
        assert!(matches!(
            run.record_stage_execution(
                &key,
                StageExecutionInput::new(
                    steps[0].clone(),
                    2,
                    StageExecutionOutcome::Succeeded,
                    Instant::from_unix_seconds(6),
                    Instant::from_unix_seconds(7),
                    None,
                ),
            ),
            Err(PipelineError::StageOrderMismatch { .. })
        ));
    }

    #[test]
    fn targeted_retry_and_source_delta_are_explicit_and_validated() {
        let operation = OperationId::new("nizaam.kg.ingestion.reprocess").unwrap();
        let source = SourceId::new("source-pipeline").unwrap();
        let targeted = ReprocessingRequest::new(
            operation.clone(),
            source.clone(),
            Some("snapshot-1".to_owned()),
            ReprocessingMode::TargetedCandidates([candidate_key()].into_iter().collect()),
            Instant::from_unix_seconds(4),
            "retry recovered candidate",
        )
        .unwrap();
        assert!(matches!(
            targeted.mode(),
            ReprocessingMode::TargetedCandidates(_)
        ));
        let delta = SourceDelta::new("snapshot-1", "snapshot-2", ["record-1".to_owned()]).unwrap();
        let delta_request = ReprocessingRequest::new(
            operation,
            source,
            Some("snapshot-2".to_owned()),
            ReprocessingMode::SourceDelta(delta),
            Instant::from_unix_seconds(5),
            "new source snapshot",
        )
        .unwrap();
        assert!(matches!(
            delta_request.mode(),
            ReprocessingMode::SourceDelta(_)
        ));
        assert!(SourceDelta::new("same", "same", ["record".to_owned()]).is_err());
        assert!(SourceDelta::new("v1", "v2", Vec::<String>::new()).is_err());
    }

    #[test]
    fn full_source_reprocessing_is_an_explicit_mode() {
        let request = ReprocessingRequest::new(
            OperationId::new("nizaam.kg.ingestion.full-reprocess").unwrap(),
            SourceId::new("source-pipeline").unwrap(),
            Some("snapshot-3".to_owned()),
            ReprocessingMode::FullSource,
            Instant::from_unix_seconds(7),
            "source schema changed",
        )
        .unwrap();
        assert!(matches!(request.mode(), ReprocessingMode::FullSource));
        let _unused_version =
            PipelineStageVersion::new(PipelineStage::Mapping, "v1", "config-v1").unwrap();
    }
}
