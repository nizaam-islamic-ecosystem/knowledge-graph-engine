# Scope: Nizaam Knowledge Graph Engine

The Nizaam Knowledge Graph Engine (`knowledge-graph`) is a Nizaam
infrastructure engine responsible for representing, organizing,
validating, connecting, querying, and eventually reasoning over
structured knowledge.

The Knowledge Graph is not merely a `Node + Edge` database. Its semantic
model must support entities, concepts, lexical forms, mentions,
claims/assertions, events, sources, documents, evidence, provenance,
authority, uncertainty, temporal validity, relationships, ontology,
controlled inference, and governed knowledge lifecycle.

The engine is designed as a **Rust-first, library-first engine with a
standalone executable runtime**, with a future Python ML layer connected
through gRPC. The Rust side remains authoritative for KG semantics and
execution. The Python side is a later ML subsystem and must not directly
mutate canonical KG storage.

> **Important:** The decisions in this document are initial/provisional
> implementation decisions. They are not irreversible architectural
> commitments. During implementation, a decision may be revisited when
> actual requirements, benchmarks, integration constraints, or
> implementation evidence justify doing so. Any such change must be
> explicitly discussed and the scope updated with approval.

------------------------------------------------------------------------

## 1. Mission, Scope & Responsibilities

### 1.1 Mission

The Knowledge Graph Engine provides the semantic knowledge foundation of
the Nizaam ecosystem.

``` text
Core
  ↓
Knowledge Graph
  ↓
Knowledge APIs
  ↓
Nizaam Applications
```

The KG connects meaningful knowledge rather than merely storing isolated
records.

Examples of knowledge that may eventually be represented include:

``` text
Entity
Concept
Lexical Form
Mention
Claim / Knowledge Assertion
Event
Source
Document
Passage
Evidence
Relationship
Ontology
Temporal state
Inference
```

The graph must preserve the distinction between:

``` text
observed / sourced knowledge
        ≠
curated knowledge
        ≠
inferred knowledge
        ≠
machine-generated candidate knowledge
```

### 1.2 Primary responsibilities

The KG Engine owns:

``` text
Knowledge representation
Entity architecture
Concept architecture
Lexical / word mapping
Canonical identity and entity resolution
Source and document modeling
Claims / Knowledge Assertions
Semantic types
Relationship semantics
Inverse and multi-directional relationship semantics
Graph traversal
Ontology and taxonomy
Semantic context
Evidence
Provenance and lineage
Authority / reliability metadata
Uncertainty and contradiction representation
Temporal knowledge
Controlled deterministic inference
Knowledge ingestion
Validation and curation
KG query planning
KG query execution
KG retrieval and ranking
KG storage abstraction
KG indexing abstraction
KG versioning
KG integration contracts
Future Rust ↔ Python ML boundary
```

### 1.3 What the KG does not own

The KG must not recreate Nizaam Core responsibilities:

``` text
Global Control Plane
Global routing
Global engine registry
Global scheduler
Universal transport infrastructure
Universal framing
Universal serialization infrastructure
Universal lifecycle mechanisms
Universal security infrastructure
Universal result envelope implementation
```

The KG must also not become the owner of:

``` text
Arabic morphological analysis
Arabic linguistic parsing
Application UI
Quran application workflows
Hadith application workflows
Kids application workflows
Halal application workflows
Global API gateway responsibilities
Global transaction ownership
Global infrastructure execution scheduling
```

The Arabic Engine provides linguistic interpretation; the KG connects
that information to the broader knowledge world.

The Indexing Engine provides indexing infrastructure; the KG owns the
meaning of semantic mappings and knowledge relationships.

------------------------------------------------------------------------

## 2. Architectural Position

The KG participates in Nizaam through the common infrastructure-engine
architecture:

``` text
Nizaam Control Plane
        ↓
KG Engine
        ↓
Core Engine Runtime
        ↓
Core Capability Infrastructure
        ↓
KG Capability
        ↓
KG-local Planner
        ↓
KG-local Workflow / Execution
        ↓
KG Result
```

The KG-local planner is not a second Control Plane.

``` text
Core Control Plane
→ global engine coordination

KG Query Planner
→ planning of KG-specific knowledge operations

KG Execution
→ execution of the planned KG operation
```

------------------------------------------------------------------------

## 3. Rust / Python Architecture

The engine has two planned computational parts:

``` text
KG ENGINE
├── Rust KG Core
└── Python ML Layer
```

### 3.1 Rust responsibility

Rust is authoritative for:

``` text
Knowledge model
Knowledge Assertions
Entities
Concepts
Lexical mappings
Relationships
Graph representation
Traversal
Query planning
Query execution
Validation
Entity resolution mechanics
Ingestion mechanics
Evidence
Provenance
Authority metadata
Versioning
Deterministic reasoning
KG contracts
Storage abstraction
Index abstraction
Runtime integration
```

### 3.2 Python responsibility

Python is reserved for future ML functionality:

``` text
Feature preparation
Model training
Unsupervised learning
Clustering
Embeddings
Similarity
Anomaly detection
Pattern discovery
Model evaluation
Other approved ML workloads
```

Python must not directly mutate canonical KG storage.

``` text
Rust KG
   ↓
ML-ready representation
   ↓
gRPC
   ↓
Python ML
   ↓
MLResult / derived signal
   ↓
Rust validation / governance
   ↓
candidate or derived knowledge
```

The following distinctions are mandatory:

``` text
Model
    ≠
Prediction
    ≠
Canonical Knowledge

Anomaly
    ≠
Incorrectness

Pattern
    ≠
Fact

Similarity
    ≠
Semantic Equivalence

ML Confidence
    ≠
Authority / Reliability
```

------------------------------------------------------------------------

## 4. Provisional Architectural Decisions

These decisions establish the initial implementation direction. They may
be revisited later when implementation evidence justifies a change.

### Decision 1 --- Physical storage architecture

Initial direction:

``` text
Document-oriented storage
        ↓
KG grows / workload becomes larger
        ↓
Hybrid architecture when justified
```

The logical KG model must remain independent of the physical provider.

No specific database provider is frozen by this scope.

### Decision 2 --- Inverse relationship representation

Initial direction:

``` text
Option B
```

The KG will initially use one canonical semantic relationship and derive
inverse traversal semantically.

As mapping size and traversal workload grow, the implementation may move
toward:

``` text
Option C
canonical relationship + reverse index
```

or another better mechanism discovered during implementation.

The semantic contract must support forward and inverse traversal
independently of the physical representation.

### Decision 3 --- Canonical knowledge unit

The KG will use a first-class:

``` text
KnowledgeAssertion
```

rather than treating the graph as only:

``` text
Entity + Relationship
```

Conceptually:

``` text
KnowledgeAssertion
├── subject
├── predicate
├── object
├── context
├── qualifiers
├── evidence
├── provenance
├── authority
├── confidence
├── temporal validity
└── status
```

### Decision 4 --- Initial governance

Initial implementation:

``` text
Single global approval rule
```

The initial KG does not require a full policy-based governance engine.

When LLM/ML-derived knowledge is introduced, the governance architecture
may evolve toward:

``` text
Policy-based governance
```

Accuracy and trust are prioritized over maximum automation.

Machine-generated knowledge must not bypass validation and approval.

### Decision 5 --- Rust ↔ Python communication

Initial/future bridge:

``` text
gRPC
```

The semantic boundary remains:

``` text
Rust = authoritative KG
Python = ML subsystem
```

The exact Python execution/deployment architecture remains open until
the Python phase.

### Decision 6 --- Python implementation timing

The Python layer is intentionally deferred. Initial implementation
focuses on the Rust KG. The Python ML subsystem may be implemented much
later.

### Decision 7 --- Search, evidence, ranking, and query interface

#### Search

A search index is required.

``` text
small/simple search index
        ↓
future Nizaam search infrastructure
```

#### Evidence / provenance retrieval

Evidence and provenance are required from the beginning, but the first
implementation should remain simple and extensible.

#### Ranking

Initial direction:

``` text
KG-owned ranking
```

Ownership may be moved later if actual workload analysis shows that
another engine or subsystem is more appropriate.

#### Query interface

Initial direction:

``` text
Conceptual / typed KG API
```

No query language is required initially.

A future Nizaam-specific query language remains possible.

### Decision 8 --- Knowledge mutability / versioning

Initial direction:

``` text
simple initial model
```

The implementation must preserve a path toward richer versioning and
lifecycle semantics later.

------------------------------------------------------------------------

## 5. Semantic Knowledge Model

The KG must not collapse all knowledge into a generic node.

The foundational semantic vocabulary includes:

``` text
Entity
Concept
Lexical Form
Lemma
Root
Sense
Mention
Claim
Knowledge Assertion
Event
Source
Document
Passage
Reference
Evidence
Rule
Observation
Inference
Relationship
```

The exact taxonomy may evolve during the dedicated semantic-model
phases.

### 5.1 Entity

Entities represent identifiable objects or references.

Examples:

``` text
Allah
Prophet Muhammad ﷺ
Prophet Musa عليه السلام
Makkah
Madinah
Quran
Sahih al-Bukhari
Ramadan
```

The entity architecture must support:

``` text
canonical identity
aliases
names
references
types
subtypes
disambiguation
same-entity resolution
```

### 5.2 Concept

Concepts are distinct from entities.

Examples:

``` text
sabr
taqwa
rahmah
justice
patience
worship
```

Concepts may connect to lexical forms, entities, texts, events, and
other concepts.

### 5.3 Lexical model

The KG consumes linguistic interpretation from the Arabic Engine.

``` text
Arabic word
    ↓
Lexical Form
    ↓
Lemma
    ↓
Root
    ↓
Sense
    ↓
Concept / Entity Reference
```

The KG must not duplicate the Arabic Engine's linguistic analysis
responsibilities.

### 5.4 Knowledge Assertion

The canonical semantic statement model is:

``` text
KnowledgeAssertion
```

It must be capable of representing:

``` text
subject
predicate
object
context
qualifiers
evidence
provenance
authority
confidence
temporal validity
status
```

The implementation must preserve the distinction between the assertion
itself and the relationship vocabulary used by that assertion.

------------------------------------------------------------------------

## 6. Relationship Architecture

The relationship model must support the semantic relationship families
defined by the KG architecture:

``` text
Semantic
Identity
Hierarchical
Temporal
Causal
Reference
Part-whole
Logical
Linguistic
Knowledge
```

Lexical / word mapping is handled separately as a knowledge-model
concern.

The relationship model must support:

``` text
direct relationships
inverse relationships
multi-directional traversal
relationship characteristics
relationship composition
relationship paths
filtered traversal
```

The physical representation remains an implementation concern.

------------------------------------------------------------------------

## 7. Graph Traversal

Traversal and inference are separate responsibilities.

``` text
Traversal
→ follows known graph relationships

Inference
→ derives additional knowledge
```

Ordinary traversal must not silently perform unrestricted inference.

Traversal must eventually support:

``` text
1-hop
2-hop
N-hop
path traversal
relationship chains
inverse traversal
filtered traversal
bounded traversal
context-aware traversal
```

Traversal must have protections against uncontrolled graph expansion.

------------------------------------------------------------------------

## 8. Ontology and Semantics

The ontology layer defines what kinds of knowledge objects exist and
which relationships are meaningful.

It must support concepts such as:

``` text
Entity
Concept
Source
Passage
Event
Claim
Relationship
```

and relationships such as:

``` text
is-a
instance-of
subclass-of
part-of
broader-than
narrower-than
```

The ontology layer must prevent arbitrary semantic connections from
being treated as equivalent merely because the graph can technically
store them.

Semantic context must support:

``` text
meaning
context
sense
usage
domain
disambiguation
semantic type
interpretation
```

------------------------------------------------------------------------

## 9. Evidence, Provenance, Authority and Trust

The KG must preserve why knowledge exists and where it came from.

### Evidence

Evidence answers:

``` text
Why is this assertion supported?
```

Evidence may reference:

``` text
Quran passage
Hadith
Tafsir
scholarly source
other approved source
```

### Provenance

Provenance answers:

``` text
Where did this knowledge originate?
Who added it?
Which source produced it?
Was it extracted?
Was it inferred?
Which version generated it?
What changed?
```

### Authority

Authority metadata remains separate from confidence.

Potential dimensions include:

``` text
source authority
reliability
scholarly status
authentication
verification
human review
machine extraction
curated status
```

### Uncertainty

The KG must be capable of representing:

``` text
known
unknown
uncertain
ambiguous
disputed
conflicting
```

Conflicting claims must be representable without blindly overwriting one
another.

------------------------------------------------------------------------

## 10. Temporal Knowledge

Temporal knowledge must support:

``` text
when something happened
when something was valid
historical states
dates
ranges
before / after
contemporary relationships
temporal validity of assertions
```

Temporal validity must remain separate from KG storage versioning.

------------------------------------------------------------------------

## 11. Controlled Reasoning

The KG will eventually support deterministic and controlled reasoning.

Potential mechanisms include:

``` text
rule-based reasoning
graph inference
logical inference
semantic inference
path-based inference
constraint-based inference
```

Reasoning must preserve:

``` text
observed knowledge
    ≠
inferred knowledge
```

Inferred relationships must remain distinguishable from domain-canonical
knowledge.

Reasoning should be query-aware and bounded. The KG must not
automatically execute every reasoning rule for every query.

------------------------------------------------------------------------

## 12. Knowledge Ingestion and Curation

Knowledge may enter the KG from:

``` text
Quran data
Hadith data
Tafsir data
Arabic Engine
manual curation
external datasets
future Nizaam engines
machine-generated candidates
```

The conceptual lifecycle is:

``` text
raw
 ↓
normalized
 ↓
mapped
 ↓
validated
 ↓
approved
 ↓
published
```

LLM/ML extraction must not bypass this governance path.

The ingestion architecture must support:

``` text
source adapters
normalization
mapping
validation
approval
publication
deduplication
entity resolution
conflict handling
reprocessing
```

------------------------------------------------------------------------

## 13. Entity Resolution

Entity resolution must support:

``` text
same-string detection
different-string / same-entity detection
aliases
transliteration
language variation
source-specific identifiers
duplicate detection
entity merging
entity splitting
disambiguation
```

Entity resolution must establish or propose identity without erasing
source-level provenance differences.

------------------------------------------------------------------------

## 14. Query and Retrieval Architecture

Applications interact with the KG through conceptual knowledge
capabilities rather than physical database mechanics.

The initial query categories are:

``` text
Lookup
Traversal
Semantic Retrieval
```

Conceptually:

``` text
Control Plane
    ↓
KG Capability Request
    ↓
KG Runtime
    ↓
KG Query Planner
    ↓
KG Execution
    ↓
KG Result
```

The query planner may consider:

``` text
graph pattern
traversal requirements
index requirements
filters
context
evidence constraints
authority constraints
temporal conditions
reasoning profile
ranking
```

The query layer must not expose physical storage details as part of the
semantic API.

A useful KG result should eventually be able to explain:

``` text
matched knowledge
relationship/path
evidence
provenance
authority
inference status
```

------------------------------------------------------------------------

## 15. Search and Indexing

Search indexing is separate from canonical KG storage.

``` text
KG Semantic Model
       ↓
Canonical Storage
       │
       ├── Search Index
       ├── Reverse Index
       ├── Forward Index
       └── future specialized indexes
```

Indexes are acceleration structures, not canonical knowledge.

The initial search implementation is intentionally simple. Later it may
evolve toward a Nizaam-wide search engine without requiring a redesign
of the semantic KG model.

------------------------------------------------------------------------

## 16. Storage Architecture

The storage layer is abstracted:

``` text
KG
 ↓
KG Storage Abstraction
 ├── document implementation
 ├── future hybrid implementation
 └── future specialized implementations
```

The initial physical direction is document-oriented.

The later migration path is:

``` text
Document
   ↓
larger graph / workload
   ↓
Hybrid
```

The storage layer must not leak provider-specific types into the
semantic model.

The following remain implementation decisions for later phases:

``` text
database provider
graph database provider
document database provider
RDF store
relational store
physical schema
physical partitioning
sharding
replication
cache technology
search provider
vector provider
```

------------------------------------------------------------------------

## 17. Versioning and Evolution

The KG must eventually support knowledge evolution without confusing:

``` text
knowledge version
schema version
source version
model version
storage version
```

The initial implementation should remain simple while preserving
extension points for:

``` text
knowledge revisions
supersession
retraction
migration
compatibility
historical state
change tracking
```

Model version and KG version remain separate.

------------------------------------------------------------------------

## 18. Runtime and Standalone Execution

The KG is an independently runnable Nizaam engine.

The Rust crate is library-first:

``` text
src/lib.rs
    ↓
KG implementation
```

The executable runtime is a deployment surface:

``` text
src/main.rs
    ↓
KG Engine Runtime
    ↓
KG capabilities
```

The actual startup logic must live in the engine/runtime implementation
rather than being duplicated in `main.rs`.

The final executable packaging may be refined during implementation if
actual runtime requirements justify a different arrangement.

------------------------------------------------------------------------

## 19. Module Structure

The current project scaffold is:

``` text
src/
├── lib.rs
├── main.rs
├── engine/
├── contract/
├── identity/
├── entity/
├── concept/
├── lexical/
├── assertion/
├── relationship/
├── graph/
├── source/
├── evidence/
├── provenance/
├── authority/
├── uncertainty/
├── temporal/
├── resolution/
├── ontology/
├── semantics/
├── reasoning/
├── ingestion/
├── query/
├── storage/
├── index/
├── versioning/
├── ml/
└── integration/

python/
tests/
```

Every current module contains `mod.rs`.

This is a **provisional implementation scaffold**. A source file may
later be split, merged, renamed, moved, or removed when implementation
demonstrates a better boundary.

The existence of a scaffold file does not mean that its functionality
belongs to the current phase.

------------------------------------------------------------------------

## 20. Implementation Phases

The architecture contains many conceptual sections. They are
intentionally grouped into a smaller number of implementation phases so
implementation remains manageable.

The exact internal file boundaries may change during phase
implementation.

------------------------------------------------------------------------

### Phase 0: Engine Workspace & Integration Foundation

#### Status

**Status:** Completed

Phase 0 establishes the Knowledge Graph (KG) Engine as a valid Nizaam infrastructure engine and establishes the Rust library boundary through which all later KG phases will be implemented.

Phase 0 is an **integration and runtime-foundation phase**, not a knowledge-model phase and not a persistence phase.

The objective is to make the KG Engine capable of being constructed, integrated with `nizaam-core`, registered with the Nizaam Control Plane, admitted through the Core lifecycle, exposed through a minimal capability boundary, and shut down cleanly.

The KG Engine must reuse Core infrastructure rather than recreate it.

#### Goal

```text
construct
  ↓
initialize Core-backed runtime
  ↓
start
  ↓
enter Registering
  ↓
register KG engine instance
  ↓
register Phase 0 capability
  ↓
mark Ready
  ↓
serve
  ↓
receive UniversalRequest
  ↓
Core lifecycle admission
  ↓
Core capability resolution/dispatch
  ↓
minimal KG capability
  ↓
Universal result
  ↓
drain
  ↓
Stopped
```

Phase 0 establishes:

- package/library integration;
- KG engine identity;
- Core-backed engine runtime;
- lifecycle startup/shutdown boundary;
- engine registration;
- capability registration boundary;
- Universal Request/Response integration;
- `OperationContext` / `EngineContext` propagation;
- runtime admission boundary;
- Core Control Plane integration boundary;
- Core health/readiness integration;
- minimal Phase 0 capability;
- KG-local planner extension point;
- test foundation where actual Phase 0 behavior exists.

It must not implement the KG knowledge model, storage engine, search index, Python ML, or domain ingestion.

#### 0.3 Core Ownership

`nizaam-core` is the authoritative owner of shared Nizaam infrastructure.

KG Phase 0 must use Core for:

- engine runtime and lifecycle;
- engine registration;
- Control Plane;
- capability registration and dispatch;
- universal contracts;
- operation context;
- engine context;
- request admission;
- cancellation;
- deadlines;
- execution coordination;
- health/readiness;
- security;
- logging, metrics, tracing and diagnostics;
- transport;
- draining and shutdown.

The KG Engine must not create parallel implementations of these mechanisms.

Core's current documentation exposes `EngineRuntime`, `Lifecycle`, `OperationContext`, `EngineContext`, capability registration/dispatch, request-admission errors, and Control Plane registry/membership/routing boundaries. The Core `EngineRegistry` is keyed by concrete `EngineInstanceId`, while `EngineRegistration` separately carries the logical `EngineId` and concrete instance identity.

#### 0.4 Control Plane Decision

The KG Engine uses the Core Control Plane across **Rust Phases 0–8 as required by each phase**. Control Plane integration is therefore not a Phase 0-only concern.

The KG Engine also has a KG-local planner.

The distinction is:

```text
Core Control Plane
    = ecosystem-level coordination

KG Local Planner
    = KG-domain execution planning
```

The KG local planner must not become a second global Control Plane and must not own global membership, routing, destination selection, transport, global retry, global scheduling, or global lifecycle.

#### 0.5 Library-First Decision

The KG crate is implemented as a **library first**.

`src/lib.rs` is authoritative during Phase 0.

`src/main.rs` may remain scaffolded/deferred. The final standalone executable/daemon entry point is deliberately selected later, after the complete KG library has been developed enough to establish the real runtime requirements.

The existence of `main.rs` does not mean the final process architecture belongs to Phase 0.

#### 0.6 Phase 0 Module Boundary

##### Implement in Phase 0

```text
src/lib.rs
src/engine/
├── mod.rs
├── engine.rs
├── runtime.rs
├── lifecycle.rs
└── registration.rs

src/contract/
├── mod.rs
└── <minimal Phase 0 capability contract>

src/integration/
├── mod.rs
└── core.rs
```

##### Deferred/scaffold only

```text
src/main.rs
src/integration/grpc.rs
```

gRPC is reserved for the later Rust ↔ Python integration boundary.

The future-domain directories may exist as scaffolding but receive no Phase 0 functionality:

```text
src/entity/
src/concept/
src/lexical/
src/assertion/
src/relationship/
src/graph/
src/source/
src/evidence/
src/provenance/
src/authority/
src/uncertainty/
src/temporal/
src/resolution/
src/ontology/
src/semantics/
src/reasoning/
src/ingestion/
src/query/
src/storage/
src/index/
src/versioning/
src/ml/
```

#### 0.7 Engine Identity

Preserve Core's distinction:

```text
EngineId
    = logical KG Engine identity

EngineInstanceId
    = concrete runtime instance identity
```

Multiple instances may belong to one logical KG engine.

Phase 0 must not collapse these identities into one value or create a competing identity system.

#### 0.8 Runtime and Lifecycle

`src/engine/runtime.rs` is the main Phase 0 adapter over Core's `EngineRuntime`.

It must not implement a second lifecycle state machine.

The lifecycle boundary is:

```text
Created
↓
Starting
↓
Configuring
↓
Dependencies
↓
Capabilities
↓
Registering
↓
Ready
↓
Serving
↓
Draining
↓
Stopped
```

Core remains authoritative for lifecycle validity, admission, execution coordination, context propagation, capability dispatch, cancellation, deadlines, and shutdown.

Important rules:

- `Ready` is distinct from `Serving`.
- Only `Serving` admits normal requests.
- `Draining` rejects new normal work.
- `Stopped` is terminal for the current lifecycle.
- Capability execution must never bypass lifecycle admission.

#### 0.9 Registration

The intended sequence is:

```text
Construct
↓
Start
↓
Begin Registering
↓
Register engine instance with Core
↓
Register Phase 0 capability
↓
Mark Ready
↓
Serve
```

Engine registration and capability registration remain separate.

The KG Engine uses Core's `EngineRegistry` and capability infrastructure.

It must not implement a local registry that competes with Core.

#### 0.10 Minimal Phase 0 Capability

The initial minimal capability is **Option A**.

Its purpose is only to prove the integration path:

```text
UniversalRequest
↓
Core admission
↓
capability resolution
↓
KG capability handler
↓
Universal result
```

It is not a real KG query, graph, search, ingestion, storage, or reasoning operation.

In the future, the capability boundary may evolve to Option C or another design if actual requirements justify it. Phase 0 must not over-engineer this future decision.

#### 0.11 Universal Request/Response

KG must use Core's Universal Request/Response contracts.

It must not create a second universal envelope, message protocol, or transport protocol.

The boundary is:

```text
Core UniversalRequest
↓
KG capability
↓
KG operation
↓
Core UniversalResponse
```

The current Indexing documentation confirms the intended architectural pattern: `IndexEvent` wraps/stores the existing Core `UniversalRequest` and preserves Core identities and context instead of reconstructing a second event/message/correlation protocol.

KG Phase 0 must follow the same principle.

#### 0.12 Context Propagation

Use Core's:

- `OperationContext`;
- `EngineContext`.

Do not create duplicate KG-local context types.

Conceptually:

```text
UniversalRequest
↓
OperationContext
↓
KG runtime boundary
↓
EngineContext
↓
capability execution
```

Core remains authoritative for operation/correlation context, cancellation, deadlines, and runtime context propagation.

#### 0.13 Cancellation and Deadlines

Cancellation and deadlines are Core concerns.

Phase 0 must preserve Core's cancellation and deadline mechanisms through the KG request boundary.

No KG-specific cancellation protocol should replace the Core operation cancellation model.

#### 0.14 Health and Readiness

Phase 0 health/readiness means **engine runtime health and readiness**.

It does not mean KG data completeness.

Use Core's health/readiness infrastructure as-is.

No database is required for engine health.

Do not make health depend on:

- number of mappings;
- graph contents;
- persistence state;
- ingestion completion;
- search index availability;
- knowledge freshness.

Runtime readiness should correspond to the Core-backed engine having completed the required registration boundary and being able to serve.

#### 0.15 `.nizaam` Filesystem Persistence

The initial Nizaam filesystem location is:

```text
~/.nizaam
```

However, **Phase 0 does not implement the final KG filesystem persistence layer**.

`.nizaam` persistence will be implemented properly **phase by phase** as logical objects, query requirements, storage contracts, versioning, and performance requirements become concrete.

Therefore:

```text
Phase 0
    = runtime/integration

Later phases
    = logical KG behavior

Storage phase
    = physical persistence architecture and implementation
```

The presence of `~/.nizaam` must not be interpreted as a requirement to build a database or persistence engine in Phase 0.

Physical storage remains a later concern.

#### 0.16 Error Strategy

Do not create a giant speculative KG error hierarchy in Phase 0.

Use Core error types where they already own the relevant infrastructure boundary and introduce KG-specific errors only when the corresponding KG behavior is actually implemented.

The intended evolution is:

```text
Phase 0
    → runtime/setup/registration integration errors

Phase 1+
    → phase-specific semantic errors

Later phases
    → domain-specific validation, query, storage, reasoning, etc.
```

The KG error boundary must compose with Core's global error infrastructure rather than replacing it.

#### 0.17 Local Planner Extension Point

Phase 0 creates only a clean extension point for the KG local planner.

There is **no fake planner implementation**.

Future shape:

```text
Core Control Plane
↓
KG capability request
↓
KG Engine Runtime
↓
KG Local Planner
↓
KG Execution
```

Later planner responsibilities may include graph-pattern planning, traversal/index requirements, relationship lookup, evidence filtering, temporal constraints, and reasoning profiles.

None of those are Phase 0 behavior.

#### 0.18 gRPC / Python Boundary

The Rust ↔ Python bridge is **gRPC**, but its implementation is deferred.

Python is a separate scope and runtime.

Rust remains authoritative for:

- graph representation;
- graph operations;
- traversal;
- query execution;
- deterministic reasoning;
- validation;
- entity-resolution mechanics;
- ingestion mechanics;
- provenance;
- versioning;
- contracts;
- runtime integration.

Python will later own ML-oriented capabilities such as feature preparation, training, embeddings, similarity, clustering, anomaly detection, pattern discovery, and evaluation.

Python must not directly mutate KG storage.

Rust Phase 9 is deferred until the Python foundation is ready.

Therefore `src/integration/grpc.rs` is not a Phase 0 implementation target.

##### Testing Strategy

Testing is behavior-driven, not file-driven.

If Phase 0 contains only scaffolding, do not create artificial tests.

If Phase 0 contains actual runtime/integration behavior, test that behavior.

Negative tests belong to **every phase**, not only Phase 0.

##### Positive Phase 0 coverage

Where the implementation exists, verify:

- engine construction;
- preservation of `EngineId`;
- preservation of `EngineInstanceId`;
- valid Core lifecycle progression;
- separate `Ready` and `Serving`;
- registration through Core;
- capability registration through Core;
- readiness only after required registration;
- request admission only while `Serving`;
- Universal Request/Response boundary;
- `OperationContext` propagation;
- `EngineContext` construction/propagation;
- minimal capability invocation;
- clean drain;
- clean shutdown to `Stopped`;
- Core Control Plane integration without a duplicate local Control Plane.

##### Negative Phase 0 coverage

Where the APIs expose these behaviors, verify rejection of:

- invalid lifecycle transitions;
- engine registration outside `Registering`;
- capability registration outside `Registering`;
- capability owned by a different engine;
- readiness before required registration;
- normal requests during `Draining`;
- normal requests after `Stopped`;
- wrong engine targeting;
- wrong concrete instance targeting;
- invalid request/capability contracts;
- invalid shutdown/lifecycle operations.

Do not test future knowledge semantics in Phase 0.

##### Non-Goals

Phase 0 does not implement:

- entities;
- concepts;
- lexical forms;
- knowledge assertions;
- relationships;
- inverse relationships;
- graph storage;
- traversal;
- ontology;
- taxonomy;
- semantic interpretation;
- entity resolution;
- evidence;
- provenance;
- authority;
- uncertainty;
- temporal knowledge;
- inference/reasoning;
- ingestion;
- search;
- query execution;
- index algorithms;
- physical storage;
- durable KG mapping persistence;
- Python ML;
- embeddings;
- gRPC implementation;
- final standalone executable architecture.

#### 0.21 Relationship to Later Phases

```text
Phase 0
Engine Workspace & Integration Foundation
↓
Phase 1
Knowledge Identity & Core Semantic Objects
↓
Phase 2
Knowledge Assertion & Relationship Model
↓
Phase 3
Ontology, Semantics & Entity Resolution
↓
Phase 4
Evidence, Provenance, Authority, Uncertainty & Temporal Knowledge
↓
Phase 5
Knowledge Ingestion, Validation & Governance
↓
Phase 6
Query, Traversal, Search & Retrieval
↓
Phase 7
Storage, Versioning & Performance Architecture
↓
Phase 8
Controlled Reasoning
↓
Phase 9
Rust ↔ Python Integration
↓
Phase 10
Nizaam Integration, Conformance & Hardening
```

The Python scope is separate. Rust Phase 9 is intentionally deferred until the Python foundation is ready.

#### 0.23 Mandatory Implementation Rules

1. **Core owns shared infrastructure.**
2. **KG owns only KG-specific behavior.**
3. **Control Plane remains available across later KG phases.**
4. **The local planner is distinct from Control Plane.**
5. **Use the minimal Option A capability initially.**
6. **Library first; final executable later.**
7. **Health/readiness is real-time engine runtime state.**
8. **`.nizaam` persistence is implemented phase by phase, not all in Phase 0.**
9. **Errors grow with actual phase behavior.**
10. **Use Core `OperationContext` and `EngineContext`.**
11. **Use Core universal contracts.**
12. **Do not create duplicate message/event/operation/correlation identities.**
13. **Do not implement future-domain behavior early.**
14. **Test behavior, not filenames.**
15. **Every phase owns its own negative tests.**
16. **A scaffold is not an implementation.**
17. **Do not claim future functionality merely because a module exists.**

#### Phase 0 Implementation Record

Phase 0 has been implemented as the Core-backed KG Engine foundation. The following
items are now present in the implementation:

- [x] Rust library/package boundary established in `src/lib.rs`.
- [x] `nizaam-core` and `nizaam-indexing` are declared as crate dependencies; no
  indexing-specific runtime behavior is introduced because that is outside the
  Phase 0 scope.
- [x] `KgEngine` facade composes the Core-backed runtime, Control Plane
  registration boundary, and minimal capability boundary.
- [x] Core `EngineId` and `EngineInstanceId` remain distinct and are preserved
  across the engine lifecycle.
- [x] Core `EngineRuntime` is used as the authoritative lifecycle and admission
  mechanism; no second KG lifecycle state machine was introduced.
- [x] The Phase 0 lifecycle path is implemented through Core:
  `Created → Starting → Configuring → Dependencies → Capabilities →
  Registering → Ready → Serving → Draining → Stopped`.
- [x] Engine registration is performed through Core `EngineRegistration` and
  `EngineRegistry`, with no competing KG-local engine registry.
- [x] Phase 0 capability ownership and advertisement use Core's capability
  definition/registration model.
- [x] The minimal Option A capability contract is implemented as an opaque
  integration probe rather than a knowledge/query capability.
- [x] Universal requests are accepted through Core's `UniversalRequest` contract
  and adapted into Core `CapabilityInvocation` values.
- [x] Request handling passes through Core lifecycle admission before capability
  dispatch.
- [x] Logical-engine and concrete-instance target mismatches are rejected at the
  KG engine boundary before capability execution.
- [x] Capability lookup and dispatch use Core's canonical capability dispatcher.
- [x] `OperationContext` is preserved from the Universal Request and converted
  into Core `EngineContext` without introducing KG-local context types.
- [x] Capability results remain Core-owned `CapabilityDispatchResult` values;
  no parallel KG universal result envelope was introduced.
- [x] Graceful drain and shutdown delegate to Core and terminate in Core's
  `Stopped` state.
- [x] KG-specific setup/request errors are limited to actual Phase 0 boundary
  concerns and compose with Core error types.
- [x] A KG-local planner extension point exists as an architectural seam only;
  no fake planner or query implementation was introduced.
- [x] Deferred integration surfaces (`grpc`, Arabic, indexing-specific adapter
  files) remain scaffolding with no premature Phase 0 functionality.
- [x] Future knowledge-domain modules remain scaffolding only; Phase 0 does not
  implement entities, concepts, assertions, relationships, storage, search,
  traversal, ingestion, reasoning, or ML.
- [x] Level 1 unit tests cover the implemented lifecycle, registration,
  capability, runtime, and facade behavior.
- [x] Level 2/3 integration and conformance tests cover the complete Core-backed
  Phase 0 path, including positive and negative lifecycle, registration,
  targeting, request, context, dispatch, drain, and shutdown behavior.
- [x] The final Phase 0 test suite contains 33 unit tests and 35 Level 3 tests
  in the supplied implementation snapshot.

#### Phase 0 Decisions Recorded During Implementation

- **[x] Core remains authoritative.** KG adapts to Core and does not recreate
  runtime, lifecycle, registry, Control Plane, capability dispatch, or
  universal contracts.
- **[x] Minimal capability remains Option A.** Phase 0 proves the admission →
  resolution → handler → result path without introducing real KG semantics.
- **[x] Library-first architecture remains frozen.** `src/lib.rs` is the
  authoritative Phase 0 boundary; the final standalone executable remains
  deferred.
- **[x] Engine registration and capability registration remain separate
  concerns.** Core registration metadata advertises the Phase 0 capability,
  while the Core capability registry owns executable dispatch.
- **[x] Target validation is enforced at the KG boundary.** Because Core's
  request-admission error covers lifecycle admission, the facade exposes
  explicit target-engine and target-instance mismatch errors rather than
  inventing a new Core error mechanism.
- **[x] No knowledge-domain behavior is pulled forward.** The planner,
  knowledge model, storage, search, ingestion, reasoning, and Python/gRPC
  functionality remain later-phase concerns.
- **[x] Testing remains behavior-driven.** Tests verify actual Phase 0 behavior
  and Core ownership boundaries rather than merely asserting that scaffold
  files exist.

#### Completion Criteria

Phase 0 is complete when the KG Engine can:

1. exist as a reusable Rust library crate;
2. construct a KG engine with distinct `EngineId` and `EngineInstanceId`;
3. initialize the Core-backed runtime;
4. start through the Core lifecycle;
5. enter `Registering`;
6. register the engine instance through Core;
7. register the minimal Phase 0 capability through Core;
8. reach `Ready` only after required registration;
9. enter `Serving`;
10. receive a Core `UniversalRequest`;
11. pass Core lifecycle admission;
12. resolve and dispatch the Phase 0 capability through Core;
13. preserve Core operation/context semantics;
14. produce a Core-compatible universal result;
15. use Core health/readiness infrastructure;
16. enter `Draining`;
17. reject new normal work while draining;
18. shut down cleanly to `Stopped`;
19. expose a clean KG-local planner extension point;
20. avoid implementing a second Control Plane;
21. avoid implementing physical KG persistence;
22. avoid implementing future knowledge-domain functionality;
23. avoid implementing Python/gRPC integration;
24. contain tests only for behavior actually present in Phase 0.


#### Verification Checklist
- [x] The phase goal and approved scope are satisfied.
- [x] The implementation and module boundaries match the approved architecture.
- [x] Positive and negative behavior is covered by appropriate tests.
- [x] Previously verified phases remain intact and regression-safe.
- [x] Required verification and quality checks pass before completion is declared.
- [x] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

> **The Knowledge Graph Engine becomes a first-class Nizaam engine by composing with Core, not by recreating Core.**

The Phase 0 boundary is therefore:

```text
                     NIZAAM CORE
┌───────────────────────────────────────────────────────────┐
│ Engine Runtime                                             │
│ Lifecycle                                                  │
│ Request Admission                                          │
│ Capability Registry / Dispatch                             │
│ Universal Contracts                                        │
│ OperationContext / EngineContext                           │
│ Control Plane                                               │
│ Membership / Provider / Routing                            │
│ Security                                                    │
│ Health / Readiness                                          │
│ Observability                                               │
│ Transport                                                   │
│ Shutdown / Draining                                         │
└──────────────────────────────┬────────────────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │     KG ENGINE       │
                    │                     │
                    │ Engine facade       │
                    │ Runtime adapter     │
                    │ Registration        │
                    │ Minimal capability  │
                    │ Local planner seam  │
                    └──────────┬──────────┘
                               │
                               ▼
                       Future KG phases
```

Phase 0 answers:

> **Can the KG Engine exist correctly inside Nizaam?**

Later phases answer:

> **What knowledge can the KG represent, validate, store, query, retrieve, reason over, and integrate?**

The first question must be solved without prematurely implementing the second.

------------------------------------------------------------------------

### Phase 1: Knowledge Identity & Core Semantic Objects

#### Status

**Completed**

#### Goal

Establish the foundational identity system and semantic-object vocabulary of the Knowledge Graph Engine without implementing later-phase relationship, resolution, ontology, evidence, storage, indexing, query, reasoning, or Python functionality.

Phase 1 establishes the semantic objects that later phases will connect and operate on.

> **The KG must preserve distinct semantic identities rather than collapsing all knowledge into a generic node.**

---

#### 1. Phase 1 Architectural Position

The KG is built on top of `nizaam-core`.

Core already owns the common Nizaam identity infrastructure. Therefore KG must reuse Core's identity mechanism rather than creating another ID-generation system.

```text
Nizaam Core
    │
    └── identity! macro
            │
            └── generate()
                    │
                    └── KG semantic IDs
```

The KG does **not** implement:

```text
custom SHA generation
custom hash generation
custom UUID generation
custom ULID generation
custom generic identity generator
duplicate Core identity infrastructure
```

This preserves the purpose of having Core established before the engines.

---

#### 2. Identity Strategy

##### 2.1 Core identity macro is authoritative

All Phase 1 KG semantic identity types must be created using the Core `identity!` macro.

Conceptually:

```text
identity!
    │
    ├── EntityId
    ├── ConceptId
    ├── SourceId
    ├── ReferenceId
    ├── LexicalFormId
    ├── MentionId
    └── KnowledgeAssertionId
```

The macro supplies the identity behavior and `generate()` mechanism.

KG therefore owns the **meaning and type of the identity**, while Core owns the **identity-generation mechanism**.

The KG must not reproduce Core's SHA-based generation logic locally.

##### 2.2 Phase-specific identity creation

Only identities required by Phase 1 are introduced in Phase 1.

Future semantic objects receive their identities in their respective phases.

```text
Phase 1
    EntityId
    ConceptId
    SourceId
    ReferenceId
    LexicalFormId
    MentionId
    KnowledgeAssertionId

Later phases
    DocumentId
    PassageId
    EventId
    ObservationId
    InferenceId
    ...
```

No generic “all future KG identity” framework is required.

---

#### 3. Typed Identity Model

Phase 1 uses a shared conceptual identity foundation with strongly typed semantic IDs.

```text
                 KG Identity Foundation
                         │
             shared identity behavior
                         │
        ┌────────────────┼────────────────┐
        │                │                │
    EntityId         ConceptId        SourceId
        │                │                │
    ReferenceId     MentionId      LexicalFormId
                         │
                  KnowledgeAssertionId
```

The underlying generation mechanism comes from Core's `identity!` macro.

The KG does not introduce a second typed-identity framework around it.

The semantic types remain distinct:

```text
EntityId
    ≠ ConceptId
    ≠ SourceId
    ≠ ReferenceId
    ≠ MentionId
    ≠ LexicalFormId
    ≠ KnowledgeAssertionId
```

even though all are generated through the same Core identity mechanism.

---

#### 4. Phase 1 Semantic Objects

Phase 1 establishes the identity-bearing semantic foundation for:

```text
Entity
Concept
Source
Reference
Lexical Form
Mention
```

and establishes the identity boundary for:

```text
KnowledgeAssertion
```

The actual assertion structure belongs to Phase 2.

The KG therefore preserves distinctions such as:

```text
Entity      ≠ Concept
Entity      ≠ Mention
Entity      ≠ Lexical Form
Concept     ≠ Lexical Form
Source      ≠ Reference
Mention     ≠ Entity
KnowledgeAssertion ≠ Relationship implementation
```

---

#### 5. Entity Identity

An `Entity` represents an identifiable referent in the KG knowledge universe.

Examples include:

```text
Allah
Prophet Muhammad ﷺ
Musa عليه السلام
Makkah
Madinah
Quran
Sahih al-Bukhari
Ramadan
Masjid al-Haram
```

An entity is not required to be a physical object.

Phase 1 establishes the entity's stable semantic identity and only the minimal representation required by later phases.

Phase 1 does **not** implement:

```text
entity resolution
same-entity detection
candidate ranking
disambiguation algorithms
ontology classification
relationship traversal
evidence evaluation
```

Those belong to later phases.

---

#### 6. Concept Identity

A `Concept` represents an abstract semantic idea.

Examples:

```text
sabr
taqwa
rahmah
justice
patience
worship
```

Concept identity remains distinct from lexical representation.

For example:

```text
"صبر"
    │
    ▼
lexical representation
    │
    ▼
sense
    │
    ▼
Concept
```

The string itself is not automatically the concept.

Phase 1 establishes concept identity but does not implement the semantic relationship between lexical objects and concepts.

---

#### 7. Source Identity

Phase 1 introduces `SourceId` and the minimal identity-bearing source representation required by the semantic model.

Phase 1 does **not** implement the full source/document/provenance model.

The following remain later-phase responsibilities:

```text
authority
reliability
scholarly status
provenance
lineage
evidence
document structure
passage structure
ingestion governance
```

Therefore:

```text
Phase 1
    Source identity
        ↓
Later phases
    Source semantics + provenance + evidence + ingestion
```

---

#### 8. Reference Identity

A `Reference` is a generic semantic reference object.

It is not restricted to one specific source format and is not implemented as a special-purpose Quran/Hadith/document locator in Phase 1.

Its purpose is to provide an identity-bearing semantic reference boundary that later phases can use when connecting knowledge objects to other objects or external/source representations.

Phase 1 does not implement:

```text
reference resolution
reference hydration
source lookup
document lookup
passage lookup
external API lookup
```

The reference remains a semantic object, not a storage or retrieval mechanism.

---

#### 9. Names, Aliases and Mentions

##### 9.1 Name

A name is a representation used to identify or describe an entity.

Names belong to the entity representation model.

##### 9.2 Alias

Aliases are alternative names/representations of the same entity.

Aliases are **not separate semantic objects in Phase 1**.

They are represented as part of the same entity/name representation structure.

Conceptually:

```text
Entity
├── identity
├── name
└── aliases
```

The implementation must not create a separate `AliasId` merely for aliases.

##### 9.3 Mention

A mention is an occurrence of a representation in a source/context.

For example:

```text
Source passage
    │
    └── "محمد"
            │
            ▼
         Mention
            │
            ▼
       Entity reference
```

Therefore:

```text
Mention ≠ Entity
```

A mention may later participate in entity resolution, but Phase 1 does not implement that resolution.

---

#### 10. Cross-Language Representation

The KG must be capable of representing multiple language forms without making language-specific linguistic analysis its responsibility.

Conceptually:

```text
Entity
   │
   ├── Arabic representation
   ├── English representation
   ├── Urdu representation
   └── other representations
```

The exact linguistic analysis remains outside KG.

```text
KG
    = stores/connects linguistic representations

Arabic Engine
    = performs Arabic linguistic analysis
```

Phase 1 therefore must not recreate Arabic morphology, syntax, stemming, root extraction, or other Arabic-engine behavior.

Language metadata may be required by the representation model, but the exact language-contract representation should remain aligned with the future Arabic Engine boundary rather than inventing a second Arabic language system inside KG.

---

#### 11. Lexical Foundation

The architecture contains lexical concepts because lexical representation can be important when mapping source representations to KG semantic identities.

However, the KG is **not** the Arabic Engine.

Therefore Phase 1 only establishes the minimum lexical identity foundation required by the KG semantic model.

The conceptual vocabulary remains:

```text
Lexical Form
Lemma
Root
Sense
```

but Phase 1 must not implement Arabic linguistic operations.

The boundary is:

```text
Arabic Engine
    │
    ├── linguistic analysis
    ├── morphology
    ├── grammatical processing
    └── linguistic interpretation
             │
             ▼
Knowledge Graph
    │
    └── represents/connects resulting semantic objects
```

Phase 1 therefore implements only the lexical identity boundary that is actually required.

It does not implement:

```text
Arabic morphology
Arabic stemming
Arabic root extraction
Arabic grammatical analysis
Arabic parsing
Arabic linguistic inference
```

The lexical module must not become a second Arabic Engine.

If implementation demonstrates that a lexical scaffold is unnecessary for the Phase 1 identity boundary, the scaffold may remain deferred without being given artificial functionality.

---

#### 12. KnowledgeAssertion Identity Boundary

Phase 1 defines:

```text
KnowledgeAssertionId
```

only.

It does **not** create:

```text
struct KnowledgeAssertion {
    ...
}
```

or an empty assertion structure.

The actual `KnowledgeAssertion` model belongs to Phase 2.

```text
Phase 1
    KnowledgeAssertionId
          │
          ▼
Phase 2
    KnowledgeAssertion
    ├── subject
    ├── predicate
    ├── object
    ├── context
    ├── qualifiers
    ├── status
    └── ...
```

This prevents Phase 1 from creating a second or incomplete assertion system.

---

#### 13. Indexing Integration Boundary

The KG will use the Indexing Engine's `IndexAssignedId` when KG mappings need to refer to an indexed source object.

The semantic distinction is:

```text
KG identity
    ↓
identifies a KG semantic object

IndexAssignedId
    ↓
identifies the source object identity assigned by Indexing
```

`IndexAssignedId` is therefore **not** a replacement for:

```text
EntityId
ConceptId
SourceId
MentionId
...
```

and KG must not generate its own duplicate version of `IndexAssignedId`.

The existing Indexing contract defines:

```text
IndexId
    = Index Assignment Operation identity

IndexAssignedId
    = assigned source-object identity
```

Therefore KG consumes `IndexAssignedId` according to the Indexing contract.

The Indexing engine remains responsible for generating and owning that identity.

KG remains responsible for the semantic mapping that may eventually connect that indexed identity to KG knowledge.

Conceptually:

```text
Source object
     │
     ▼
Indexing Engine
     │
     └── IndexAssignedId
              │
              ▼
        KG mapping layer
              │
              ▼
        KG semantic identity
```

This does not mean Phase 1 implements the complete mapping system.

The actual mapping semantics belong to later relationship/mapping phases.

---

#### 14. Mapping Boundary

Phase 1 does not implement a universal generic mapping abstraction.

In particular, Phase 1 must not create a generic:

```text
MAPS_TO
RELATED_TO
Mapping<A, B>
```

system simply to connect the newly created identity types.

The architecture distinguishes mapping dimensions/families from individual semantic predicates and from physical index families.

Therefore Phase 1 establishes identities only.

Later phases determine which mappings are semantically valid.

---

#### 15. Persistence Boundary

Phase 1 does not persist KG operations or semantic objects directly into `~/.nizaam`.

The long-term persistence location remains:

```text
~/.nizaam
```

but physical persistence is intentionally deferred until the later storage phase after the KG's semantic, relationship, query, indexing, reasoning and other requirements are known.

The intended sequence is:

```text
Phase 1
    identity + semantic objects
        ↓
Phase 2
    assertions + relationships
        ↓
Phase 3
    ontology + semantics + resolution
        ↓
Phase 4
    evidence + provenance + authority + uncertainty + temporal
        ↓
Phase 5
    ingestion + governance
        ↓
Phase 6
    query + traversal + retrieval
        ↓
Phase 7
    storage + versioning + performance
        ↓
Phase 8
    controlled reasoning
        ↓
~/.nizaam operational persistence
```

---

#### 16. Phase 1 Module Scope

The current scaffold contains:

```text
src/
├── identity/
├── entity/
├── concept/
├── lexical/
├── source/
└── assertion/
```

Phase 1 uses these modules only where they contain Phase 1 responsibilities.

##### `src/identity/`

Responsible for Phase 1 KG identity types generated through Core's `identity!` macro.

No custom identity-generation implementation.

##### `src/entity/`

Responsible for:

```text
Entity
Name
Aliases
Mention boundary
```

as defined by the Phase 1 semantic model.

No entity-resolution algorithm.

##### `src/concept/`

Responsible for the foundational `Concept` identity and minimal semantic representation required by Phase 1.

No ontology or concept hierarchy.

##### `src/lexical/`

Only retained if the Phase 1 semantic identity model requires lexical representation.

If retained, it contains only the identity-level lexical foundation.

It must not implement Arabic linguistic processing.

No lexical mapping algorithm is implemented in Phase 1.

##### `src/source/`

Responsible only for source identity and the minimal source semantic boundary.

No provenance, authority, document ingestion, or storage implementation.

##### `src/assertion/`

Responsible only for the `KnowledgeAssertionId` identity boundary.

No assertion structure and no relationship semantics are implemented in Phase 1.

---

#### Non-Goals

Phase 1 must not implement:

```text
entity resolution
candidate generation
candidate ranking
disambiguation
ontology
taxonomy
relationship semantics
KnowledgeAssertion structure
predicate system
inverse relationships
graph traversal
graph algorithms
evidence
provenance
authority
reliability
scholarly status
uncertainty
confidence evaluation
contradiction handling
temporal reasoning
ingestion pipeline
validation/approval workflow
query planning
query execution
search
retrieval
index construction
Indexing algorithms
physical storage
database integration
~/.nizaam KG persistence
versioning
migration
reasoning
inference
Python ML
gRPC Python integration
Arabic linguistic processing
```

---

#### Testing Strategy

Tests are added only for logic actually implemented in Phase 1.

If a module contains only declarations/exports with no meaningful behavior, it does not require artificial tests merely because the file exists.

Where behavior exists, tests should verify at minimum:

```text
Core identity macro is used
generated IDs are non-empty
generated IDs are distinct across generated instances where applicable
typed identity types remain distinct
semantic objects preserve their identity
Entity ≠ Concept
Entity ≠ Mention
Source ≠ Reference
KnowledgeAssertionId exists independently from assertion semantics
aliases remain part of entity representation
IndexAssignedId remains an external Indexing-owned identity
```

Negative tests should be added wherever Phase 1 behavior makes the relevant architectural boundary observable.

No tests should assert the exact generated value of a Core-generated identity.

The test suite must verify identity behavior and semantic separation rather than internal SHA output.

---

#### Phase 1 Implementation Record

The Phase 1 implementation was completed against the approved scope. The following
items are now present in the implementation:

- [x] Core-backed semantic identity foundation is implemented for the seven Phase 1 IDs:
  `EntityId`, `ConceptId`, `SourceId`, `ReferenceId`, `LexicalFormId`, `MentionId`, and
  `KnowledgeAssertionId`.
- [x] KG identity types reuse the Core `identity!` mechanism; no local SHA-256, hash,
  UUID, ULID, or generic identity generator was introduced.
- [x] `src/identity/mod.rs` wires and re-exports the seven Phase 1 identity types.
- [x] `src/identity/object.rs` remains inspect-only and does not introduce an automatic
  `ObjectId` in Phase 1.
- [x] `Entity` is implemented as a minimal identity-bearing representation with a
  primary `Name` and zero or more aliases.
- [x] `Name` preserves a value plus opaque language metadata without introducing a
  linguistic subsystem.
- [x] `Alias` remains part of the Entity representation boundary and does not receive
  a separate `AliasId`.
- [x] `Mention` is implemented as a distinct occurrence-level object with `MentionId`
  and preserved representation text.
- [x] `Entity` Level 1 and Level 2 tests cover identity/name/alias preservation,
  multilingual representation, alias semantics, and Entity/Mention distinction.
- [x] `Concept` is implemented as a minimal identity-bearing object with an opaque
  semantic representation.
- [x] Concept relationship and type scaffolds remain deferred and contain no
  Phase 1 relationship/ontology logic.
- [x] `Source` is implemented as a minimal identity-bearing object with opaque
  identifying metadata.
- [x] `Reference` is implemented with `ReferenceId` plus an opaque reference value.
- [x] Collection, document, and passage source scaffolds remain deferred without
  introducing lookup, parsing, or persistence behavior.
- [x] The lexical module establishes the `LexicalFormId` boundary only; the concrete
  lexical object, lemma, root, sense, and mapping behavior remain deferred.
- [x] The assertion module establishes `KnowledgeAssertionId` only; no
  `KnowledgeAssertion` structure or assertion-component semantics were introduced.
- [x] Phase 1 semantic module roots expose only functionality actually implemented in
  the current phase.
- [x] The Core integration boundary from Phase 0 remains intact and Core continues to
  own runtime, lifecycle, request admission, context, capability dispatch, registration,
  Control Plane, and universal contract infrastructure.
- [x] The Indexing integration boundary remains non-owning: `IndexAssignedId` is treated
  as an external Indexing-owned identity and no duplicate KG assignment system is added.
- [x] The test suite contains repository-level Phase 1 tests for identity, entity,
  concept, source, lexical, assertion, and cross-module conformance boundaries.
- [x] The existing Phase 0 conformance coverage was retained while Phase 1 conformance
  coverage was added.
- [x] Comment-only Phase 1 files do not receive artificial executable tests.

#### Phase 1 Decisions Recorded During Implementation

- **[x] Core identity ownership is preserved.** The KG defines the semantic identity
  types, while Core remains the sole owner of identity-generation behavior through
  `identity!`.
- **[x] `Entity`, `Concept`, and `Source` use intentionally minimal representations.**
  No later ontology, relationship, provenance, evidence, or resolution semantics were
  pulled forward simply because the scaffold already existed.
- **[x] `Name` language metadata remains opaque.** The KG preserves language markers
  without becoming responsible for Arabic or other linguistic analysis.
- **[x] Aliases are representation data, not independent semantic identities.**
  No `AliasId` was introduced.
- **[x] `Reference` uses the selected `ReferenceId + opaque reference value` model.**
  Reference resolution and source/document/passage lookup remain deferred.
- **[x] Lexical Phase 1 remains identity-only.** `LexicalFormId` is established, while
  concrete lexical semantics remain deferred so the KG does not duplicate the Arabic
  Engine.
- **[x] Assertion Phase 1 remains identity-only.** `KnowledgeAssertionId` exists, but
  `KnowledgeAssertion` is deferred to Phase 2.
- **[x] Core-backed integration is preserved.** Phase 1 does not create replacement
  runtime, lifecycle, registry, capability, Control Plane, or universal request systems.
- **[x] Indexing remains the owner of `IndexAssignedId`.** KG does not generate or
  redefine that identity.
- **[x] Module implementation files were named to avoid Rust module-inception warnings.**
  The Entity, Concept, and Source implementation models use `model.rs` under their
  corresponding module roots rather than `entity/entity.rs`, `concept/concept.rs`, or
  `source/source.rs`.
- **[x] Deferred/comment-only files remain deliberately minimal.** Their existence is
  preserved as scaffold structure without creating placeholder domain logic.
- **[x] Testing remains behavior-driven.** Executable tests were added where Phase 1
  behavior exists; lexical and assertion repository test files remain comment-only
  because those modules have no Phase 1 semantic behavior beyond their identity
  boundaries.

#### Completion Criteria

Phase 1 is complete when:

```text
Core identity infrastructure
        │
        ▼
Phase 1 KG identity types
        │
        ├── EntityId
        ├── ConceptId
        ├── SourceId
        ├── ReferenceId
        ├── LexicalFormId
        ├── MentionId
        └── KnowledgeAssertionId
        │
        ▼
Foundational semantic objects
        │
        ├── Entity
        ├── Concept
        ├── Source
        ├── Reference
        ├── lexical identity boundary
        └── Mention
        │
        ▼
Phase 2-ready semantic foundation
```

The implementation must demonstrate that:

1. KG semantic identities use Core's `identity!` infrastructure.
2. KG does not duplicate Core identity-generation logic.
3. Phase 1 IDs are strongly typed and semantically distinct.
4. Entity, Concept, Source, Reference and Mention remain distinct concepts.
5. Names and aliases remain part of the entity representation model rather than becoming unnecessary independent identity objects.
6. `KnowledgeAssertionId` exists without prematurely implementing `KnowledgeAssertion`.
7. Lexical foundations do not duplicate Arabic Engine functionality.
8. `IndexAssignedId` remains Indexing-owned and is consumed as an external assigned source-object identity.
9. No physical KG persistence is introduced.
10. No later-phase relationship, ontology, resolution, evidence, query, storage, reasoning, or Python functionality is pulled into Phase 1.

---


#### Verification Checklist
- [x] The phase goal and approved scope are satisfied.
- [x] The implementation and module boundaries match the approved architecture.
- [x] Positive and negative behavior is covered by appropriate tests.
- [x] Previously verified phases remain intact and regression-safe.
- [x] Required verification and quality checks pass before completion is declared.
- [x] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

> **Phase 1 establishes who and what a KG object is, not how all KG knowledge relates, is stored, retrieved, resolved, or reasoned about.**

```text
                    NIZAAM CORE
                        │
                identity! macro
                        │
                        ▼
                 KG Identity Layer
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
      Entity          Concept          Source
        │               │                │
        ├── Name        │                └── Reference
        ├── Aliases     │
        └── Mention     │
                        │
                  Lexical foundation
                        │
                        ▼
               KnowledgeAssertionId
                        │
                        ▼
                ─── Phase 2 ───
             Assertions + Relations
```

Phase 1 deliberately stops at this boundary.

------------------------------------------------------------------------

### Phase 2: Knowledge Assertion & Relationship Model

#### Status

**In Progress**

> These are the current development decisions for Phase 2. They are not
> irreversible architectural commitments. They may be revisited if
> implementation or later-phase requirements demonstrate that a change
> is necessary.

------------------------------------------------------------------------

#### Goal

Phase 2 establishes the central semantic assertion and relationship
layer of the Knowledge Graph Engine.

The current Rust scope defines Phase 2 around:

-   `KnowledgeAssertion`
-   `subject`
-   `predicate`
-   `object`
-   `context`
-   `qualifiers`
-   `status`
-   relationship predicates
-   relationship direction
-   inverse relationships
-   relationship characteristics
-   relationship families

Planned modules:

``` text
src/assertion/
src/relationship/
src/graph/
```

Phase 2 is complete when the KG can represent meaningful assertions and
relationships while preserving the distinction between:

``` text
assertion
relationship semantics
graph structure
```

and can traverse canonical and inverse relationships without requiring
physical duplication of every inverse edge.

------------------------------------------------------------------------

#### 2. Architectural Position

The Phase 2 architecture is:

``` text
                 Knowledge Graph Engine
                          │
             ┌────────────┼────────────┐
             │            │            │
             ▼            ▼            ▼
        Assertion     Relationship     Graph
           Model          Model       Structure
             │            │            │
             └────────────┼────────────┘
                          │
                          ▼
                 semantic graph model
```

The three layers remain conceptually distinct.

##### Assertion

Represents a semantic statement:

``` text
subject ── predicate ──> object
```

Example:

``` text
Allah ── has-name ──> Ar-Rahman
```

###### Relationship

Defines what the predicate means and how it behaves.

###### Graph

Provides structural representation and graph primitives. It is not a
replacement for the semantic assertion.

------------------------------------------------------------------------

#### 3. Core Rules

1.  `KnowledgeAssertion` is first-class.
2.  A relationship predicate is not the same thing as an assertion.
3.  A graph edge is not automatically the same thing as an assertion.
4.  Semantic direction differs from traversal direction.
5.  Inverse relationships differ from symmetric relationships.
6.  Canonical inverse semantics do not require mandatory physical
    inverse duplication.
7.  Traversal must not silently become inference.
8.  Composition rules may be established in Phase 2, but full
    composition execution belongs to later reasoning/traversal work.
9.  Ontological semantic validation belongs primarily to Phase 3.
10. Full query and traversal execution belongs primarily to Phase 6.
11. Physical persistence and optimization belong to Phase 7.
12. Controlled reasoning belongs to Phase 8.
13. Indexing is used only if Phase 2 actually requires it; no artificial
    Indexing contract is introduced merely for future use.
14. Every KG identity type introduced in Phase 2 or later must use
    Core's `identity!` macro. KG never creates a second
    identity-generation system.

------------------------------------------------------------------------

#### 4. KnowledgeAssertion

##### 4.1 Selected model

Use a direct semantic assertion model:

``` text
KnowledgeAssertion
├── id
├── subject
├── predicate
├── object
├── context
├── qualifiers
└── status
```

A reusable internal assertion core may be introduced if implementation
requires it, but a generic `Assertion<T...>` framework is not required.

Evidence, provenance, authority, confidence, temporal validity, and
other later metadata are not Phase 2 responsibilities.

------------------------------------------------------------------------

#### 5. KnowledgeAssertion Identity

##### 5.1 Decision

Use **deterministic semantic identity**.

Conceptually:

``` text
subject + predicate + object + canonical context/qualifiers
                         │
                         ▼
              KnowledgeAssertionId
```

If two assertions are semantically identical according to the canonical
Phase 2 representation, they receive the same `KnowledgeAssertionId`.

Therefore:

``` text
same assertion identity
        ⇒
same canonical assertion
```

##### 5.2 Core identity mechanism

`KnowledgeAssertionId` is declared using Core's `identity!` macro.

Do not implement:

``` text
custom UUID
custom ULID
custom SHA generation
custom hash generator
custom timestamp generator
```

The Core identity mechanism remains authoritative.

------------------------------------------------------------------------

#### 6. Subject and Object Representation

##### Decision

Use a **typed extensible reference representation**.

Do not hard-code:

``` rust
subject: EntityId
object: EntityId
```

because KG relationships may cross semantic dimensions:

``` text
Entity → Concept
Entity → LexicalForm
Mention → Entity
Concept → Concept
Source → Reference
```

A closed enum is not preferred because future semantic object categories
would require repeatedly changing the enum.

A completely untyped generic reference is also not preferred because it
can allow incompatible type/ID combinations.

The Phase 1 semantic-reference foundation should be reused/enriched
rather than creating a competing reference system.

------------------------------------------------------------------------

#### 7. Predicate and Relationship Representation

##### Decision

Use a **strongly typed relationship predicate**.

The human-readable namespace is:

``` text
kg.relationship.<name>
```

Example:

``` text
kg.relationship.has-name
```

The predicate is therefore semantic infrastructure, not an arbitrary
string.

------------------------------------------------------------------------

#### 8. Relationship Vocabulary

A relationship vocabulary is first-class.

Conceptually:

``` text
RelationshipVocabulary
├── has-name
├── aliases
├── inverse support
└── future relationships
```

The exact Rust type name should be meaningful, for example:

``` text
RelationshipVocabulary
```

The vocabulary grows incrementally.

It must not be populated with every conceivable relationship at once.

The intended development model is:

``` text
requirement appears
      ↓
define relationship
      ↓
add to vocabulary
      ↓
test semantics
      ↓
use relationship
```

The vocabulary behaves like a semantic API: entries such as `has-name`
represent meaningful KG relationships and may expose semantic operations
over properly mapped assertions.

This does not make the vocabulary a general application-logic container.

------------------------------------------------------------------------

#### 9. Relationship Families

##### Decision

Use an **extensible relationship-family representation**.

The architecture identifies conceptual families including:

``` text
identity
lexical
linguistic
semantic
conceptual
hierarchical
part-whole
reference
temporal
causal
logical
knowledge
```

Phase 2 establishes the family mechanism without pretending that this
list is the final vocabulary.

A family is not a predicate:

``` text
family:
PART_WHOLE

predicate:
part-of
```

The implementation must preserve this distinction.

------------------------------------------------------------------------

#### 10. Relationship Characteristics

##### Decision

Relationship characteristics are represented with **metadata and
automatic structural/semantic behavior**.

The relationship model can express characteristics such as:

``` text
symmetric
asymmetric
transitive
reflexive
functional
```

and can be extended later.

Automatic behavior here does not make Phase 2 the full reasoning engine.

For example:

``` text
A ── R ──> B
B ── R ──> C
```

must not automatically produce:

``` text
A ── R ──> C
```

merely because `R` is transitive.

Knowledge derivation belongs to the controlled reasoning architecture.

------------------------------------------------------------------------

#### 11. Relationship Direction

Semantic direction is explicit.

Example:

``` text
Person ── acted-in ──> Movie
```

has semantic direction:

``` text
Person → Movie
```

A traversal can move in the opposite direction without changing the
semantic assertion.

Therefore:

``` text
semantic direction
    ≠
traversal direction
```

This is a hard architectural distinction.

------------------------------------------------------------------------

#### 12. Inverse Relationships

##### Decision

Use the previously selected **Option B: store one canonical relationship
and derive its inverse semantically**.

Example:

``` text
Allah ── has-name ──> Ar-Rahman
```

with an explicit relationship declaration:

``` text
has-name
    inverse-of
name-of
```

Reverse traversal can then expose:

``` text
Ar-Rahman ── name-of ──> Allah
```

without requiring a second canonical semantic assertion.

###### Rules

-   No mandatory physical inverse duplication.
-   Inverse declarations are explicit.
-   Do not infer an inverse merely because two names look opposite.
-   Semantic direction remains intact.

------------------------------------------------------------------------

#### 13. Symmetric Relationships

Symmetric relationships are different from inverse relationships.

Example:

``` text
A ── sibling-of ──> B
```

means:

``` text
B ── sibling-of ──> A
```

The predicate itself is symmetric.

By contrast:

``` text
A ── has-name ──> B
B ── name-of ──> A
```

uses inverse semantic views.

Therefore:

``` text
symmetric ≠ inverse
```

must remain explicit in the relationship model.

------------------------------------------------------------------------

#### 14. Graph Model

##### Decision

Graph structure references the canonical `KnowledgeAssertion` rather
than making a graph edge identical to the assertion.

Conceptually:

``` text
KnowledgeAssertion
        │
        ▼
Graph representation
        │
        └── assertion reference
```

This preserves:

``` text
semantic assertion
      ≠
graph execution structure
```

Graph nodes represent typed semantic references and must not create
unnecessary duplicate identities for the underlying semantic objects.

------------------------------------------------------------------------

#### 15. Graph Identity

Every KG identity introduced in this and all future phases follows one
universal rule:

``` text
KG identity type
      │
      ▼
Core identity! macro
      │
      ▼
generate()
```

This applies to:

``` text
KnowledgeAssertionId
GraphNodeId
GraphEdgeId
PathId
future KG identity types
```

and any other identity type introduced later.

No second identity-generation mechanism may be introduced in KG.

Tests must verify identity properties rather than hard-code generated ID
values.

------------------------------------------------------------------------

#### 16. Graph Edge

A graph edge represents structural connection associated with an
assertion.

Conceptually:

``` text
GraphEdge
├── id
├── assertion reference
├── source
└── target
```

The exact implementation fields are determined from actual Phase 2
needs.

The edge must not duplicate the complete semantic meaning of
`KnowledgeAssertion`.

------------------------------------------------------------------------

#### 17. Multiple Relationships

The graph must support multiple relationships between the same semantic
objects:

``` text
A ── R1 ──> B
A ── R2 ──> B
```

Therefore the semantic model cannot be:

``` text
(subject, object) → one edge
```

Predicate/assertion identity remains part of the distinction.

------------------------------------------------------------------------

#### 18. Multiple Assertions

Multiple semantically distinct assertions may connect the same objects.

Example:

``` text
A ── predicate-1 ──> B
A ── predicate-2 ──> B
```

They remain distinct through their canonical semantic identities.

------------------------------------------------------------------------

#### 19. Assertion Equality

The initial equality model is identity-based:

``` text
same KnowledgeAssertionId
        ⇒
same assertion
```

Semantic canonicalization must therefore be stable.

The implementation must not introduce a second unrelated equality rule
based on graph position, memory address, or generated graph identity.

------------------------------------------------------------------------

#### 20. Assertion Status

##### Decision

Use a **simple epistemic status enum** in Phase 2.

Status represents the semantic state of an assertion, not storage
lifecycle.

Phase 2 does not implement the complete truth/validation/reasoning
system.

Phase 8 will own the reasoning module and related derived-knowledge
logic.

------------------------------------------------------------------------

#### 21. Negative Assertions

##### Decision

Use **Option C**: establish polarity in the assertion model, but do not
implement the full negative-knowledge reasoning system yet.

Conceptually:

``` text
Assertion
├── polarity
│   ├── positive
│   └── negative
└── ...
```

The distinction remains:

``` text
absence of assertion
    ≠
negative assertion
```

Detailed interpretation and reasoning are later responsibilities.

------------------------------------------------------------------------

#### 22. Context

Context is part of `KnowledgeAssertion`.

Phase 2 establishes an extensible context type but does not implement
the complete semantic-context/interpretation system.

Potential future dimensions include:

``` text
language
domain
semantic scope
interpretive context
```

Later phases can extend the context model.

------------------------------------------------------------------------

#### 23. Qualifiers

Qualifiers are attached to assertions and are distinct from context:

``` text
KnowledgeAssertion
├── context
└── qualifiers
```

A qualifier describes additional information about a particular
assertion.

Example:

``` text
Person ── held-office ── Caliph
```

with:

``` text
start = ...
end = ...
```

Phase 2 establishes the generic qualifier structure.

Specialized temporal, evidence, provenance, authority, and other
qualifier semantics belong to later phases.

------------------------------------------------------------------------

#### 24. Assertion Validation

Phase 2 performs **structural validation**.

Examples:

``` text
subject exists
predicate exists
object exists
required assertion structure is valid
relationship representation is internally coherent
```

Phase 2 does not perform complete ontology/domain/range semantic
validation.

That belongs to Phase 3.

------------------------------------------------------------------------

#### 25. Relationship Validation vs Semantic Validation

The boundary is:

``` text
Phase 2
    relationship mechanics
    relationship characteristics
    structural consistency

Phase 3
    ontology
    semantic types
    domain/range
    constraints
    entity resolution
    semantic validity
```

Therefore Phase 2 answers:

> What relationship is this and how does it structurally behave?

Phase 3 answers:

> Is this relationship semantically valid in this ontology/context?

------------------------------------------------------------------------

#### 26. Relationship Vocabulary Registry

A relationship registry/vocabulary is required:

``` text
RelationshipVocabulary
├── relationship definitions
├── characteristics
├── inverse declarations
└── family information
```

It grows incrementally.

Only relationships actually required by the KG are added.

No speculative vocabulary explosion.

------------------------------------------------------------------------

#### 27. Initial Vocabulary

Phase 2 starts with only the minimum relationships required to establish
the architecture.

Examples already identified include:

``` text
kg.relationship.has-name
kg.relationship.aliases
```

and inverse support where required.

Additional relationships are added when actual phase requirements
appear.

------------------------------------------------------------------------

#### 28. Relationship Namespace

The selected human-readable namespace is:

``` text
kg.relationship.<name>
```

Example:

``` text
kg.relationship.has-name
```

This keeps the vocabulary readable while the Rust implementation remains
strongly typed.

------------------------------------------------------------------------

#### 29. Relationship Composition

##### Decision

Phase 2 defines composition rules/capability but does not execute full
composition.

Conceptually:

``` text
Relationship
    ↓
composition rule
    ↓
later execution
```

Example:

``` text
R1 + R2
    ↓
possible composed relationship
```

The existence of a composition rule does not mean Phase 2 automatically
derives new knowledge.

------------------------------------------------------------------------

#### 30. Composition and Reasoning

Composition belongs conceptually with reasoning because reasoning can
use composition to derive knowledge.

Therefore:

``` text
Phase 2
    define composition rules

Phase 8
    execute controlled composition/reasoning
```

This keeps Phase 2 from becoming the reasoning engine.

------------------------------------------------------------------------

#### 31. Graph Traversal

##### Phase 2

Phase 2 establishes graph primitives:

``` text
Graph
GraphNode
GraphEdge
basic adjacency
canonical direction
inverse relationship representation
basic canonical/inverse traversal primitives
```

##### Phase 6

Phase 6 owns:

``` text
multi-hop traversal
bounded traversal
filtered traversal
path execution
query planning
search
retrieval
ranking
complex traversal results
```

Phase 2 therefore does not implement the full query engine.

------------------------------------------------------------------------

#### 32. Traversal Result

Full traversal-result modeling is deferred to Phase 6.

Phase 2 only needs enough graph primitives to establish and test
structural behavior.

Future objects such as:

``` text
Path
TraversalResult
QueryResult
```

must not be overdesigned now.

------------------------------------------------------------------------

#### 33. Path Identity

A path is a query/execution artifact.

Phase 2 does not introduce path identity merely for the sake of having a
`PathId`.

If a future phase requires path identity, it must use Core's `identity!`
macro.

For Phase 2:

``` text
no unnecessary PathId behavior
```

is implemented.

------------------------------------------------------------------------

#### 34. Cycle Handling

Phase 2 establishes graph primitives rather than the complete traversal
engine.

Complete bounded/cycle-safe traversal belongs to Phase 6.

Phase 2 must not introduce an unrelated traversal engine solely to
handle future query cases.

------------------------------------------------------------------------

#### 35. Graph vs Query Planner

The graph is a structural component.

It is not the KG query planner.

The later architecture remains:

``` text
Control Plane
      ↓
KG capability
      ↓
KG Engine Runtime
      ↓
KG Query Planner
      ↓
graph/semantic execution
```

The KG query planner remains KG-local and must not become a second
global Control Plane.

------------------------------------------------------------------------

#### 36. Indexing Engine Boundary

Phase 2 does not add artificial Indexing integration.

The rule is intentionally simple:

``` text
If Phase 2 needs Indexing
        ↓
use the appropriate Indexing mechanism

If Phase 2 does not need Indexing
        ↓
do not call Indexing
        ↓
do not create a fake contract
```

`IndexAssignedId` remains an Indexing-owned source-object identity.

KG must not redefine or generate it.

This preserves:

``` text
KG semantic identity
        ≠
Indexing assigned-object identity
```

------------------------------------------------------------------------

#### 37. Core / Control Plane Boundary

Control Plane and Indexing Engine are separate architectural components
with different responsibilities.

KG uses Core's Control Plane and runtime infrastructure according to the
needs of each phase.

Phase 2 does not create a second Control Plane.

If Phase 2 functionality is exposed through Nizaam execution, it follows
the established Core boundary:

``` text
UniversalRequest
      ↓
KG capability boundary
      ↓
KG typed operation
      ↓
assertion/relationship execution
      ↓
UniversalResponse
```

Only capabilities actually required by Phase 2 are introduced.

------------------------------------------------------------------------

#### 38. Persistence Boundary

Phase 2 does not implement physical persistence.

The existing decision to defer `.nizaam` persistence remains in force.

Phase 2 can therefore use in-memory semantic structures while
establishing the logical model.

Phase 7 owns:

``` text
physical representation
repository
transactions
snapshots
versioning
storage optimization
persistence
```

------------------------------------------------------------------------

#### 39. Versioning and Mutation

Phase 2 does not implement the complete versioning architecture.

Assertions should be treated as semantic values while
mutation/versioning semantics are formalized later.

Phase 7 owns the physical/versioning architecture.

Phase 2 must not create an ad-hoc mutable persistence model that later
phases have to undo.

------------------------------------------------------------------------

#### 40. Error Model

Only errors required by actual Phase 2 behavior should be introduced.

Do not create a speculative error taxonomy.

Errors may be added incrementally for:

``` text
assertion construction
relationship validation
inverse registration
graph structural operations
```

as actual implementation requires them.

------------------------------------------------------------------------

#### 41. Module Structure

The Phase 2 planned structure is:

``` text
src/
├── assertion/
│   ├── assertion.rs
│   ├── context.rs
│   ├── object.rs
│   ├── predicate.rs
│   ├── qualifier.rs
│   ├── status.rs
│   └── mod.rs
│
├── relationship/
│   ├── characteristic.rs
│   ├── direction.rs
│   ├── inverse.rs
│   ├── predicate.rs
│   ├── relationship.rs
│   └── mod.rs
│
└── graph/
    ├── edge.rs
    ├── graph.rs
    ├── node.rs
    ├── path.rs
    ├── traversal.rs
    └── mod.rs
```

##### Important

The repository already contains future-phase files in these directories.
Their existence does not mean all of their functionality belongs to
Phase 2.

For example:

``` text
graph/path.rs
graph/traversal.rs
```

may exist as scaffolding while complete path/query traversal remains
Phase 6 functionality.

------------------------------------------------------------------------

#### 42. File Responsibilities

##### `src/assertion/assertion.rs`

Own:

-   `KnowledgeAssertion`
-   assertion identity usage
-   canonical assertion construction
-   identity/equality behavior
-   structural assertion validation

Do not place graph traversal here.

##### `src/assertion/context.rs`

Own the Phase 2 assertion-context representation.

Do not implement the complete Phase 3 semantic context system.

##### `src/assertion/object.rs`

Own typed extensible subject/object references.

Do not create a second identity system.

##### `src/assertion/predicate.rs`

Own the assertion-level predicate reference and connection to the
relationship vocabulary.

##### `src/assertion/qualifier.rs`

Own generic Phase 2 qualifier representation.

##### `src/assertion/status.rs`

Own the Phase 2 epistemic status representation.

##### `src/relationship/relationship.rs`

Own the core relationship definition:

``` text
identity
family
direction
inverse information
characteristics
```

##### `src/relationship/predicate.rs`

Own strongly typed relationship predicate identity/representation and
the `kg.relationship.<name>` namespace.

##### `src/relationship/characteristic.rs`

Own relationship characteristics.

Do not turn this into a reasoning engine.

##### `src/relationship/direction.rs`

Own semantic relationship direction.

Do not confuse it with traversal direction.

##### `src/relationship/inverse.rs`

Own explicit inverse declarations and inverse semantics.

Canonical relationship storage remains primary.

##### `src/relationship/mod.rs`

Expose relationship model and vocabulary infrastructure.

##### `src/graph/node.rs`

Own structural graph-node representation.

Avoid unnecessary duplicate semantic identities.

##### `src/graph/edge.rs`

Own structural graph-edge representation associated with the canonical
assertion.

##### `src/graph/graph.rs`

Own basic graph structure and adjacency primitives.

It is not the Phase 6 query engine.

##### `src/graph/path.rs`

Only the minimum Phase 2 scaffold required by the graph architecture.

No full path-query semantics.

##### `src/graph/traversal.rs`

Only Phase 2 canonical/inverse traversal primitives.

Full traversal belongs to Phase 6.

------------------------------------------------------------------------

#### 43. Relationship Vocabulary Evolution

The vocabulary is intentionally incremental.

Conceptually:

``` text
RelationshipVocabulary
│
├── has_name()
├── aliases()
└── inverse support
```

Later phases add relationships when actual requirements appear.

The vocabulary is a semantic registry/API, not a general
application-logic container.

------------------------------------------------------------------------

#### 44. Phase 2 / Phase 3 Boundary

##### Phase 2

``` text
KnowledgeAssertion
relationship definition
predicate
direction
inverse
characteristics
relationship family
basic structural validation
graph primitives
canonical/inverse traversal primitives
```

###### Phase 3

``` text
ontology
classes
properties
taxonomy
constraints
semantic types
meaning
context interpretation
entity resolution
domain/range semantic validation
```

------------------------------------------------------------------------

#### 45. Phase 2 / Phase 6 Boundary

##### Phase 2

``` text
graph primitives
basic adjacency
canonical direction
inverse traversal primitive
```

###### Phase 6

``` text
query
planning
multi-hop traversal
path execution
filters
search
retrieval
ranking
complex traversal results
```

------------------------------------------------------------------------

#### 46. Phase 2 / Phase 7 Boundary

##### Phase 2

``` text
logical semantic structures
in-memory graph primitives
identity/relationship model
```

###### Phase 7

``` text
physical storage
repository
transactions
snapshots
versioning
performance
physical indexes
persistence
```

------------------------------------------------------------------------

#### 47. Phase 2 / Phase 8 Boundary

##### Phase 2

``` text
relationship characteristics
composition rule declarations
structural graph behavior
```

###### Phase 8

``` text
controlled inference
derived knowledge
composition execution
reasoning profiles
reasoning validation
inference lifecycle
```

A relationship being marked `transitive` does not mean Phase 2 silently
generates transitive facts.

------------------------------------------------------------------------

#### Testing Strategy

Tests cover only behavior actually implemented in Phase 2.

##### Identity

Test:

``` text
KnowledgeAssertionId exists
ID type is distinct
Core identity mechanism is used
same canonical assertion → same identity
different semantic assertion → different identity
```

Never assert exact generated ID contents.

##### Assertion

Test:

``` text
valid construction
invalid structural assertion
typed subject/object preservation
predicate association
context preservation
qualifier preservation
status preservation
polarity representation
```

##### Relationship

Test:

``` text
relationship definition
family
characteristics
semantic direction
inverse declaration
inverse behavior
symmetric behavior
```

##### Graph

Test:

``` text
node creation
edge creation
assertion-to-edge association
multiple relationships between nodes
canonical traversal
inverse traversal
direction preservation
```

##### Negative tests

Where Phase 2 introduces validation, add negative tests for:

``` text
invalid assertion structure
invalid relationship definition
invalid inverse declaration
invalid graph edge association
invalid typed reference
```

------------------------------------------------------------------------

#### 49. Inverse Conformance Test

A central test must establish:

``` text
canonical:

A ── R ──> B
```

and:

``` text
inverse traversal:

B ── inverse(R) ──> A
```

without requiring a second canonical assertion to be stored.

The test must also verify that semantic direction remains unchanged.

------------------------------------------------------------------------

#### 50. Symmetry Conformance Test

A symmetric relationship must demonstrate:

``` text
A ── R ──> B
```

and allow the semantic reverse interpretation:

``` text
B ── R ──> A
```

without representing this as a named inverse relationship.

------------------------------------------------------------------------

#### 51. Identity Testing Rule

All identity tests follow Core's identity principles.

Test:

``` text
presence
type distinction
equality
stability
propagation
```

Do not hard-code generated values.

For example, tests must not expect a particular:

``` text
KnowledgeAssertionId-<hash>
```

string.

------------------------------------------------------------------------

#### 52. Recommended Implementation Order

Implement Phase 2 incrementally:

``` text
1. assertion identity integration
        ↓
2. typed assertion references
        ↓
3. predicate representation
        ↓
4. assertion model
        ↓
5. status/context/qualifier foundations
        ↓
6. relationship definition
        ↓
7. relationship characteristics
        ↓
8. direction
        ↓
9. inverse model
        ↓
10. relationship vocabulary
        ↓
11. graph node
        ↓
12. graph edge
        ↓
13. graph container
        ↓
14. basic canonical traversal
        ↓
15. inverse traversal
        ↓
16. composition rule declarations
        ↓
17. Phase 2 tests/conformance
```

Each substantial layer should be implemented and verified before moving
forward.

------------------------------------------------------------------------

#### Non-Goals

Phase 2 must not implement:

``` text
full ontology
ontology constraints
entity resolution
evidence system
provenance system
authority system
uncertainty engine
temporal reasoning
domain-specific ingestion
source extraction
full search engine
full query planner
full retrieval engine
physical persistence
database provider
storage transactions
storage snapshots
performance indexes
full versioning system
Python ML
embeddings
similarity
clustering
anomaly detection
full inference engine
reasoning profiles
```

Phase 2 must also not duplicate Core infrastructure or Indexing
infrastructure.

------------------------------------------------------------------------

#### 54. Architectural Example

A minimal semantic example:

``` text
Entity:
    Allah

Semantic object:
    Ar-Rahman

Relationship:
    kg.relationship.has-name

Assertion:

    subject
        = Allah

    predicate
        = kg.relationship.has-name

    object
        = Ar-Rahman
```

Canonical graph:

``` text
Allah
  │
  │ has-name
  ▼
Ar-Rahman
```

Inverse semantic view:

``` text
Ar-Rahman
  │
  │ name-of
  ▼
Allah
```

Only the canonical semantic assertion needs to exist.

------------------------------------------------------------------------

#### 55. Cross-Dimensional Relationship Example

Phase 2 must support relationships across semantic dimensions, for
example:

``` text
LexicalForm
      │
      │ expresses
      ▼
Concept
```

or:

``` text
Mention
      │
      │ refers-to
      ▼
Entity
```

This does not mean arbitrary objects may be connected arbitrarily.

Typed references and relationship semantics establish the structure;
Phase 3 later provides richer ontology/domain/range validation.

------------------------------------------------------------------------

#### 56. Composition Example

Phase 2 may define a relationship composition rule:

``` text
part-of
```

but does not automatically derive:

``` text
A part-of B
B part-of C
        ↓
A part-of C
```

The rule can exist as semantic information for later reasoning.

Phase 8 can execute controlled reasoning over that rule.

------------------------------------------------------------------------

#### Completion Criteria

Phase 2 is complete when:

##### Knowledge Assertion

-   [ ] `KnowledgeAssertionId` exists.
-   [ ] Deterministic semantic assertion identity exists.
-   [ ] Core `identity!` is used.
-   [ ] Subject/object use typed extensible references.
-   [ ] Predicate is strongly typed.
-   [ ] Context exists.
-   [ ] Qualifiers exist.
-   [ ] Simple epistemic status exists.
-   [ ] Structural assertion validation exists.
-   [ ] Positive/negative polarity representation exists without full
    reasoning.

###### Relationship

-   [ ] Relationship definition exists.
-   [ ] Relationship predicate exists.
-   [ ] `kg.relationship.<name>` namespace exists.
-   [ ] Relationship family exists.
-   [ ] Relationship characteristics exist.
-   [ ] Semantic direction exists.
-   [ ] Explicit inverse declarations exist.
-   [ ] Symmetric and inverse semantics remain distinct.
-   [ ] Relationship vocabulary exists.
-   [ ] Vocabulary is incrementally extensible.
-   [ ] Composition rules can be represented without executing full
    reasoning.

###### Graph

-   [ ] Graph node exists.
-   [ ] Graph edge exists.
-   [ ] Graph edges reference canonical assertions.
-   [ ] Multiple relationships between objects are supported.
-   [ ] Basic graph structure exists.
-   [ ] Canonical forward traversal primitive exists.
-   [ ] Inverse traversal primitive exists.
-   [ ] Traversal preserves semantic direction.
-   [ ] Full query/traversal remains Phase 6 responsibility.

###### Boundaries

-   [ ] No physical storage implementation.
-   [ ] No full ontology engine.
-   [ ] No full semantic-validation engine.
-   [ ] No full reasoning engine.
-   [ ] No speculative Indexing integration.
-   [ ] No duplicate Core identity generation.
-   [ ] No duplicate Control Plane.
-   [ ] No speculative relationship vocabulary.

###### Testing

-   [ ] Implemented Phase 2 behavior has focused tests.
-   [ ] Negative tests exist for implemented validation.
-   [ ] Identity tests do not assert generated ID contents.
-   [ ] Inverse traversal is tested without duplicated inverse
    assertions.
-   [ ] Symmetry and inverse semantics are tested separately.
-   [ ] Graph and assertion responsibilities are tested separately.

------------------------------------------------------------------------


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

> **Phase 2 establishes how the KG expresses a semantic statement,
> defines the relationship used by that statement, and represents that
> statement structurally in a graph. It does not yet decide the complete
> ontology, storage system, query system, ingestion pipeline, or
> reasoning system.**

The resulting architecture is:

``` text
                  KnowledgeAssertion
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Subject    Predicate    Object
                         │
                         ▼
               Relationship Model
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
     Direction         Inverse      Characteristics
        │                │                │
        └────────────────┼────────────────┘
                         ▼
                 Relationship
                   Vocabulary
                         │
                         ▼
                   Graph Structure
                         │
                ┌────────┴────────┐
                ▼                 ▼
            Forward            Inverse
           traversal           traversal
                │                 │
                └────────┬────────┘
                         ▼
                  Later Query Layer
                       Phase 6

Composition rules
        │
        ▼
   Later reasoning
       Phase 8
```

------------------------------------------------------------------------

### Phase 3: Ontology, Semantics & Entity Resolution

#### Status

**Planned**

---

#### Goal

Phase 3 establishes the semantic structures required to prevent the Knowledge Graph from becoming an arbitrary collection of connected objects.

Phase 3 introduces three closely related capabilities:

1. **Ontology**
   - classes
   - properties
   - domain and range
   - constraints
   - taxonomy and hierarchy infrastructure
   - semantic typing
   - ontology registry

2. **Semantics**
   - meaning
   - interpretation
   - context
   - semantic types
   - lexical-to-concept connections

3. **Entity Resolution**
   - source-level references and mentions
   - normalization
   - candidate generation
   - deterministic candidate matching/ranking
   - ambiguity handling
   - canonical identity resolution
   - external-identifier crosswalks
   - graph-assisted resolution

Phase 3 establishes the **structural and semantic primitives** for these capabilities. It does not attempt to implement the later ingestion-governance workflow, evidence/provenance system, full query system, reasoning engine, physical persistence, ontology version-management system, or ML layer.

---

#### 2. Architectural Position

Phase 3 builds directly on the identity, assertion, and relationship model established in Phases 0–2.

The architectural dependency is:

```text
Phase 0
Engine / Runtime / Core Integration
        ↓
Phase 1
Identity + Core Semantic Objects
        ↓
Phase 2
Knowledge Assertion + Relationship Model
        ↓
Phase 3
Ontology + Semantics + Entity Resolution
        ↓
Phase 4
Evidence + Provenance + Authority + Uncertainty + Temporal Knowledge
        ↓
Phase 5
Ingestion + Validation + Governance
        ↓
Phase 6
Query + Retrieval + Traversal
        ↓
Phase 7
Storage + Indexing + Versioning
        ↓
Phase 8
Reasoning + Inference
        ↓
Phase 9
Rust ↔ Python ML Integration
        ↓
Phase 10
Nizaam Integration + Conformance + Hardening
```

Phase 3 therefore provides the semantic foundation that later phases consume.

---

#### 3. Inherited Architectural Rules

Phase 3 inherits the following decisions from Phases 0–2.

##### 3.1 Core Identity System

Every KG identity introduced now or in future phases uses the Core `identity!` macro.

KG must not introduce:

- UUID generation
- ULID generation
- custom hash-based identity generation
- a second identity framework

Generated identity values must not be asserted directly in tests.

Tests should verify:

- identity presence
- equality where semantically required
- inequality/distinctness where required
- propagation
- type safety
- behavioral correctness

---

##### 3.2 No Duplicate Core Infrastructure

Phase 3 does not recreate Core infrastructure.

In particular, KG does not implement its own:

- message framing
- transport
- routing
- Control Plane
- lifecycle infrastructure
- universal message identity
- Core communication protocol

KG consumes the Core infrastructure established in earlier phases.

---

##### 3.3 Knowledge Assertion Remains the Semantic Unit

A relationship is not merely an unqualified graph edge.

The Phase 2 Knowledge Assertion model remains authoritative.

Conceptually:

```text
Knowledge Assertion
├── subject
├── predicate
├── object
├── context
├── qualifiers
├── evidence
├── provenance
├── authority
├── confidence
├── temporal validity
└── status
```

Phase 3 does not implement all of these fields. Later phases own evidence, provenance, authority, uncertainty, temporal knowledge, and related concerns.

---

##### 3.4 Relationship Registry Continuity

Phase 3 extends the relationship vocabulary/registry introduced in Phase 2.

It does not create a competing relationship system.

The relationship namespace remains:

```text
kg.relationship.<name>
```

Examples include semantic predicates such as:

```text
kg.relationship.has-name
kg.relationship.aliases
kg.relationship.inverse
```

Ontology properties build formal semantic constraints around relationship predicates rather than replacing the Phase 2 relationship registry.

---

#### 4. Phase 3 Scope

The scope of Phase 3 is:

```text
Ontology
├── Classes
├── Properties
├── Domain / Range
├── Constraints
├── Taxonomy
└── Ontology Registry

Semantics
├── Semantic Types
├── Meaning
├── Interpretation
├── Context
└── Lexical ↔ Concept Semantics

Entity Resolution
├── References / Mentions
├── Normalization
├── Candidate Generation
├── Candidate Ranking
├── Disambiguation
├── Resolution State
├── Canonical Identity Resolution
├── External Identifier Crosswalks
└── Graph-Assisted Resolution
```

---

#### 5. Ontology Model

##### 5.1 Class as a First-Class Ontology Object

A class is a first-class ontology object.

Initial architecture:

```text
Class
├── ClassId
├── semantic metadata
├── hierarchy relationships
└── ontology constraints
```

Phase 3 initially gives each class a dedicated `ClassId`.

The architecture remains open to later reusing `ConceptId` for classes if implementation experience demonstrates that this produces a cleaner model.

That is a future architectural evolution, not a Phase 3 requirement.

---

##### 5.2 Class vs Concept

A **Class** and a **Concept** remain separate semantic objects.

###### Class

Answers:

> What kind of thing is this?

Examples:

```text
Person
Book
Place
Organization
Event
ReligiousText
```

###### Concept

Represents a semantic concept or meaning in the knowledge model.

A concept can participate in semantic relationships without being identical to an ontology class.

The implementation must not collapse the two merely for convenience.

---

##### 5.3 Semantic Types

KG supports semantic typing using a hybrid representation.

An entity/object may have:

```text
Primary Semantic Type
+
Additional Semantic Types
```

Semantic typing is represented through both:

1. a direct typed field for efficient structural access
2. ontology assertions for explicit semantic representation

Conceptually:

```text
Entity
├── primary_type
├── additional_types[]
└── ontology assertions describing type membership
```

This permits both convenient runtime access and explicit graph semantics.

---

##### 5.4 Multiple Semantic Types

An object may belong to multiple semantic types.

The model therefore distinguishes:

```text
primary semantic type
additional semantic types
```

The architecture must not assume that every entity has exactly one type.

---

#### 6. Multiple Inheritance

Ontology hierarchy infrastructure supports multiple inheritance structurally.

Example:

```text
A
├── B
└── C

D
└── B
└── C
```

Phase 3 records the structure.

It does not implement the full inference consequences of multiple inheritance.

Reasoning over inheritance belongs to Phase 8.

---

#### 7. Ontology Properties

##### 7.1 Property vs Relationship Predicate

Phase 3 distinguishes between:

###### Relationship predicate

The Phase 2 operational semantic relationship.

```text
kg.relationship.has-name
```

###### Ontology property

A formal ontology-level definition that can describe:

```text
property
├── semantic relationship
├── domain
├── range
├── hierarchy metadata
├── characteristics
└── constraints
```

Therefore:

> A relationship predicate expresses a semantic relationship; an ontology property formally constrains and describes how that relationship may be used.

---

##### 7.2 Domain and Range

Domain and range are required for ontology-defined properties.

For example:

```text
Property:
    authored-by

Domain:
    Book

Range:
    Person
```

Generic Phase 2 relationships may remain unconstrained.

Therefore:

```text
Generic relationship
    → domain/range optional

Ontology-defined property
    → domain/range required
```

Domain/range implications may later produce inference candidates, but Phase 3 does not execute the inference.

---

#### 8. Constraint System

Phase 3 introduces an extensible minimum constraint framework.

The architecture must not attempt to implement every possible ontology constraint immediately.

Initial constraint infrastructure should be extensible so future phases can add constraint types without replacing the model.

---

##### 8.1 Constraint Violation Policy

Constraint violations are context-dependent.

The system supports:

- warning
- error
- context-dependent acceptance/rejection
- explicit invalidity state

A non-fatal invalid assertion may remain represented with its invalidity information.

For a fatal violation, the assertion is rejected from the **active KG state**.

The source-level record or provenance trace must not be interpreted as requiring physical destruction of all evidence that the source contained the rejected assertion.

Conceptually:

```text
source-level record
        ↓
semantic validation
        ↓
fatal violation
        ↓
rejected from active KG state
```

This preserves compatibility with the later evidence/provenance model.

---

##### 8.2 Cardinality

Cardinality constraints are defined structurally in Phase 3.

However, full enforcement is deferred.

Phase 3 therefore establishes the vocabulary and representation without turning the ontology layer into a complete enforcement engine.

---

##### 8.3 Disjointness

Disjointness is represented in the ontology model.

Full enforcement is deferred to later semantic validation/reasoning work.

The presence of a disjointness declaration does not automatically cause Phase 3 to perform global inference.

---

#### 9. Taxonomy and Hierarchy

##### 9.1 Ontology vs Taxonomy

Ontology defines classes and their formal semantic properties.

Taxonomy organizes concepts/classes into hierarchical structures.

Therefore:

```text
Ontology
    → defines what kinds of things exist

Taxonomy
    → organizes them hierarchically
```

They share infrastructure where practical, but they remain semantically distinct.

---

##### 9.2 Shared Hierarchy Infrastructure

The implementation may use common hierarchy machinery while retaining separate semantic predicates.

Examples:

```text
subclass-of
instance-of
broader-than
narrower-than
part-of
```

These must not be silently collapsed into one generic relationship.

---

##### 9.3 Class Hierarchy vs Concept Hierarchy

Class hierarchy and concept hierarchy are distinct.

They may share structural hierarchy infrastructure, but they use separate semantic predicates.

This prevents:

```text
Class
=
Concept
```

and prevents:

```text
subclass-of
=
broader-than
```

from becoming accidental assumptions.

---

##### 9.4 Part-Whole Relationships

Part-whole relationships are distinct from taxonomy.

Ontology may formally classify part-whole relationships where appropriate.

A part-whole relation must not automatically be treated as:

- subclassing
- conceptual broader/narrower relation
- simple graph adjacency

---

##### 9.5 Multiple Taxonomy Schemes

Phase 3 initially implements one working taxonomy mechanism.

The architecture must nevertheless be designed so multiple taxonomy schemes can coexist later.

This avoids coupling the semantic model to one permanent hierarchy.

---

#### 10. Meaning

Meaning is a first-class semantic object.

The model must distinguish:

```text
lexical form
    ≠
sense
    ≠
meaning
    ≠
concept
    ≠
entity
```

A lexical form can have multiple senses.

A meaning can participate in semantic interpretation without becoming identical to a concrete entity.

---

#### 11. Interpretation

Interpretation is a first-class object.

Interpretation represents how some source/linguistic material is understood in a particular semantic context.

The architecture deliberately avoids making every interpretation mutate the canonical semantic object.

Instead:

```text
Source / Linguistic material
        ↓
Interpretation
        ↓
Semantic connection
        ↓
Concept / Entity / Assertion
```

---

#### 12. Context

Context uses a hybrid model.

##### 12.1 Reusable Contexts

Reusable contexts receive identities and can be referenced repeatedly.

##### 12.2 Ephemeral Contexts

Short-lived or one-off contexts can be represented directly as values without creating unnecessary persistent identities.

Therefore:

```text
Reusable context
    → identity-backed

Ephemeral context
    → value-backed
```

---

##### 12.3 Typed Context Dimensions

Context dimensions are typed.

The model should support multiple dimensions rather than assuming that context is one unstructured string.

Potential dimensions include:

```text
language
source
domain
audience
time
location
interpretive framework
scholarly context
```

The actual vocabulary can grow in later phases.

---

##### 12.4 Contextual Interpretation

Contextual interpretation must not mutate the canonical meaning or concept simply because the interpretation differs by context.

Therefore:

```text
Contextual relevance
    ≠
Semantic identity
```

This is especially important for Islamic knowledge where the same lexical or conceptual material may appear in different interpretive contexts.

---

#### 13. Lexical-to-Concept Semantics

KG supports both:

```text
Lexical Form
    ↓
Concept
```

and:

```text
Lexical Form
    ↓
Sense
    ↓
Concept
```

The choice depends on available evidence and context.

The KG therefore must not force every lexical mapping through exactly one representation.

---

#### 14. Arabic Engine Boundary

The Arabic Engine owns linguistic analysis and linguistic interpretation.

KG stores and connects the resulting semantic information.

The boundary is:

```text
Arabic Engine
    ↓
linguistic analysis / interpretation
    ↓
KG semantic representation
```

KG must not duplicate Arabic linguistic processing merely because it consumes Arabic-derived information.

---

#### 15. Entity Resolution

##### 15.1 Core Principle

Naming is not identity.

Examples:

```text
name
alias
transliteration
source identifier
external identifier
mention
reference
```

must not automatically become canonical identity.

A stable internal entity identity remains distinct from names and aliases.

---

#### 16. Resolution Pipeline

Phase 3 defines the resolution pipeline and its interfaces.

The conceptual pipeline is:

```text
Mention / Reference
        ↓
Normalization
        ↓
Candidate Generation
        ↓
Candidate Ranking
        ↓
Identity Decision
        ↓
Resolved / Ambiguous / Unresolved / Unknown / Provisional / Rejected
```

Phase 3 implements the minimum deterministic portion of this pipeline.

Advanced probabilistic and ML-assisted resolution is deferred.

---

#### 17. Candidate Generation

Candidate generation is designed for high recall.

Phase 3 supports deterministic/non-ML candidate strategies including:

```text
exact identifier
exact alias
normalized string
transliteration
lexical mapping
source identifier
language mapping
semantic relationships
graph neighborhood
context
```

ML embedding-based candidate generation is explicitly excluded from Phase 3.

This does not prevent future ML integration.

---

#### 18. Candidate Ranking

Phase 3 implements basic deterministic candidate ranking.

Ranking may combine multiple deterministic signals rather than relying on a single string comparison.

The architecture remains open for more advanced ranking later.

The following must not become implicit canonical-identity rules:

- popularity
- arbitrary source frequency
- embedding similarity alone
- LLM output alone

---

#### 19. Resolution Threshold and Candidate Margin

Resolution decisions use both:

1. an absolute threshold
2. a candidate margin

Conceptually:

```text
best candidate score >= threshold
AND
best score - second-best score >= margin
```

This prevents a weak candidate from being treated as resolved merely because it happens to be the top candidate.

---

#### 20. Resolution State Model

Phase 3 uses the following resolution states:

```text
Resolved
Ambiguous
Unresolved
Unknown
Provisional
Rejected
```

These states are semantically distinct.

##### Resolved

The reference has been associated with a canonical identity under the current resolution policy.

###### Ambiguous

Multiple candidates remain sufficiently plausible.

###### Unresolved

A resolution attempt occurred but no acceptable candidate was established.

###### Unknown

The system does not have enough information to determine whether a known identity exists.

###### Provisional

A candidate resolution is useful as a working hypothesis but must not become a canonical entity identity.

###### Rejected

A candidate or resolution decision has been explicitly rejected.

---

#### 21. Provisional Resolution

A provisional candidate does not create a canonical `Entity`.

Conceptually:

```text
Reference
    ↓
Provisional candidate
    ↓
future decision
```

This avoids polluting the canonical identity space with premature entities.

---

#### 22. Resolution vs Merge

Entity resolution does not perform entity merging.

Phase 3 answers:

> Which existing canonical identity may this reference correspond to?

It does not answer:

> Which canonical entities should be permanently merged?

Merge operations are deferred to later architecture.

---

#### 23. Reversible Resolution Decisions

Phase 3 represents resolution decisions so that they can be revised.

Full historical resolution/version history is deferred to later versioning/provenance phases.

The Phase 3 model therefore records enough structure to represent a decision without pretending to provide the complete historical system.

---

#### 24. External Identifier Crosswalks

External identifiers are preserved through separate crosswalk mapping records.

Conceptually:

```text
External Identifier
        ↓
Crosswalk Mapping
        ↓
Canonical Entity
```

The crosswalk must preserve the distinction between:

- source identity
- external identifier
- canonical KG identity

The crosswalk is not itself a canonical entity identity.

---

#### 25. IndexAssignedId Boundary

`IndexAssignedId` remains outside the entity-resolution system.

Phase 3 does not redefine, generate, or interpret the Indexing Engine's identity.

Its future role is to support KG/index mapping where required.

Therefore:

```text
Entity Resolution
    ≠
IndexAssignedId generation
```

and:

```text
KG
    consumes IndexAssignedId later
    rather than redefining it
```

No actual Indexing integration is required merely because the identity may later be useful.

---

#### 26. Resolution Evidence Boundary

Phase 3 records the resolution result and minimum resolution certainty/status.

Full evidence and provenance for why a resolution was made belong to Phase 4.

Therefore:

```text
Phase 3
    → resolution result + minimal certainty/state

Phase 4
    → evidence + provenance + authority + richer confidence
```

---

#### 27. Resolution Confidence vs Claim Confidence

Resolution confidence and claim-truth confidence are different dimensions.

For example:

```text
"Reference X resolves to Entity Y"
```

can have one certainty level, while:

```text
"Entity Y has relationship R with Entity Z"
```

can have another.

The implementation must not reuse claim confidence as identity confidence.

Phase 4 provides the richer confidence/provenance model.

---

#### 28. Context in Entity Resolution

Context is a strong resolution signal.

Candidate resolution may consider:

```text
language
source
document
surrounding references
semantic type
graph neighborhood
time
domain
interpretive context
```

Context helps distinguish references that have the same or similar surface forms.

Contextual evidence must not silently become canonical identity.

---

#### 29. Graph-Assisted Resolution

Graph relationships may participate in entity resolution.

For example:

```text
Reference
   ↓
candidate A
   ↓
known graph neighborhood

Reference
   ↓
candidate B
   ↓
incompatible graph neighborhood
```

Graph context can therefore assist candidate ranking.

However, graph-assisted resolution must be controlled to avoid circular evidence.

---

#### 30. Circular Evidence Protection

Resolution must track dependency between identity decisions and graph-derived evidence.

An uncertain assertion whose existence depends on the same uncertain identity decision must not be treated as independent evidence for confirming that identity.

Conceptually:

```text
identity hypothesis
        ↓
uncertain graph assertion
        ↓
must NOT become independent evidence
        ↓
same identity hypothesis
```

Phase 3 establishes the structural protection.

The complete evidence/provenance model is deferred to Phase 4.

---

#### 31. Semantic Validation Boundary

Phase 3 defines semantic validation primitives.

It does not own the complete governed ingestion validation workflow.

The intended separation is:

```text
Phase 3
    → semantic rules and validation primitives

Phase 5
    → governed ingestion validation and publication workflow
```

This keeps semantic definitions separate from operational ingestion governance.

---

#### 32. Ontology Inference Boundary

Phase 3 may define the structural rules required for ontology inference.

It does not execute the complete inference engine.

For example:

```text
domain/range rule
subclass rule
property characteristic
disjointness rule
```

may be represented.

Execution belongs to Phase 8.

Therefore:

```text
Phase 3
    defines rules

Phase 8
    executes reasoning/inference
```

---

#### 33. Relationship Registry Extension

Phase 3 extends the same relationship registry established in Phase 2.

Ontology properties do not introduce a separate competing vocabulary.

The architecture therefore remains:

```text
Phase 2 relationship registry
            ↓
Phase 3 ontology/property semantics
            ↓
future reasoning/query usage
```

---

#### 34. Identity Minimization

Phase 3 minimizes the number of new identities.

A new identity is introduced only where it represents a genuine first-class semantic object.

The implementation should prefer:

- existing assertion identities
- existing reference identities
- existing Core identities
- value objects

over creating unnecessary IDs.

---

#### 35. Candidate Identity

Candidate objects are ephemeral.

Candidates do not receive a persistent `CandidateId`.

Conceptually:

```text
Candidate
├── reference
├── possible target
├── deterministic score/signals
└── resolution metadata
```

The candidate itself is not a canonical identity.

---

#### 36. Ontology Registry

A central ontology registry is required.

Conceptually:

```text
Ontology Registry
├── classes
├── properties
├── constraints
├── taxonomy definitions
└── active ontology snapshot
```

The registry provides a controlled access point for ontology definitions.

---

#### 37. Runtime Mutability and Immutable Snapshots

The ontology registry uses a hybrid approach:

- controlled APIs may update the registry
- ontology definitions/snapshots are immutable once created

An update therefore creates or activates a new immutable snapshot rather than mutating an existing definition in place.

Conceptually:

```text
Ontology Registry

V1 immutable
V2 immutable
V3 immutable
       ↑
 current snapshot
```

This provides controlled evolution without allowing arbitrary in-place semantic mutation.

---

#### 38. Ontology Versioning Boundary

Phase 3 does **not** implement ontology version management.

Formal ontology versioning belongs to Phase 7.

However, Phase 3's APIs and validation abstractions must remain architecturally compatible with future ontology-version-relative validity.

Therefore the conceptual boundary is:

```text
Phase 3:
validate(assertion, ontology)

Future version-aware architecture:
validate(assertion, ontology_version)
```

Phase 3 must not introduce a fake or incomplete versioning subsystem merely to satisfy future requirements.

---

#### 39. Ontology Evolution

Ontology validity is conceptually relative to the ontology definition/snapshot under which it is evaluated.

The full lifecycle of ontology versions is deferred to Phase 7.

This means Phase 3 establishes a clean semantic boundary without implementing the later version-management machinery.

---

#### 40. Generic + Islamic Ontology Domain

The ontology implementation is generic while Phase 3 also establishes an initial Islamic vocabulary.

The architecture must not hard-code the entire KG around one domain.

Conceptually:

```text
Generic Ontology Infrastructure
        +
Initial Islamic Ontology Vocabulary
```

This permits later expansion into other knowledge domains without redesigning the ontology engine.

---

#### 41. Hard-Coded + Data-Driven Implementation

Phase 3 uses a hybrid implementation strategy.

Some ontology structures can be represented through Rust types and compile-time semantics.

Other vocabulary and definitions should remain data-driven where appropriate.

The goal is:

```text
Rust
→ structural guarantees / behavior

Data
→ extensible ontology vocabulary / definitions
```

The implementation must avoid both extremes:

```text
everything hard-coded
```

and:

```text
everything dynamic with no type safety
```

---

#### Non-Goals

The following are explicitly outside Phase 3 implementation scope.

##### Deferred to Phase 4

- evidence
- provenance
- authority
- scholarly status
- rich uncertainty
- contradiction modeling
- richer confidence
- temporal validity

##### Deferred to Phase 5

- complete ingestion pipeline
- governed validation
- approval
- publication
- ingestion lifecycle enforcement

##### Deferred to Phase 6

- full query planner
- complete traversal engine
- semantic retrieval
- query result model

##### Deferred to Phase 7

- physical persistence architecture
- storage implementation
- complete indexing integration
- ontology version management
- migration/version lifecycle

##### Deferred to Phase 8

- ontology inference execution
- full semantic reasoning
- inheritance reasoning
- relationship composition execution
- controlled inference

##### Deferred to Phase 9

- Python ML integration
- embeddings
- clustering
- similarity models
- anomaly detection
- model execution

##### Deferred to Phase 10

- final Nizaam ecosystem integration
- complete conformance
- final hardening

---

#### 43. Module and File Plan

The Phase 3 implementation is organized around the three scope areas.

##### 43.1 Ontology

```text
src/ontology/
├── class.rs
├── constraint.rs
├── mod.rs
├── ontology.rs
├── property.rs
├── registry.rs
└── taxonomy.rs
```

###### `class.rs`

Owns the first-class ontology `Class` model and `ClassId`.

###### `constraint.rs`

Owns the extensible minimum constraint representation, including the structural representation needed for:

- domain/range constraints
- cardinality declarations
- disjointness declarations
- future constraint extensions

###### `ontology.rs`

Owns the ontology-level structure and coordination of ontology definitions.

###### `property.rs`

Owns ontology property definitions and their relationship to Phase 2 relationship predicates.

###### `registry.rs`

Owns the central ontology registry and controlled activation/access to immutable ontology snapshots.

This does not implement Phase 7 version management.

###### `taxonomy.rs`

Owns shared hierarchy infrastructure and distinct taxonomy semantics.

---

##### 43.2 Semantics

```text
src/semantics/
├── context.rs
├── interpretation.rs
├── meaning.rs
├── mod.rs
└── semantic_type.rs
```

###### `context.rs`

Owns reusable/ephemeral context representation and typed context dimensions.

###### `meaning.rs`

Owns first-class meaning representation.

###### `interpretation.rs`

Owns first-class interpretation representation and semantic connections produced from source/linguistic interpretation.

###### `semantic_type.rs`

Owns primary/additional semantic typing and the connection between direct type representation and ontology assertions.

---

##### 43.3 Entity Resolution

```text
src/resolution/
├── candidate.rs
├── crosswalk.rs
├── disambiguation.rs
├── matching.rs
├── mod.rs
└── resolver.rs
```

###### `candidate.rs`

Owns ephemeral candidate representations and deterministic candidate signals.

No persistent `CandidateId`.

###### `matching.rs`

Owns normalization and deterministic matching primitives.

###### `disambiguation.rs`

Owns candidate ranking, threshold/margin decisions, ambiguity handling, and resolution-state transitions.

###### `resolver.rs`

Owns the resolution pipeline and resolution interfaces.

###### `crosswalk.rs`

Owns external identifier crosswalk records.

These records remain distinct from canonical entity identities.

---

#### 44. Module Boundary Rules

The following boundaries must be maintained.

##### Ontology must not own

- graph traversal
- query planning
- storage
- full inference execution
- ingestion workflow

##### Semantics must not own

- Arabic linguistic analysis
- source ingestion
- physical persistence
- ML inference

##### Resolution must not own

- entity merging
- physical storage
- IndexAssignedId generation
- full provenance
- evidence management
- ML-only canonical identity decisions

##### No Phase 3 module may

- create a second identity framework
- bypass Core identity infrastructure
- directly mutate physical storage
- implement a second relationship vocabulary
- silently turn ML similarity into canonical identity

---

#### Testing Strategy

Phase 3 tests must cover only behavior that actually exists.

No speculative tests should be added for deferred functionality.

Every significant semantic subsystem should include both positive and negative cases.

---

##### 45.1 Ontology Tests

Test:

- class creation
- class identity
- class vs concept separation
- primary/additional semantic types
- ontology property definitions
- required domain/range for ontology-defined properties
- unconstrained generic relationships
- constraint representation
- cardinality representation
- disjointness representation
- multiple inheritance structure
- taxonomy structure
- distinction between subclass/instance predicates
- distinction between class and concept hierarchies
- part-whole semantic distinction
- central registry behavior
- immutable ontology snapshots
- controlled registry updates

---

##### 45.2 Semantic Tests

Test:

- first-class meaning
- first-class interpretation
- reusable contexts
- ephemeral contexts
- typed context dimensions
- direct lexical-to-concept mapping
- Sense-mediated lexical-to-concept mapping
- preservation of contextual distinctions
- semantic type behavior
- Arabic Engine ownership boundary

---

##### 45.3 Resolution Tests

Test:

- normalization
- exact identifier matching
- exact alias matching
- normalized matching
- transliteration matching
- source identifier matching
- context-assisted candidate generation
- graph-assisted candidate generation
- deterministic ranking
- threshold behavior
- candidate margin behavior
- ambiguous results
- unresolved results
- unknown results
- provisional results
- rejected results
- resolution without entity merge
- reversible decision representation
- external identifier crosswalks
- separation from IndexAssignedId
- graph-assisted circular-evidence protection

---

##### Negative Testing Requirements

Phase 3 must explicitly test invalid and unsafe semantic behavior.

Examples include:

```text
invalid domain
invalid range
incompatible semantic type
illegal constraint configuration
ambiguous candidate
below-threshold candidate
insufficient candidate margin
attempted provisional canonicalization
attempted entity merge through resolver
candidate incorrectly treated as identity
circular graph evidence treated as independent evidence
runtime mutation of an existing immutable ontology snapshot
```

Negative tests should validate the intended boundary rather than merely exercising error branches.

---

#### 47. Identity Testing Rule

Generated Core identities must not be compared against hard-coded generated values.

Tests may verify:

```text
id.is_present()
id_a != id_b
id_a == id_b
id propagates correctly
typed identity is accepted/rejected correctly
```

but must not depend on the exact SHA-derived generated string.

---

#### Completion Criteria

Phase 3 is complete when all of the following are true.

##### Ontology

- [ ] Class is a first-class ontology object.
- [ ] `ClassId` uses Core identity infrastructure.
- [ ] Class and Concept remain distinct.
- [ ] Semantic typing supports primary and additional types.
- [ ] Ontology assertions can represent type membership.
- [ ] Multiple inheritance is structurally representable.
- [ ] Relationship predicates can be formalized as ontology properties.
- [ ] Domain/range are supported for ontology-defined properties.
- [ ] Generic relationships may remain unconstrained.
- [ ] Minimum constraint framework exists.
- [ ] Cardinality can be represented without full enforcement.
- [ ] Disjointness can be represented without full enforcement.
- [ ] Taxonomy is distinct from ontology while sharing appropriate hierarchy infrastructure.
- [ ] Class hierarchy and concept hierarchy remain semantically distinct.
- [ ] Part-whole semantics remain distinct from taxonomy.
- [ ] Central ontology registry exists.
- [ ] Registry updates use controlled APIs.
- [ ] Ontology snapshots are immutable.
- [ ] Phase 3 does not implement ontology version management.

##### Semantics

- [ ] Meaning is first-class.
- [ ] Interpretation is first-class.
- [ ] Context supports reusable and ephemeral representations.
- [ ] Context dimensions are typed.
- [ ] Lexical-to-concept mapping supports both direct and Sense-mediated forms.
- [ ] Arabic linguistic interpretation remains owned by the Arabic Engine.
- [ ] Context can participate strongly in entity resolution without becoming canonical identity.

##### Entity Resolution

- [ ] Resolution pipeline interfaces exist.
- [ ] Deterministic normalization/matching exists.
- [ ] Candidate generation supports all planned non-ML strategies.
- [ ] Candidate ranking is deterministic.
- [ ] Absolute threshold and candidate margin are used.
- [ ] Full resolution state model exists.
- [ ] Provisional candidates do not create canonical entities.
- [ ] Resolution does not perform merging.
- [ ] Resolution decisions can be represented for later revision.
- [ ] External identifier crosswalks are separate records.
- [ ] `IndexAssignedId` remains outside the resolution system.
- [ ] Phase 3 stores only minimum resolution certainty/state.
- [ ] Rich evidence/provenance is deferred to Phase 4.
- [ ] Graph-assisted resolution is supported.
- [ ] Circular evidence is explicitly controlled.

##### Architecture

- [ ] Phase 2 relationship registry is extended rather than replaced.
- [ ] All new identities use Core `identity!`.
- [ ] No unnecessary identity types are introduced.
- [ ] No physical storage implementation is introduced.
- [ ] No full reasoning engine is introduced.
- [ ] No Python ML integration is introduced.
- [ ] No Indexing integration is added merely for completeness.
- [ ] Tests cover implemented behavior and negative boundaries.
- [ ] Deferred capabilities remain deferred to their designated phases.

---


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

The central principle of Phase 3 is:

> **The Knowledge Graph must know what things mean, what kinds of things exist, how semantic relationships are constrained, and how source-level references can be resolved toward canonical identities — without prematurely turning semantic hypotheses into canonical knowledge.**

The phase therefore deliberately separates:

```text
Class
    ≠ Concept

Lexical Form
    ≠ Sense
    ≠ Meaning
    ≠ Interpretation
    ≠ Entity

Relationship
    ≠ Ontology Property

Taxonomy
    ≠ Ontology

Resolution
    ≠ Merge

Candidate
    ≠ Canonical Entity

Resolution Confidence
    ≠ Claim Confidence

Context
    ≠ Identity

Graph Evidence
    ≠ Independent Evidence

Rule Definition
    ≠ Rule Execution

Ontology Snapshot
    ≠ Ontology Version Management
```

Phase 3 establishes the semantic foundation required for later evidence, ingestion, retrieval, storage, reasoning, ML, and Nizaam integration phases while keeping those later responsibilities outside the implementation boundary.

------------------------------------------------------------------------

### Phase 4: Evidence, Provenance, Authority, Uncertainty & Temporal Knowledge

#### Status

**Planned**

---

#### Goal

Phase 4 makes **knowledge quality, origin, trust, conflict, and time first-class parts of the Knowledge Graph (KG)**.

Phase 1 established identity and semantic-object foundations. Phase 2 established the central `KnowledgeAssertion` and relationship model. Phase 3 established ontology, semantics, and entity-resolution foundations. Phase 4 adds the epistemic and temporal context needed to understand why an assertion exists, where it came from, how it was produced, what authority applies, how certain it is, whether it is disputed, and when it is valid.

The phase therefore moves the conceptual model from:

```text
KnowledgeAssertion
    subject
    predicate
    object
```

toward:

```text
KnowledgeAssertion
├── subject
├── predicate
├── object
├── context
├── qualifiers
├── status
├── evidence
├── provenance
├── authority
├── confidence / uncertainty
├── contradiction relationships
└── temporal validity
```

The surrounding concepts remain distinct. Phase 4 does **not** introduce a catch-all epistemic-metadata object.

---

#### 2. Scope

Phase 4 covers:

```text
evidence
source support
verification

provenance
lineage
origin
audit information

authority
reliability
scholarly status

confidence
uncertainty
contradiction

temporal validity
temporal intervals
temporal relationships
valid time
system time
```

Planned modules:

```text
src/evidence/
src/provenance/
src/authority/
src/uncertainty/
src/temporal/
```

The current planned structure is:

```text
src/evidence/
├── assertion.rs
├── evidence.rs
├── mod.rs
├── support.rs
└── verification.rs

src/provenance/
├── audit.rs
├── lineage.rs
├── mod.rs
├── origin.rs
└── provenance.rs

src/authority/
├── authority.rs
├── mod.rs
├── reliability.rs
└── scholarly.rs

src/uncertainty/
├── confidence.rs
├── contradiction.rs
├── mod.rs
└── status.rs

src/temporal/
├── interval.rs
├── mod.rs
├── temporal_relation.rs
└── validity.rs
```

`evidence/assertion.rs` is retained only if implementation requires a dedicated assertion/evidence attachment type. It must not create a second assertion model.

---

#### 3. Inherited Architectural Rules

##### 3.1 Core identity remains authoritative

All Phase 4 identity types use the established Core `identity!` mechanism. Phase 4 must not introduce UUIDs, ULIDs, custom random IDs, or a second identity framework.

Tests must not assert exact generated ID values. They should verify identity presence, type distinction, propagation, equality where semantically required, and behavior.

##### 3.2 KnowledgeAssertion remains canonical

Phase 4 extends the existing `KnowledgeAssertion`; it does not replace it with another claim model.

Evidence, provenance, authority, uncertainty, contradiction, and temporal information qualify knowledge. They do not become a competing assertion system.

##### 3.3 Evidence is not provenance

```text
Evidence
    → Why is this assertion supported, refuted, or qualified?

Provenance
    → Where did this knowledge come from and how was it produced?
```

The same assertion can have both.

##### 3.4 Core provenance is not KG provenance

Core owns generic runtime/artifact/operation provenance. KG owns knowledge-specific provenance such as source/version, extraction, transformation, review, derivation, and lineage.

```text
Core provenance
    runtime / operation / infrastructure lineage

KG provenance
    knowledge / source / transformation / review lineage
```

The KG must reuse the Core boundary rather than replacing it.

##### 3.5 Resolution state remains separate from epistemic state

Phase 3 resolution states remain:

```text
Resolved
Ambiguous
Unresolved
Unknown
Provisional
Rejected
```

Phase 4 epistemic states are:

```text
Known
Unknown
Uncertain
Ambiguous
Disputed
Conflicting
```

They are related but not interchangeable.

```text
ResolutionState::Ambiguous
        may contribute to
EpistemicState::Uncertain
```

but:

```text
ResolutionState != EpistemicState
```

This preserves the Phase 3 distinction between resolution confidence and claim/evidence confidence.

##### 3.6 Phase boundaries remain strict

Phase 4 must not prematurely implement Phase 5 ingestion governance, Phase 6 full query/search, Phase 7 physical storage and versioning, Phase 8 reasoning execution, Phase 9 Python/gRPC, or Phase 10 final conformance/hardening.

---

#### 4. Final Decisions

##### Q1 — Evidence representation

**Decision: A — Evidence is a first-class object.**

Evidence receives its own typed identity and semantic representation. It must be independently addressable and must not be reduced to an untyped field on `KnowledgeAssertion`.

##### Q2 — Evidence role

**Decision: C — Extensible evidence-role vocabulary.**

Initial roles may include:

```text
supports
refutes
qualifies
corroborates
contextualizes
illustrates
```

The vocabulary must be extensible without redesigning `Evidence`.

##### Q3 — Direct vs indirect evidence

**Decision: C — Represent direct/indirect support through the evidence mechanism.**

Use mechanisms such as:

```text
DirectSource
DerivedSupport
```

rather than a permanent boolean such as `is_direct`.

Phase 4 represents support chains; it does not execute the full reasoning system.

##### Q4 — Evidence source addressability

**Decision: D — Support structured source references and textual spans.**

Evidence can identify precise source locations where available:

```text
Source
SourceVersion
Document
Passage
Section
Page
Paragraph
Verse / Hadith reference
Text span
```

The model must not force every source into one text-offset representation.

##### Q5 — Verification

**Decision: C — Hybrid verification model.**

Verification uses references and separate verification records where needed. It must not collapse into a single `verified: bool` field.

A verification record may describe who/what verified an item, when, what was evaluated, the result, and which evidence/source was considered.

---

#### 5. Provenance Decisions

##### Q6 — Core vs KG provenance

**Decision: C — Layered provenance.**

Maintain the explicit boundary:

```text
Core provenance
        +
KG knowledge provenance
```

##### Q7 — Provenance model maturity

**Decision: B initially, migration-friendly toward C.**

Start with an `Activity + Agent` model or equivalent structured lineage representation. Design it so it can evolve toward richer PROV-style semantics later without a complete rewrite.

Do not implement a standards-complete provenance framework prematurely.

##### Q8 — Provenance mutability

**Decision: A — Provenance is an immutable value attached to the knowledge object.**

A historical provenance record is not edited in place. New knowledge activity creates new provenance information.

##### Q9 — Agent model

**Decision: C — Generic Agent + AgentType.**

The model can represent:

```text
Human
Organization
Software
Pipeline
Model
System
Other / Unknown
```

without hard-coding every future actor category.

##### Q10 — Historical provenance

**Decision: B — Append-only historical provenance.**

Historical lineage remains inspectable rather than being overwritten.

Conceptually:

```text
Source / SourceVersion
        ↓
Activity
        ↓
Transformation / Extraction
        ↓
Review / Derivation
        ↓
Knowledge object
```

##### Q11 — Audit vs provenance

**Decision: B — Audit is separate from provenance.**

```text
Provenance
    epistemic lineage: how knowledge came to exist

Audit
    operational history: what changed, when, and through what operation
```

Audit is not a substitute for provenance and provenance is not a logging system.

---

#### 6. Authority Decisions

##### Q12 — Authority dimensions

**Decision: C — Multi-dimensional authority.**

Authority may include:

```text
source authority
source reliability
authentication
scholarly status
verification
human review
machine extraction
curation status
process reliability
```

There is no universal trust score replacing these dimensions.

##### Q13 — Authority attachment

**Decision: C — Authority applies to both Sources and Assertions.**

Source-level authority and assertion-level authority are distinct because a source can have strong authority while a particular extraction or interpretation still requires evaluation.

##### Q14 — Reliability

**Decision: C — Distinguish source reliability and process/extraction reliability.**

At minimum:

```text
SourceReliability
ProcessReliability
```

A reliable source with poor extraction and a reliable extraction process applied to a poor source must remain distinguishable.

##### Q15 — Scholarly status

**Decision: C — Generic status + domain-specific vocabulary.**

The generic model remains extensible so domain-specific scholarly classifications can be represented without forcing domain terminology into the universal base model.

##### Q16 — Authority evaluation profiles

**Decision: C — Generic authority model + domain-specific evaluation profiles.**

The core model remains generic; domain-specific profiles interpret authority, reliability, and scholarly status according to their own evidence and evaluation practices.

---

#### 7. Uncertainty and Confidence Decisions

##### Q17 — Confidence representation

**Decision: C — Hybrid confidence representation.**

Confidence can preserve a representation/value together with its basis and evaluation context. It must not erase the underlying evidence/support information.

##### Q18 — Confidence ownership

**Decision: B — Confidence belongs to evidence/support evaluation.**

Conceptually:

```text
Evidence
   ↓
Support Evaluation
   ↓
Confidence
```

Multiple pieces of evidence can therefore have different evaluation contexts.

##### Q19 — Confidence calculation

**Decision: A — Represent confidence, but do not define a calculation formula in Phase 4.**

Phase 4 must not prematurely freeze a formula such as:

```text
confidence = source_weight × evidence_weight × authority_weight
```

The architecture requires confidence to be evidence-based, reproducible, and explainable, but that does not require a mathematical evaluator now.

Phase 4 defines:

```text
confidence representation
confidence context
confidence basis
```

A future `ConfidenceEvaluator` may be introduced only when later evidence justifies it.

This avoids fake precision.

##### Q20 — Status model

**Decision: B — Extend the Phase 2 status model.**

Phase 4 builds on the existing assertion-status architecture rather than introducing an unrelated status system. The semantic state vocabulary can represent:

```text
Known
Unknown
Uncertain
Ambiguous
Disputed
Conflicting
```

##### Q21 — Resolution state vs epistemic state

**Decision: C — Keep them separate, with explicit relationships.**

Resolution asks:

```text
What entity/reference does this source-level thing refer to?
```

Epistemic state asks:

```text
What is the knowledge status of this assertion?
```

They can interact, but must not be merged into one enum.

---

#### 8. Contradiction Decisions

##### Q22 — Contradiction representation

**Decision: C — First-class Contradiction object.**

A contradiction is not merely:

```text
contradiction: bool
```

It can relate multiple assertions and preserve type, context, status, and supporting information.

##### Q23 — Contradiction detection

**Decision: B — Basic deterministic contradiction detection.**

Phase 4 may detect structurally explicit contradictions, for example when the same subject/predicate/context has mutually incompatible values according to known relationship characteristics.

Advanced semantic contradiction detection belongs to later reasoning.

##### Q24 — Current view vs conflict view

**Decision: C — Policy-controlled current view plus explicit conflict query.**

A current-view policy may select active knowledge, but conflicting assertions are not physically erased. Applications must be able to inspect the complete conflict set.

##### Q25 — Contradiction resolution

**Decision: C — Represent now; resolve later.**

Phase 4 preserves conflict and supporting information. Later reasoning may use evidence, authority, confidence, temporal context, ontology, and rules to reason about the conflict.

---

#### 9. Temporal Decisions

##### Q26 — Temporal value

**Decision: C — Rich `TemporalValue`.**

Support:

```text
Instant
Interval
Approximate
OpenEnded
Unknown
```

The model must not assume every date is exact.

##### Q27 — Interval boundaries

**Decision: C — Flexible interval boundary model.**

Boundary semantics can represent inclusive, exclusive, open, or unknown boundaries as appropriate.

##### Q28 — Approximate dates

**Decision: A — Center value + approximate flag.**

An approximate date keeps an explicit central/representative value plus approximation semantics. Approximate information must not silently become an exact instant.

##### Q29 — Valid time and system time

**Decision: C — Implement both valid time and system time.**

```text
Valid Time
    when the represented knowledge is true/applicable

System Time
    when the system recorded/changed the representation
```

##### Q30 — Temporal validity attachment

**Decision: C — Attach temporal validity to the assertion/canonical semantic unit through a shared temporal model.**

Temporal validity is semantic qualification, not merely a database timestamp.

##### Q31 — Temporal relationship model

**Decision: C — Shared temporal primitives with distinct temporal semantics.**

The same interval/instant primitives can support both temporal validity and temporal relationships, but the meanings remain distinct.

For example:

```text
Assertion valid during 600–610
```

is different from:

```text
Event A BEFORE Event B
```

##### Q32 — System time semantics

**Decision: C — System time is provenance/activity time.**

Valid time belongs to temporal knowledge. System time belongs to provenance/activity/history semantics.

---

#### 10. Mutability and Attachment Decisions

##### Q33 — Phase 4 metadata mutability

**Decision: A — Phase 4 metadata may be mutable where it represents current semantic metadata.**

This does **not** make historical provenance mutable.

```text
Current semantic metadata
    may change through controlled APIs

Historical provenance / audit
    remains immutable / append-only
```

Thus the intended model is:

```text
mutable current view
        +
immutable historical record
```

##### Q34 — Attachment model

**Decision: C — Hybrid structural metadata + semantic relationships.**

Intrinsic metadata can be structural:

```text
TemporalValidity
current epistemic status
basic authority metadata
```

Explicit semantic relationships remain relationships:

```text
Evidence supports Assertion
Evidence refutes Assertion
Assertion conflicts with Assertion
Verification evaluates Evidence
Provenance records activity involving an object
```

The implementation must avoid both "everything is a field" and "everything is a graph assertion".

##### Q35 — Generic epistemic metadata abstraction

**Decision: A — No generic `EpistemicMetadata` abstraction.**

Do not create:

```text
EpistemicMetadata {
    evidence,
    provenance,
    authority,
    confidence,
    temporal,
    ...
}
```

Instead keep the semantic concepts explicit:

```text
Evidence
Provenance
Authority
Uncertainty
Contradiction
Temporal
```

---

#### 11. Evidence Architecture

Conceptually:

```text
                    KnowledgeAssertion
                           │
                           │ supports / refutes /
                           │ qualifies / ...
                           ▼
                    ┌───────────────┐
                    │    Evidence   │
                    ├───────────────┤
                    │ EvidenceId    │
                    │ Role          │
                    │ Mechanism     │
                    │ SourceRef     │
                    │ Location/Span │
                    │ Evaluation    │
                    └───────┬───────┘
                            │
                            ▼
                       Verification
```

Evidence may be direct or derived. It may support, refute, qualify, corroborate, or contextualize according to the extensible vocabulary.

Evidence must remain distinguishable from source, provenance, authority, confidence, and verification.

---

#### 12. Provenance Architecture

Initial conceptual lineage:

```text
SOURCE / SOURCE VERSION
          │
          ▼
       ACTIVITY
          │
     ┌────┼───────────┐
     ▼    ▼           ▼
   AGENT SOFTWARE  TRANSFORMATION
     │                / EXTRACTION
     └───────┬──────────────┘
             ▼
       KNOWLEDGE OBJECT
             │
       ┌─────┼─────────┐
       ▼     ▼         ▼
     REVIEW DERIVATION MODIFICATION
```

This is an initial Activity/Agent model designed to evolve toward richer provenance semantics later.

Provenance records are immutable historical values and historical provenance is append-only.

---

#### 13. Authority Architecture

Authority is multi-dimensional:

```text
Authority
├── source authority
├── source reliability
├── process reliability
├── authentication
├── scholarly status
├── verification
├── review status
└── curation status
```

There is no universal trust score replacing these dimensions.

Authority can apply to both:

```text
Source
Assertion
```

and domain-specific evaluation profiles can interpret the generic representation.

---

#### 14. Uncertainty Architecture

Epistemic status must preserve meaningful distinctions:

```text
Known
Unknown
Uncertain
Ambiguous
Disputed
Conflicting
```

The model must not collapse all uncertainty into one boolean or one universal number.

Confidence is primarily a support/evidence evaluation. Phase 4 represents it but deliberately does not define a universal calculation formula.

---

#### 15. Contradiction Architecture

Contradiction is first-class:

```text
Assertion A
     │
     └──────┐
            ▼
      Contradiction
            ▲
     ┌──────┘
     │
Assertion B
```

Both assertions can coexist. A current-view policy may select one for a particular use, but the underlying conflict remains available.

Phase 4 performs only basic deterministic contradiction detection. Full resolution is deferred to reasoning work.

---

#### 16. Temporal Architecture

The temporal foundation is:

```text
TemporalValue
├── Instant
├── Interval
├── Approximate
├── OpenEnded
└── Unknown
```

An interval contains start/end boundary semantics. Temporal validity qualifies a semantic assertion/unit.

```text
KnowledgeAssertion
       │
       ▼
TemporalValidity
       │
       └── Valid Time
```

System time is represented through provenance/activity semantics:

```text
Valid Time
    → when knowledge applies

System Time
    → when the KG recorded/changed it
```

The design is compatible with later bitemporal storage semantics without implementing the physical storage model in Phase 4.

---

#### 17. Module Responsibilities

##### `src/evidence/`

Owns:

```text
Evidence identity
Evidence roles
Direct/derived support mechanisms
Source references
Source locations/spans
Support relationships
Verification integration
```

Suggested file responsibilities:

```text
evidence.rs
    Evidence object and identity

support.rs
    support/refutation/qualification mechanisms

verification.rs
    verification records/references

assertion.rs
    assertion/evidence attachment if required

mod.rs
    public exports
```

##### `src/provenance/`

Owns:

```text
knowledge provenance
activities
agents
origin
lineage
historical provenance
audit
```

Suggested file responsibilities:

```text
provenance.rs
    core provenance representation

origin.rs
    origin/source-of-knowledge representation

lineage.rs
    lineage relationships and historical chain

audit.rs
    operational audit information

mod.rs
    public exports
```

##### `src/authority/`

Owns:

```text
authority
reliability
scholarly status
evaluation profiles
source-level authority
assertion-level authority
```

##### `src/uncertainty/`

Owns:

```text
confidence representation
epistemic status
contradiction
basic deterministic contradiction detection
```

Resolution state remains in Phase 3 `resolution`.

##### `src/temporal/`

Owns:

```text
TemporalValue
Instant
Interval
Approximate
OpenEnded
Unknown
interval boundaries
temporal validity
temporal relationships
```

It is not a storage/versioning subsystem.

---

#### 18. Cross-Module Ownership

```text
Evidence
    → evidence semantics

Provenance
    → knowledge lineage

Authority
    → authority/reliability evaluation metadata

Uncertainty
    → epistemic state, confidence representation, contradiction

Temporal
    → temporal primitives and semantics

KnowledgeAssertion
    → canonical semantic object being qualified

Graph
    → graph structure

Ontology
    → semantic constraints/types

Resolution
    → source-to-canonical identity resolution
```

Related concepts may interact, but they must not be collapsed.

---

#### 19. Core and Indexing Boundaries

Core owns:

```text
engine runtime
lifecycle
Control Plane
capability dispatch
operation/execution context
transport
serialization
security
runtime provenance
observability
```

KG Phase 4 owns:

```text
knowledge evidence
knowledge provenance
authority
uncertainty
contradiction
temporal knowledge
```

Indexing remains an infrastructure/indexing subsystem. Phase 4 must not redefine `IndexAssignedId`, `IndexId`, `IndexVersion`, or other Indexing-owned contracts.

Important distinctions:

```text
Evidence          != Index entry
Provenance        != Index version
Authority         != Index ranking
Confidence        != Search score
```

---

#### 20. Phase 5 / Phase 6 / Phase 7 / Phase 8 Boundaries

##### Phase 5

Phase 5 will consume Phase 4 metadata during:

```text
raw
 ↓
normalized
 ↓
mapped
 ↓
validated
 ↓
approved
 ↓
published
```

Phase 4 does not implement that governance workflow.

##### Phase 6

Phase 6 may use evidence, authority, uncertainty, and temporal filters during retrieval. Phase 4 itself does not implement the query planner/search engine.

##### Phase 7

Phase 7 owns physical persistence and versioning. Temporal validity must not be confused with KG versioning.

```text
Temporal validity
    → When was the knowledge true?

KG versioning
    → Which KG state/version contained it?
```

##### Phase 8

Phase 8 will reason over the structures created in Phase 4.

```text
Phase 4
    represents contradiction/evidence

Phase 8
    reasons about contradiction/evidence
```

---

#### 21. Implementation Order

Implement one responsibility at a time:

```text
1. Temporal primitives
2. Evidence foundation
3. Verification
4. Provenance foundation
5. Authority
6. Uncertainty/status
7. Confidence representation
8. Contradiction
9. Cross-module attachment/integration
10. Public module exports
11. Phase 4 integration tests
```

The exact file order may change for Rust dependency reasons, but semantic ownership must remain unchanged.

Do not implement multiple future-phase systems merely to make Phase 4 compile.

---

#### Testing Strategy

Phase 4 testing verifies actual behavior, not just compilation.

Tests must cover positive behavior, negative behavior, identity/type behavior, boundary conditions, and cross-module integration.

##### Evidence

Verify:

```text
evidence can be created
identity is typed
role is preserved
direct/derived mechanisms are distinguishable
structured source references survive
text spans survive where applicable
multiple evidence records can coexist
support/refutation/qualification behave correctly
```

##### Verification

Verify:

```text
verification can reference evidence
verification result is preserved
verification metadata is preserved
multiple verification records can coexist
historical verification is not silently overwritten
```

##### Provenance

Verify:

```text
activity is represented
agent is represented
source/source-version origin is preserved
transformation/extraction is preserved
provenance records remain immutable
multiple historical records coexist
lineage is inspectable
audit remains separate
```

##### Authority

Verify:

```text
authority attaches to sources
authority attaches to assertions
source reliability differs from process reliability
scholarly status is represented
multiple dimensions coexist
domain-specific status vocabulary is possible
```

##### Uncertainty

Verify:

```text
epistemic status is represented
known/unknown/uncertain/ambiguous/disputed/conflicting are distinct
resolution state remains distinct
confidence can be represented at support/evidence evaluation
no confidence formula is required
```

##### Contradiction

Verify:

```text
contradiction can relate assertions
basic deterministic detection works
non-contradictory assertions are not falsely classified
conflicting assertions remain preserved
current-view selection does not erase conflicts
explicit conflict inspection remains possible
```

##### Temporal

Verify:

```text
Instant
Interval
Approximate
OpenEnded
Unknown
```

and boundary behavior including inclusive, exclusive, open, and unknown boundaries where the final API supports them.

Also verify:

```text
valid time != system time
approximate dates are not silently exact
temporal validity attaches to the intended semantic unit
temporal relationships remain distinct from validity
```

---

##### Negative Testing

Every major subsystem must have meaningful negative tests based on actual API invariants.

Potential cases include:

```text
invalid evidence reference
invalid source span
invalid verification target
invalid provenance activity
invalid agent reference
invalid authority attachment
invalid confidence representation
invalid contradiction relationship
invalid temporal interval
invalid boundary combination
invalid epistemic-state transition where transitions are restricted
```

Do not add artificial restrictions solely to manufacture negative tests.

---

#### 24. Determinism

Phase 4 correctness behavior that is defined as deterministic must remain deterministic:

```text
basic contradiction detection
structural validation
temporal boundary validation
identity type behavior
simple confidence representation rules
```

Phase 4 must not depend on external ML/LLM behavior for correctness.

---

#### 25. Documentation Requirements

Every Phase 4 module should document what it owns, what it does not own, its relationship to `KnowledgeAssertion`, Core, Phase 3, and later phases.

The following distinctions are architectural invariants and must be documented:

```text
Evidence vs Provenance
Provenance vs Audit
Authority vs Confidence
Resolution State vs Epistemic State
Valid Time vs System Time
Temporal Validity vs Temporal Relationship
Current Metadata vs Historical Provenance
```

---

#### 27. Final Verification Before Completion

Compilation alone does not mean Phase 4 is complete.

At final completion, verification should include, where applicable:

```text
cargo fmt --check
cargo clippy
cargo check
cargo build
cargo test
cargo test --doc
```

The final review must also check:

```text
public API consistency
module export correctness
architectural boundary violations
unintended dependency direction
unused abstractions
premature future-phase functionality
```

Only after successful implementation and verification should the scope status move from:

```text
Planned
```

to:

```text
Completed
```

---

#### 28. Explicitly Deferred

The following remain outside Phase 4:

```text
full ingestion governance
approval workflows
publication workflows
source adapters
policy-based governance

full KG query language
full query planner
search/ranking implementation
retrieval engine

physical KG persistence
storage-provider selection
physical indexing design
full KG versioning
performance architecture

full inference engine
advanced contradiction resolution
probabilistic reasoning
large-scale reasoning

Python bindings
gRPC implementation

final Nizaam ecosystem conformance
```

Phase 4 may expose the semantic structures later phases need, but must not implement those systems prematurely.

---

#### Non-Goals

Phase 4 must not become:

```text
a database
```

It defines storage-independent semantic structures.

It must not become:

```text
a query engine
```

Query/traversal/search belong to Phase 6.

It must not become:

```text
a governed ingestion pipeline
```

That belongs to Phase 5.

It must not become:

```text
a reasoning engine
```

Inference execution belongs to Phase 8.

It must not become:

```text
a second Core provenance system
```

Core owns generic runtime/artifact provenance.

It must not become:

```text
a universal trust-score engine
```

Authority and confidence remain explicit and evidence-based.

---

#### 30. Final Phase 4 Architecture

The resulting conceptual model is:

```text
                         ┌──────────────────────┐
                         │ KnowledgeAssertion   │
                         └──────────┬───────────┘
                                    │
             ┌──────────────────────┼──────────────────────┐
             │                      │                      │
             ▼                      ▼                      ▼
        ┌─────────┐            ┌───────────┐         ┌───────────┐
        │ Evidence│            │Provenance │         │ Authority │
        └────┬────┘            └─────┬─────┘         └─────┬─────┘
             │                       │                     │
             ▼                       ▼                     ▼
       Verification             Lineage/Origin        Reliability
             │                       │                 Scholarly
             ▼                       ▼
        Support                  Audit
             │
             ▼
        Confidence
             │
             ▼
      Epistemic State
             │
             ▼
      Contradiction
             │
             ▼
     Temporal Validity
             │
             ▼
       Temporal Model
```

Surrounding layers remain:

```text
                         ┌──────────────────────────┐
                         │       Phase 4 KG         │
                         │                          │
                         │ Evidence                 │
                         │ Provenance               │
                         │ Authority                │
                         │ Uncertainty              │
                         │ Temporal                 │
                         └────────────┬─────────────┘
                                      │
                         KnowledgeAssertion
                                      │
                         ┌────────────┴─────────────┐
                         │                          │
                  Phase 3 Semantics           Phase 2 Relations
                         │                          │
                         └────────────┬─────────────┘
                                      │
                              Core Infrastructure
```

---

#### Completion Criteria

Phase 4 is complete when the KG can answer:

```text
What is the assertion?
Why does it exist?
Where did it come from?
How was it produced?
What authority applies?
How certain is it?
Is it disputed?
When is it valid?
```

Required completion checklist:

##### Evidence

```text
[ ] Evidence is first-class.
[ ] Evidence roles are extensible.
[ ] Direct and derived support are distinguishable.
[ ] Structured source references are supported.
[ ] Text/source spans are supported where applicable.
[ ] Verification uses the hybrid model.
```

###### Provenance

```text
[ ] KG provenance is distinct from Core provenance.
[ ] Activity and Agent are represented.
[ ] Provenance records are immutable historical values.
[ ] Historical provenance is append-only.
[ ] Origin and lineage are represented.
[ ] Audit is separate from provenance.
[ ] The model can evolve toward richer provenance semantics.
```

###### Authority

```text
[ ] Authority is multi-dimensional.
[ ] Sources can carry authority.
[ ] Assertions can carry authority.
[ ] Source reliability is distinct from process reliability.
[ ] Scholarly status is extensible.
[ ] Domain-specific evaluation profiles can be represented.
```

###### Uncertainty and contradiction

```text
[ ] Epistemic status is represented.
[ ] Confidence is represented without a premature formula.
[ ] Confidence belongs to support/evidence evaluation.
[ ] Resolution state remains distinct from epistemic state.
[ ] Contradiction is first-class.
[ ] Basic deterministic contradiction detection exists.
[ ] Conflicting assertions can coexist.
```

###### Temporal

```text
[ ] Instant is represented.
[ ] Interval is represented.
[ ] Approximate values are represented.
[ ] Open-ended values are represented.
[ ] Unknown values are represented.
[ ] Flexible interval boundaries are represented.
[ ] Valid time is represented.
[ ] System time is represented through provenance/activity semantics.
[ ] Temporal validity is distinct from temporal relationships.
```

###### Cross-cutting

```text
[ ] Phase 2 KnowledgeAssertion remains canonical.
[ ] Phase 3 resolution semantics remain intact.
[ ] Core identity remains authoritative.
[ ] Core runtime provenance is reused rather than duplicated.
[ ] Indexing identities/contracts are not redefined.
[ ] No physical storage provider is required.
[ ] No ingestion governance workflow is implemented prematurely.
[ ] No full reasoning engine is implemented prematurely.
[ ] Public exports are coherent.
[ ] Positive and negative tests cover actual behavior.
```

---


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

Phase 4 establishes the KG's **epistemic foundation**.

The KG will no longer represent only:

```text
A --relation--> B
```

or merely:

```text
Assertion(A, relation, B)
```

It can represent the surrounding knowledge conditions:

```text
Assertion
    + Evidence
    + Provenance
    + Authority
    + Uncertainty
    + Contradiction
    + Temporal validity
```

while preserving the distinctions between those concepts.

The final design rules are:

```text
Evidence explains support.
Provenance explains origin and lineage.
Audit explains operational change history.
Authority describes source/assertion authority dimensions.
Confidence describes support evaluation.
Epistemic state describes knowledge status.
Contradiction preserves conflict explicitly.
Temporal validity describes when knowledge applies.
System time describes when the system recorded/activity occurred.
Resolution state remains separate from epistemic state.
Core provenance remains separate from KG knowledge provenance.
```

This gives later phases a stable foundation for governed ingestion, evidence-aware retrieval, physical persistence/versioning, reasoning, external integration, and final Nizaam conformance without prematurely implementing those systems inside Phase 4.

------------------------------------------------------------------------

### Phase 5: Knowledge Ingestion, Validation & Governance

#### Status

**Status:** Planned
**Phase:** 5
**Depends on:** Phases 0–4
**Primary module:** `src/ingestion/`

---

#### Goal

Phase 5 creates the governed boundary through which external/source material becomes canonical Knowledge Graph state.

The phase is not merely an importer. It establishes a controlled transformation lifecycle:

```text
source material
      ↓
acquisition
      ↓
raw
      ↓
normalized
      ↓
mapped
      ↓
deduplicated
      ↓
entity resolution
      ↓
validated
      ↓
curated / reviewed when required
      ↓
approved
      ↓
index-readiness coordination
      ↓
canonical publication
      ↓
index synchronization
```

The current scope defines the core lifecycle as:

```text
raw
 ↓
normalized
 ↓
mapped
 ↓
validated
 ↓
approved
 ↓
published
```

Phase 5 extends this lifecycle with the operational controls required by the final decisions in this plan while preserving the scope boundary.

The goal is to make ingestion governed, reproducible, auditable, source-aware, and safe without prematurely implementing Phase 7 physical persistence/versioning, Phase 8 reasoning, Phase 9 Python/gRPC integration, or Phase 10 final conformance.

---

#### 2. Architectural Position

The phase order remains:

```text
Phase 0
Engine / Runtime / Core Integration
        ↓
Phase 1
Identity + Core Semantic Objects
        ↓
Phase 2
Knowledge Assertion + Relationship Model
        ↓
Phase 3
Ontology + Semantics + Entity Resolution
        ↓
Phase 4
Evidence + Provenance + Authority + Uncertainty + Temporal Knowledge
        ↓
Phase 5
Knowledge Ingestion + Validation + Governance
        ↓
Phase 6
Query + Retrieval + Traversal
        ↓
Phase 7
Storage + Versioning + Performance
        ↓
Phase 8
Reasoning + Inference
        ↓
Phase 9
Rust ↔ Python / ML Integration
        ↓
Phase 10
Nizaam Integration + Conformance + Hardening
```

Phase 5 consumes the semantic, resolution, evidence, provenance, authority, uncertainty, contradiction, and temporal primitives already established by earlier phases.

Phase 5 operationalizes those primitives; it does not redefine them.

---

#### 3. Governing Principle

The most important rule of Phase 5 is:

```text
Ingestion lifecycle state
        ≠
Knowledge truth / epistemic state
```

For example:

```text
Governance state:
Approved

Epistemic state:
Disputed
```

is valid.

Likewise:

```text
Governance state:
Quarantined

Epistemic state:
Unknown
```

is valid.

Phase 5 must never treat a governance transition such as `Published` as automatic proof of truth, authenticity, certainty, authority, or scholarly acceptance.

---

#### 4. Final Resolutions of Decision Conflicts

The 50 decisions are broadly compatible, but several required explicit interpretation before implementation.

##### 4.1 Q11 vs Q13 — Fatal findings

The decisions can appear contradictory because Q11 defines:

```text
fatal → rejected
recoverable → quarantined
```

while Q13 says even a fatal warning should not block publication and should instead be sent to quarantine.

The final interpretation is:

```text
Validation finding
    ↓
severity classification
    ├── Info
    ├── Warning
    ├── Error
    └── Fatal
```

A `Fatal` finding blocks publication of the affected candidate.

It does **not** block the whole ingestion batch unless a separate batch-level governance rule says so.

Therefore:

```text
Fatal candidate
    → quarantine / reject from active publication

Other candidates in same batch
    → continue independently
```

The term `fatal warning` must not be used as an implementation type. It will be a **fatal validation finding**.

Quarantine is therefore a publication barrier for that candidate, not a failure of the entire ingestion run.

---

##### 4.2 Q14 vs current scope — Approval model

The current scope originally describes an initial single global approval rule with future policy-based governance.

Q14 deliberately chooses source-class-based approval.

Therefore the Phase 5 implementation plan supersedes the older wording with this refined model:

```text
Initial governance
    = source-class based approval rules

Implementation mechanism
    = configuration-driven rules

Future evolution
    = general policy abstraction
```

This is an intentional refinement of the Phase 5 design, not an accidental architectural conflict.

---

##### 4.3 Q3 vs Q4 — Raw persistence ownership

Q3 assigns raw persistence responsibility to the source-adapter side, while Q4 selects reuse of Core artifact storage.

These responsibilities are compatible when separated correctly:

```text
Source Adapter
    = owns raw material capture semantics

Core Artifact Storage
    = owns the generic storage mechanism

Phase 5
    = defines the ingestion-level RawMaterial contract and persistence expectations
```

A source adapter must not implement a second generic storage system.

It may write raw material through the agreed generic storage boundary provided by Core.

---

##### 4.4 Q8 vs Q10 — Resolution before validation

Q8 chooses entity resolution before the main validation phase, while Q10 chooses layered validation.

This is resolved by splitting validation into stages:

```text
pre-resolution structural validation
        ↓
normalization
        ↓
mapping
        ↓
source-level deduplication
        ↓
entity resolution
        ↓
post-resolution semantic / ontology / reference validation
        ↓
governance validation
```

Therefore Q8 does not mean that absolutely no validation occurs before resolution. It means **canonical semantic/entity validation is reached after resolution**, while safe structural checks are allowed earlier.

Every validation layer is designed as a pipeline-compatible unit from the beginning.

---

##### 4.5 Q9 vs Q35 — Ambiguous/provisional publication

The initial behavior for ambiguous resolution is:

```text
Ambiguous
    → Quarantine
```

Future behavior may allow a provisional path with human review.

`Provisional` must not silently become equivalent to final `Published`.

The final lifecycle distinction is:

```text
Provisional
    = controlled non-final semantic state

Published
    = approved canonical state exposed by the Phase 5 publication boundary
```

A future provisional-publication capability therefore requires explicit governance semantics and human review rather than bypassing approval.

---

##### 4.6 Q20, Q35 and Phase 7 — Logical publication vs physical versioning

Q20 chooses a new immutable KG state, while Q35 defines Phase 5 publication as acceptance into canonical KG state and Phase 7 owns physical storage/versioning.

The final boundary is:

```text
Phase 5
    = logical canonical publication

Phase 7
    = durable physical persistence + snapshots + version history
```

When Phase 5 says a candidate becomes published, it means the candidate has crossed the logical canonical-state boundary.

It does not mean Phase 5 implements the full storage/version system described for Phase 7.

Semantic records accepted as published are treated as immutable from the publication operation's perspective, while durable physical snapshot/version management remains a Phase 7 responsibility.

---

##### 4.7 Q39, Q40 and Q35 — Indexing and publication

This is the most important cross-engine refinement.

The following remain separate:

```text
KG publication
    ≠
Indexing publication
```

However, the user decision requires ingestion to wait for indexing readiness before final publication.

The compatible interpretation is a two-step coordination model:

```text
Approved
   ↓
Indexing preparation request
   ↓
Index prepared + validated + ready
   ↓
Canonical KG publication
   ↓
Index activation / synchronization
   ↓
Indexed
```

Therefore:

- Indexing remains a separate engine and separate operation.
- KG does not transfer semantic ownership to Indexing.
- Phase 5 may require an Indexing readiness acknowledgement before crossing its publication boundary.
- Indexing activation after publication remains a separate synchronization operation.
- A failed index activation after canonical publication must be represented explicitly as an indexing synchronization condition, not disguised as a KG semantic failure.

This uses the Indexing Engine's existing conceptual build lifecycle of `build → validate → ready → publish → active` without making the KG engine implement Indexing internals.

No distributed transaction system is introduced in Phase 5.

---

##### 4.8 Q45 — Publishing non-authentic material

The decision to expose non-authentic material is compatible with the KG architecture only when `non-authentic` is treated as an explicit semantic/authority status rather than as an override of validation or governance.

The rule is:

```text
Source material can be represented and surfaced
    even when its authenticity is low or disputed

but

Source material being represented
    ≠
Canonical KG endorsement that the material is true/authentic
```

For examples such as Hadith, Tafsir, Fiqh, or other scholarly material:

```text
material/source claim
    + authority metadata
    + reliability metadata
    + scholarly status
    + epistemic status
    + provenance
```

may be preserved and surfaced.

A fatal structural, security, corruption, or governance violation still cannot be published merely because the source itself is described as non-authentic.

This preserves the Phase 4 distinction between authority, confidence, uncertainty, contradiction, and governance state.

---

#### 5. Final Phase 5 Mental Model

The final conceptual pipeline is:

```text
                 SOURCE
                   │
                   ▼
              Acquisition
                   │
                   ▼
             Raw Material
                   │
        ┌──────────┴──────────┐
        │                     │
        ▼                     ▼
   Raw Metadata        Core Artifact Storage
        │
        └──────────┬──────────┘
                   ▼
              Normalize
                   │
                   ▼
                Map
                   │
                   ▼
        Source-level Deduplication
                   │
                   ▼
          Entity Resolution
                   │
                   ▼
         Semantic Validation Layers
                   │
                   ▼
       Governance / Approval Rules
                   │
         ┌─────────┴─────────┐
         │                   │
       reject             approve
         │                   │
         ▼                   ▼
      Rejected        Indexing Preparation
                             │
                        ┌────┴────┐
                        │         │
                      ready     failed
                        │         │
                        ▼         ▼
                 Canonical KG   Quarantine /
                   Publish      Hold for review
                        │
                        ▼
               Index Synchronization
                        │
                        ▼
                     Indexed
```

Reprocessing may re-enter the pipeline from the appropriate previous stage rather than always beginning at raw acquisition.

---

#### 6. Ingestion Unit Model

##### 6.1 Q1 — Initial ingestion unit

Initial implementation:

```text
one source record
```

Evolution path:

```text
record
   ↓
batch
   ↓
IngestionRun
   ├── Dataset
   │    ├── Document
   │    │    └── Record
   │    └── ...
   └── semantic candidates
```

This is an evolution path, not a requirement to implement every hierarchy immediately.

---

##### 6.2 Q2 — IngestionRun

`IngestionRun` is first-class but minimal.

It represents the operational execution of one ingestion attempt.

Initially it may be identified through the Core `OperationId` rather than introducing an unnecessary new KG identity.

The design must remain open to introducing a distinct `IngestionRunId` later when a persistent/domain-level identity becomes necessary.

`IngestionRun` is not the same concept as a Core operation:

```text
Core Operation
    = universal runtime execution identity

IngestionRun
    = KG ingestion-domain execution concept
```

The initial implementation may map them one-to-one.

---

#### 7. Candidate Model

##### 7.1 Q18 — Initial candidate envelope

Initially use a generic ingestion candidate abstraction rather than introducing a permanent first-class `KnowledgeCandidate`.

Conceptually:

```text
IngestionCandidate<T>
├── source record
├── stage state
├── source metadata
├── normalized/mapped payload
├── resolution result
├── validation results
├── provenance linkage
├── evidence linkage when available
├── authority/status metadata
└── governance state
```

A future `KnowledgeCandidate` may be introduced only when the semantic candidate becomes sufficiently mature to justify a dedicated domain object.

---

##### 7.2 Q19 — Candidate payload flexibility

The candidate may contain:

```text
source/domain representation
```

at early stages and:

```text
semantic KG representation
```

at later stages.

The type should evolve with pipeline stage rather than forcing canonical semantic objects prematurely.

---

#### 8. Raw Material Boundary

##### 8.1 Q3 — Ownership

The source adapter is responsible for acquiring and describing raw material.

Phase 5 defines the raw-material contract and lifecycle expectations.

Core supplies generic artifact infrastructure where available.

---

##### 8.2 Q4 — Physical mechanism

Phase 5 does not create a new local storage engine.

Where raw material persistence is required, the implementation reuses the Core artifact-storage mechanism through the agreed abstraction.

The exact physical provider remains outside Phase 5's semantic ownership boundary.

---

#### 9. Normalization

##### 9.1 Q5 — Hybrid normalization

Normalization has two layers:

```text
Generic normalization
    +
Source-specific normalization
```

Generic normalization handles reusable invariants such as:

- representation normalization;
- encoding normalization;
- structural cleanup;
- deterministic canonical formatting;
- common metadata normalization.

Source-specific normalization handles details that depend on the source format or domain.

Source-specific logic must not leak source quirks into the canonical semantic model.

---

#### 10. Semantic Mapping

##### 10.1 Q6 — Hybrid mapping

Mapping is split between:

```text
Generic ingestion mapping
        +
Source-specific mapping
```

Generic mapping defines the boundary from ingestion structures toward Phase 1–4 semantic types.

Source-specific mapping understands the source schema and field meanings.

---

##### 10.2 Q7 — Hybrid output

Mapping may produce:

```text
deterministic semantic objects
```

when confidence and semantics are unambiguous, while uncertain mappings become candidates requiring resolution or review.

The mapper must not silently invent canonical identity.

---

#### 11. Entity Resolution Integration

##### 11.1 Q8 — Resolution order

Entity resolution occurs before the main semantic/ontology validation stage.

The pipeline still permits safe pre-resolution structural validation.

---

##### 11.2 Q9 — Ambiguity

Initial behavior:

```text
Ambiguous
    ↓
Quarantine
```

Future behavior:

```text
Ambiguous
    ↓
Provisional candidate
    ↓
Human review
    ↓
Approved / Rejected / Remain unresolved
```

Provisional state is not equivalent to final publication.

---

#### 12. Validation Architecture

##### 12.1 Q10 — Layered validation

Initial implementation uses clearly separated layers.

Every layer is designed as a pipeline unit from the beginning so the architecture can later expose the same units through an explicit `ValidationPipeline`.

Recommended logical layers are:

```text
1. Structural validation
2. Format / normalized-data validation
3. Reference validation
4. Resolution-state validation
5. Semantic validation
6. Ontology / constraint validation
7. Temporal validation where applicable
8. Evidence / provenance completeness validation where required
9. Governance validation
```

Phase 3 owns the semantic validation primitives.

Phase 5 owns orchestration and governed execution.

---

##### 12.2 Q11 — Failure handling

Validation failures are classified rather than treated uniformly.

```text
Fatal
    → candidate cannot publish
    → quarantine / rejection path

Recoverable
    → quarantine or retry/reprocess

Non-fatal
    → continue under governance policy
```

The exact fatal/recoverable boundary must be explicit in the implementation.

---

##### 12.3 Q12 — Validation result

Use a typed result model such as:

```text
Passed
Warning
Rejected
Quarantined
```

with individual findings carrying severity and diagnostic information.

A result must preserve why the state was reached.

---

##### 12.4 Q13 — Severity policy

Severity is policy-classified.

A warning does not automatically block publication.

An error/fatal finding may block publication depending on its classification.

A fatal finding always blocks the affected candidate, but does not necessarily stop other candidates in the same ingestion run.

---

#### 13. Governance and Approval

##### 13.1 Q14 — Source-class approval

Approval rules depend on source class.

Conceptually:

```text
Source Class
    ↓
Approval Policy Configuration
    ↓
Required approval mode
```

Examples of source classes may eventually include:

```text
trusted canonical source
scholarly source
community source
machine-generated source
unknown source
```

The implementation must keep the classification extensible.

---

##### 13.2 Q15 — Curation

Curation is optional, not universally mandatory.

However, a source class or governance policy may require curation before approval.

Therefore:

```text
Curation
    = optional workflow stage

Approval
    = publication gate
```

The two concepts must not be merged.

---

##### 13.3 Q16 — Approver identity

Initial approval is human-only.

Future automatic or system approval is allowed only after the automatic-approval path becomes mature enough to satisfy the governance policy.

The future approver model may reuse the generic Core/KG agent concepts rather than inventing a separate identity system.

---

##### 13.4 Q17 — Machine-generated knowledge

Machine-generated content uses the same governed lifecycle as other content.

It is distinguished through provenance and authority metadata.

There is no governance bypass for machine-generated data.

Conceptually:

```text
machine extraction
     ↓
candidate
     ↓
normalization / mapping / resolution
     ↓
validation
     ↓
human governance where required
     ↓
approval
     ↓
publication
```

---

#### 14. Rejection, Quarantine and Provisional State

These states are distinct:

```text
Rejected
    = not accepted for current publication

Quarantined
    = held for repair, review, or reprocessing

Provisional
    = controlled non-final candidate state

Approved
    = governance gate passed

Published
    = canonical Phase 5 publication boundary crossed
```

No state may be interpreted as truth merely from its name.

---

#### 15. Publication Model

##### 15.1 Q20 — Logical publication

Publication creates a logically new canonical KG state.

The operation must be controlled so the candidate cannot be simultaneously treated as unpublished and published within the same logical publication decision.

Phase 5 does not implement the durable storage version/snapshot subsystem of Phase 7.

---

##### 15.2 Q21 — Retraction / withdrawal

Phase 5 supports a basic withdrawal state without implementing the complete physical history/version subsystem.

Conceptually:

```text
Published
    ↓
Withdrawn
```

The physical and historical persistence semantics of withdrawal belong to Phase 7.

---

##### 15.3 Q22 — Corrections

Corrections preserve lineage rather than silently erasing the prior source context.

The default conceptual model is:

```text
existing knowledge
      ↓
new correction / successor candidate
      ↓
new publication decision
```

Both the source context and correction relationship are preserved.

---

##### 15.4 Q34 — Publication granularity

Publication is per candidate.

An ingestion batch is an execution unit, not an atomic semantic publication unit.

Therefore:

```text
Candidate A → publish
Candidate B → quarantine
Candidate C → reject
```

may all occur in one ingestion run unless a higher-level governance rule explicitly requires batch atomicity.

---

##### 15.5 Q35 — Meaning of published

`Published` means:

```text
accepted into the canonical KG logical state
```

within Phase 5.

Physical durability, storage transactions, snapshots, and version-history mechanisms remain Phase 7 concerns.

---

##### 15.6 Q36 — Transaction ownership

Phase 5 does not create a physical storage transaction subsystem.

It exposes publication operations and logical commit boundaries that can later be backed by the Phase 7 storage transaction model.

---

#### 16. Conflict Handling

##### 16.1 Q23 — Governance-controlled conflict

Conflicts are not automatically rejected.

The default architecture preserves both pieces of knowledge where they are individually valid representations of source claims.

Conceptually:

```text
Assertion A
    ↘
     conflict
    ↗
Assertion B
```

The conflict is represented through the Phase 4 contradiction model.

Governance policy decides whether the conflicting candidates may be published immediately, require review, or enter quarantine.

---

##### 16.2 Q49 — Contradiction is not automatic invalidity

A contradiction can be valid knowledge about disagreement.

Therefore:

```text
Conflict
    ≠
Validation failure by itself
```

Conflict may instead trigger:

```text
warning
review
quarantine
policy-specific escalation
```

The final treatment is governed by source class and policy.

---

#### 17. Deduplication

##### 17.1 Q24 — Two-layer deduplication

Deduplication occurs at two distinct semantic levels:

```text
source-level deduplication
        ↓
entity resolution
        ↓
semantic deduplication
```

###### Source-level deduplication

Removes repeated copies of the same source-level record before identity resolution.

###### Semantic deduplication

Identifies repeated or equivalent semantic content after canonical identity relationships are available.

The two algorithms must not be conflated.

---

#### 18. Reprocessing

##### 18.1 Q25 — Hybrid reprocessing

Default behavior:

```text
source delta
    ↓
process changed records only
```

Special-case behavior:

```text
full source re-ingestion
```

is available when a corrected pipeline, corrupted mapping, serious dataset issue, or other explicit policy condition requires it.

A fatal issue must not automatically cause an endless full re-ingestion loop. The reprocess trigger should identify the changed condition that makes a new run meaningful.

---

##### 18.2 Q26 — Determinism

The pipeline is designed for strict determinism.

The practical definition is:

```text
same source snapshot
+
same pipeline-stage versions
+
same configuration/policy inputs
+
same deterministic dependencies
=
same result
```

External nondeterministic dependencies must either be snapshotted, versioned, or isolated so reproducibility is not silently lost.

---

#### 19. Pipeline Component Versioning

##### 19.1 Q27 — Every stage is versioned

Each meaningful ingestion stage carries its own version metadata.

Examples:

```text
normalizer version
mapper version
resolver version
validator version
approval-policy version
publication coordinator version
```

This is **pipeline-component versioning**, not Phase 7 KG storage versioning.

The phase does not implement the complete historical version manager.

---

#### 20. Source Adapters

##### 20.1 Q28 — Adapter architecture

Use a generic ingestion contract together with source-specific implementations.

Conceptually:

```text
Generic SourceAdapter contract
        ↓
Quran adapter
Hadith adapter
Tafsir adapter
Fiqh adapter
Document adapter
Structured-data adapter
...
```

The generic contract remains domain-neutral.

---

##### 20.2 Q29 — Adapter output

Source adapters produce source-specific records.

They do not construct final canonical KG meaning by themselves.

The mapping layer converts source-specific records toward semantic candidates.

---

##### 20.3 Q30 — Structured / semi-structured / unstructured support

The abstraction is generic enough for all three forms:

```text
structured
semi-structured
unstructured
```

Initial concrete implementations may focus on structured and semi-structured data.

Unstructured ingestion can initially stop at the extraction boundary where necessary.

---

##### 20.4 Q31 — Extraction boundary

Phase 5 defines the interface boundary for extraction but does not become the final extraction/ML subsystem.

Therefore:

```text
raw document/text
    ↓
extraction boundary
    ↓
source record / extracted candidate
    ↓
Phase 5 ingestion governance
```

Advanced extraction implementation remains outside the early Phase 5 implementation.

---

#### 21. External Identity Mapping

##### 21.1 Q32 — External identifiers

Use both forms where semantically justified:

```text
Entity / semantic object
    = owns meaningful external identity information

Dedicated crosswalk / mapping record
    = preserves source-to-canonical correspondence
```

The crosswalk must not overwrite or erase the source identifier.

---

#### 22. Updating Existing Canonical Knowledge

##### 22.1 Q33 — No direct mutation bypass

Ingestion does not directly mutate published semantic objects as an uncontrolled side effect.

Instead:

```text
existing canonical state
      ↓
new update candidate
      ↓
validation / governance
      ↓
approved successor
      ↓
publication
```

This preserves provenance and correction lineage.

---

#### 23. Source Updates

##### 23.1 Q41 — Hybrid source update strategy

Default:

```text
source-level delta
    ↓
process changed material only
```

Fallback:

```text
full source re-ingestion
```

when a serious condition requires rebuilding the dataset's semantic interpretation.

The decision to full-reprocess must be explicit and reproducible.

---

#### 24. Pipeline Topology

##### 24.1 Q42 — Fixed core + stage graph hooks

Phase 5 uses a fixed conceptual backbone:

```text
raw
 ↓
normalize
 ↓
map
 ↓
deduplicate
 ↓
resolve
 ↓
validate
 ↓
approve
 ↓
publish
```

Within those boundaries, stages may expose pipeline hooks for:

```text
optional validators
optional normalizers
source-specific mapping
review stages
policy hooks
reprocessing hooks
```

This gives stability without locking the entire system into one inflexible graph.

---

#### 25. Governance State Machine

##### 25.1 Q43 — Minimal state machine

Use a small explicit lifecycle state machine rather than a general workflow engine.

Recommended conceptual states:

```text
Raw
Normalized
Mapped
Quarantined
Resolving
Resolved
Validated
NeedsReview
Approved
IndexingPending
Published
Indexed
Rejected
Withdrawn
```

Not every candidate must traverse every state.

The transition rules are explicit and tested.

---

#### 26. Governance Policy Configuration

##### 26.1 Q44 — Configuration-driven initial policy

Initial governance rules are configuration-driven.

Example dimensions:

```text
source class
required review
approval authority
allowed confidence range
conflict behavior
allowed publication state
```

Future evolution may expose a general policy abstraction.

No full workflow/policy language is introduced in Phase 5.

---

#### 27. Untrusted / Non-Authentic Sources

##### 27.1 Q45 — Preserve, label, do not endorse implicitly

Sources that are not considered authentic may still be represented when governance permits them.

They must carry explicit authority/reliability/scholarly-status metadata and appropriate epistemic interpretation.

The UI/query layer is future work, but Phase 5 must preserve the semantic distinction needed for later consumers to show:

```text
authentic / accepted
non-authentic / rejected by scholarship
disputed
uncertain
unknown
```

The exact domain-specific labels belong to the relevant authority vocabulary, not hard-coded Phase 5 strings.

---

#### 28. Evidence and Provenance Integration

##### 28.1 Q46 — Evidence is not provenance

Phase 5 does not automatically equate every source reference with Evidence.

The distinction remains:

```text
Evidence
    = support/refutation/qualification of knowledge

Provenance
    = origin and transformation lineage
```

Phase 5 may create or preserve both where appropriate.

---

##### 28.2 Q47 — Provenance requirement

Every externally sourced published assertion must have provenance linkage.

Internal/generated data may have different requirements depending on the source and governance context.

A missing mandatory provenance link is a governance/validation condition, not a reason to invent fake provenance.

---

##### 28.3 Q48 — Evidence requirement

Every externally sourced assertion should preserve evidence where an actual support relationship exists.

The implementation does not fabricate Evidence merely to satisfy a mandatory field.

Some semantic objects may not require direct Evidence in exactly the same way, but externally sourced assertions should preserve available support metadata.

---

#### 29. Core Integration

##### 29.1 Q37 — Operation identity

Initial implementation:

```text
Core OperationId
    = operational identity associated with IngestionRun
```

Later, if a durable domain-level identity becomes necessary:

```text
Core OperationId
        +
KG IngestionRunId
```

The two identities must remain semantically distinct.

---

##### 29.2 Q38 — Layered provenance linkage

The full layered relationship is:

```text
Core Operation / runtime provenance
            ↓
IngestionRun
            ↓
KG knowledge provenance
            ↓
semantic candidate / assertion
```

Core provenance remains Core-owned.

KG provenance remains knowledge-domain ownership.

Phase 5 links the layers; it does not create a duplicate Core runtime provenance system.

---

#### 30. Indexing Integration

##### 30.1 Q39 — Separate downstream operation

Publication and indexing are separate operations.

Indexing owns indexing semantics and index lifecycle.

The KG engine owns knowledge semantics and publication governance.

The intended relationship is:

```text
KG candidate
    ↓
publication coordination
    ↓
Indexing preparation / acknowledgement
    ↓
KG canonical publication
    ↓
Index activation / synchronization
```

The Indexing Engine remains responsible for index-specific build, validation, readiness, publication, and active-version lifecycle.

---

##### 30.2 Q40 — Indexing as publication precondition

Indexing is operationally important enough that Phase 5 may wait for an index-readiness acknowledgement before completing canonical publication.

This does not transfer ownership of semantic publication to Indexing.

The distinction is:

```text
Indexing readiness
    = publication precondition

Indexing publication
    = separate index lifecycle event

KG publication
    = canonical semantic lifecycle event
```

The exact failure-handshake implementation should remain an abstraction at this phase.

---

#### 31. Publication / Indexing Failure Semantics

The implementation must distinguish:

```text
semantic publication failure
```

from:

```text
index synchronization failure
```

For example:

```text
Candidate approved
        ↓
Indexing preparation succeeds
        ↓
KG publication succeeds
        ↓
Index activation fails
```

must not retroactively convert a valid KG semantic publication into a semantic validation failure.

It should produce an explicit synchronization condition that can be retried or recovered later.

---

#### 32. Authority, Confidence and Epistemic Handling

Phase 5 consumes Phase 4 metadata rather than redefining it.

The pipeline may attach or preserve:

```text
authority
reliability
scholarly status
confidence evaluation
epistemic state
contradiction linkage
temporal validity
```

But Phase 5 does not introduce a global truth score.

---

#### 33. Resolution Confidence vs Knowledge Confidence

Phase 5 must preserve the distinction established in Phase 3/4:

```text
resolution confidence
    ≠
assertion support confidence
```

A high-confidence identity resolution does not imply a high-confidence assertion.

---

#### 34. Temporal Ingestion

Temporal fields may be normalized, mapped, and validated during ingestion when present in source material.

Phase 5 must preserve the distinction between:

```text
valid time
```

and:

```text
system/activity time
```

System/activity time comes from the operational/provenance layer.

Valid time belongs to the semantic knowledge representation.

---

#### 35. Pipeline Determinism and Provenance

A successful ingestion result must be reproducible from its relevant inputs.

The provenance record should be capable of identifying:

```text
source
source version / snapshot
ingestion run
pipeline stage
stage version
configuration / policy version
resolution result
validation result
approval decision
publication decision
```

The exact physical persistence of this history remains Phase 7.

---

#### 36. Minimal Data Model

The first implementation should establish only the concepts justified by actual behavior.

Likely foundational domain structures are:

```text
IngestionRun
IngestionState
RawMaterial
SourceRecord
NormalizedRecord
MappedCandidate<T>
DeduplicationResult
ResolutionIntegrationResult
ValidationFinding
ValidationResult
GovernanceDecision
ApprovalDecision
PublicationDecision
ReprocessingRequest
PipelineStageVersion
SourceDelta
```

These names are conceptual and may be consolidated where existing structures already express the same semantic responsibility.

No object should be introduced merely to match the plan if an earlier phase already provides the correct abstraction.

---

#### 37. Proposed `src/ingestion/` Layout

The implementation should remain small and responsibility-oriented.

A likely initial layout is:

```text
src/ingestion/
├── approval.rs
├── mapping.rs
├── mod.rs
├── normalize.rs
├── pipeline.rs
├── publication.rs
├── raw.rs
└── validation.rs
```

Additional files may be introduced only when a real responsibility is too large or semantically distinct for the existing modules.

Possible later extraction points include:

```text
adapter.rs
candidate.rs
dedup.rs
governance.rs
reprocessing.rs
resolution.rs
source_delta.rs
```

The final file set should be driven by implementation complexity, not by artificial one-type-per-file rules.

---

#### 38. Module Responsibilities

##### `raw.rs`

Owns raw-material contracts and source-record capture semantics.

Must not implement a physical database.

---

##### `normalize.rs`

Owns generic and source-aware normalization stages.

Must remain deterministic.

---

##### `mapping.rs`

Maps normalized source records toward Phase 1–4 semantic objects/candidates.

Must not bypass entity resolution or governance.

---

##### `validation.rs`

Owns validation findings/results and the execution of validation layers.

Consumes Phase 3 semantic validation primitives.

---

##### `approval.rs`

Owns approval decisions and source-class governance configuration.

Initial approver is human.

---

##### `publication.rs`

Owns logical publication coordination.

Coordinates the Indexing readiness boundary without implementing Indexing internals.

---

##### `pipeline.rs`

Owns stage orchestration, transitions, re-entry points, and pipeline execution semantics.

Must not become a generic workflow engine.

---

##### `mod.rs`

Exports the public ingestion contracts and keeps internal implementation details private unless external phases require them.

---

#### 39. Error Taxonomy

Errors should grow only as actual Phase 5 behavior requires them.

Likely categories include:

```text
source acquisition error
raw material error
normalization error
mapping error
resolution integration error
validation error
governance error
approval error
publication error
index readiness error
reprocessing error
```

Not every category needs to be a distinct top-level enum variant if existing Core/shared error infrastructure already provides a cleaner representation.

The principle remains:

```text
specific enough to diagnose
not so specific that the phase creates unnecessary taxonomy
```

---

#### 40. Security Boundary

Phase 5 must reuse Core security and authorization infrastructure.

It must not implement a second authentication/authorization framework.

KG-domain authority such as scholarly status is not the same thing as runtime authorization.

Therefore:

```text
Core authorization
    = who may execute an operation

KG authority
    = how authoritative a source/claim is
```

These must remain separate.

---

#### 41. Core Context Preservation

The ingestion pipeline must preserve Core execution context where applicable:

```text
OperationContext
EngineContext
SecurityContext
Cancellation
Deadline
Core ProvenanceContext
ConfigurationSnapshot
```

Phase 5 may add genuinely KG-specific workflow metadata, but it must not duplicate Core universal runtime constructs.

---

#### 42. Observability

Phase 5 should produce enough diagnostics to answer:

```text
What source was processed?
Which ingestion run processed it?
Which stage failed?
Which version of the stage ran?
Why was the candidate quarantined?
Why was it approved or rejected?
When was it published?
Was indexing readiness obtained?
Was indexing synchronization successful?
```

Detailed metrics/tracing mechanisms remain Core-owned.

---

#### Testing Strategy

Testing is behavior-driven, not file-driven.

The implementation must test actual lifecycle behavior and negative paths.

##### Positive behavior

At minimum:

```text
raw capture
normalization
mapping
source deduplication
resolution integration
layered validation
approval
logical publication
provenance linkage
source-class policy selection
reprocessing
index readiness coordination
candidate-level publication
```

##### Negative behavior

At minimum:

```text
invalid raw input
normalization failure
mapping failure
ambiguous resolution
fatal validation finding
recoverable validation finding
missing required provenance
unauthorized approval
publication before approval
publication without required index readiness
index readiness failure
invalid state transition
reprocessing of incompatible source versions
```

##### Determinism

The same source snapshot + stage versions + policy/configuration must produce the same outcome.

##### Conflict testing

Tests must verify that contradiction does not automatically erase one side or become a validation failure merely because two valid assertions disagree.

##### Non-authentic source testing

Tests must verify that an allowed non-authentic source can be preserved with its authority/scholarly-status metadata while not being implicitly marked as authentic truth.

---

#### 44. Integration Test Boundaries

Phase 5 integration tests should verify these cross-phase contracts:

```text
Phase 1 identities
        ↓
Phase 2 assertions / relationships
        ↓
Phase 3 resolution / ontology validation
        ↓
Phase 4 evidence / provenance / authority / uncertainty / temporal metadata
        ↓
Phase 5 ingestion governance
```

The tests should verify preservation rather than reconstructing those earlier systems independently.

---

#### 45. Indexing Integration Test Boundaries

The KG tests must verify only the contract with Indexing.

They should not test Indexing internals already owned by the Indexing crate.

Verify:

```text
publication request
    ↓
index readiness request
    ↓
readiness acknowledgement
    ↓
KG publication
    ↓
index synchronization state
```

The Indexing crate separately verifies its own build/publish/active lifecycle.

---

#### 46. Reprocessing Test Matrix

Test at least:

```text
success → no-op reprocess
failure → targeted reprocess
quarantine → targeted reprocess
source delta → changed records only
pipeline version change → affected records/stages
full-reingest policy trigger → complete source run
```

Tests must prove that reprocessing does not silently lose provenance or source lineage.

---

#### Non-Goals

Phase 5 must not implement:

```text
physical KG database
full durable KG transactions
full KG snapshots/version history
large-scale storage optimization
full query language
full graph search
full reasoning engine
LLM/embedding subsystem
Python runtime integration
final gRPC API
application UI
workflow engine platform
arbitrary distributed orchestration
```

It also must not redefine:

```text
Core lifecycle
Core Control Plane
Core authentication
Core authorization
Core provenance runtime model
Indexing semantics
Phase 3 entity-resolution mechanics
Phase 4 evidence/provenance/authority primitives
```

---

#### 48. Phase 5 → Phase 6 Boundary

Phase 5 produces governed canonical knowledge.

Phase 6 consumes that knowledge for:

```text
lookup
retrieval
query planning
filtering
ranking
traversal
search
```

Phase 5 should not become a hidden query engine merely to validate its own publication decisions.

---

#### 49. Phase 5 → Phase 7 Boundary

Phase 5 defines logical publication semantics.

Phase 7 provides:

```text
physical persistence
transactions
snapshots
KG versioning
change tracking
migration
compatibility
index synchronization persistence
```

Phase 5 must keep its interfaces compatible with those future responsibilities without implementing them prematurely.

---

#### 50. Phase 5 → Phase 8 Boundary

Phase 5 may preserve:

```text
ontology validation results
resolution results
contradictions
support metadata
```

but does not execute a general reasoning engine.

A validated assertion is not the same thing as an inferred assertion.

---

#### 51. Phase 5 → Phase 9 Boundary

Machine-generated extraction may be represented as an ingestion source/candidate with provenance.

The actual Python ML stack remains Phase 9.

Phase 5 must not require Python merely to define machine provenance or governance states.

---

#### 52. Final Ownership Model

```text
                    NIZAAM CORE
                         │
       ┌─────────────────┼──────────────────┐
       │                 │                  │
   Runtime          Control Plane       Core Context
   lifecycle        security/dispatch    provenance/etc.
       │                 │                  │
       └─────────────────┼──────────────────┘
                         │
                  KG INGESTION
                         │
       ┌─────────────────┼───────────────────────┐
       │                 │                       │
   Source handling   Governance             Publication
   normalize/map     validate/review         coordinate
   dedup/resolve     approve/reject          index readiness
       │                 │                       │
       └─────────────────┼───────────────────────┘
                         │
                   CANONICAL KG
                         │
                         ▼
                    INDEXING ENGINE
                         │
                  index lifecycle
                  index versions
                  index synchronization
```

The ownership rule is:

```text
Core
    = universal infrastructure

KG ingestion
    = knowledge transformation + governance

Canonical KG
    = semantic state

Indexing
    = retrieval-oriented indexing lifecycle

Physical storage
    = later storage/provider layer
```

---

#### 53. Final Decision Matrix

| Question | Final decision | Final interpretation |
|---|---|---|
| Q1 | A → B → C | Record first, batch next, future hierarchy |
| Q2 | C | Minimal first-class `IngestionRun` |
| Q3 | B | Source-adapter side owns raw capture/persistence intent |
| Q4 | C | Reuse Core artifact-storage mechanism |
| Q5 | C | Generic + source-specific normalization |
| Q6 | C | Generic + source-specific mapping |
| Q7 | C | Direct semantic mapping where deterministic; candidate where uncertain |
| Q8 | A | Resolution before main semantic validation, with pre-resolution structural checks |
| Q9 | B + C hybrid | Quarantine initially; future provisional path with human review |
| Q10 | B → pipeline | Layered validators designed as pipeline stages from the start |
| Q11 | C | Fatal → blocked candidate; recoverable → quarantine/reprocess |
| Q12 | C | Typed validation result plus finding severity |
| Q13 | C | Policy-classified severity; fatal candidate cannot publish |
| Q14 | C | Source-class approval rules |
| Q15 | B | Curation optional, policy-controlled |
| Q16 | A → B | Human approval initially; trusted automation later |
| Q17 | C | Same lifecycle; machine origin preserved in provenance |
| Q18 | C | Generic ingestion candidate envelope initially |
| Q19 | C | Payload shape evolves by pipeline stage |
| Q20 | B | Logical new canonical state at publication; physical versioning later |
| Q21 | C | Withdrawal now, physical/history semantics later |
| Q22 | C | Preserve correction lineage and source context |
| Q23 | D | Governance-controlled conflict handling |
| Q24 | C | Source dedup before resolution; semantic dedup after resolution |
| Q25 | C | Targeted delta/retry by default; full re-ingest when justified |
| Q26 | A | Deterministic under fixed inputs/dependencies |
| Q27 | A | Every meaningful stage versioned |
| Q28 | B | Generic contract + source-specific adapters |
| Q29 | B | Adapters produce source-specific records |
| Q30 | C | Generic support for all source forms; initial implementations narrower |
| Q31 | C | Extraction boundary now, advanced extraction later |
| Q32 | C | Entity-owned external IDs + crosswalks |
| Q33 | C | Updates become governed successor candidates |
| Q34 | D | Per-candidate publication |
| Q35 | Logical canonical acceptance | Physical storage remains Phase 7 |
| Q36 | B | Publication operations, no physical transaction subsystem |
| Q37 | A → C | Core `OperationId` initially; distinct `IngestionRunId` later if needed |
| Q38 | A | Full Core + KG provenance linkage |
| Q39 | C | Separate KG publication and Indexing operations |
| Q40 | C, refined | Index-readiness is a publication precondition; indexing activation remains separate |
| Q41 | C | Delta by default; full re-ingest for explicit serious cases |
| Q42 | B + C | Fixed core pipeline + optional stage hooks |
| Q43 | B | Minimal governance state machine |
| Q44 | B → C | Config-driven initial policy; future generalized policy abstraction |
| Q45 | D, constrained | Preserve/show non-authentic content when allowed, clearly labeled, never implicit endorsement |
| Q46 | C | Evidence and provenance remain separate |
| Q47 | B | External published assertions require provenance |
| Q48 | B | External assertions preserve available evidence |
| Q49 | C | Conflict can trigger governance review; not automatic invalidity |
| Q50 | C | Published canonical state is a complete semantic subgraph, not only assertions |

---

#### Completion Criteria

Phase 5 is complete when the governed ingestion pipeline can transform source material into logically published canonical KG state while preserving validation, provenance, evidence, resolution, conflict, governance, and reprocessing boundaries.


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

Phase 5 establishes the KG as a governed semantic ingestion system rather than a collection of import scripts.

The durable architecture is:

```text
source
  ↓
raw preservation
  ↓
deterministic normalization
  ↓
semantic mapping
  ↓
source deduplication
  ↓
entity resolution
  ↓
layered validation
  ↓
governance / human approval
  ↓
index-readiness coordination
  ↓
logical canonical publication
  ↓
index synchronization
```

While preserving:

```text
Evidence ≠ Provenance
Provenance ≠ Audit
Authority ≠ Confidence
Resolution State ≠ Epistemic State
Conflict ≠ Invalidity
Valid Time ≠ System Time
Publication ≠ Indexing
Core Operation ≠ IngestionRun
Canonical Knowledge ≠ Raw Source Material
```

The implementation remains intentionally small enough to evolve.

The key future migrations are:

```text
record
  → batch
  → richer ingestion hierarchy

minimal candidate envelope
  → first-class KnowledgeCandidate if justified

global/simple governance configuration
  → richer policy abstraction

logical publication
  → durable storage/versioning in Phase 7

basic Indexing coordination
  → stronger synchronization guarantees when required

Rust-native extraction boundary
  → Python/ML integration in Phase 9
```

Phase 5 is complete only when the governed behavior exists, its negative paths are tested, earlier-phase semantics are preserved, and the implementation stays within the explicitly defined phase boundaries.

------------------------------------------------------------------------

### Phase 6: Query, Traversal, Search & Retrieval

#### Status

**Planned — architecture decisions selected provisionally**

> These decisions define the initial Phase 6 implementation direction. They are deliberately evolutionary: the initial implementation establishes stable semantic and architectural seams, while more powerful query capabilities can be added later without replacing the core model.

---

#### Goal

Phase 6 turns the KG semantic foundation established by Phases 0–5 into a usable, storage-independent retrieval system.

The phase provides:

- direct lookup;
- relationship traversal;
- semantic retrieval;
- query representation;
- query planning;
- query execution;
- typed filters;
- evidence/provenance/authority/epistemic/temporal-aware retrieval;
- reasoning-profile orchestration;
- KG-owned ranking;
- explainable results;
- query pagination and deterministic ordering;
- Indexing integration as an acceleration mechanism;
- search/forward/reverse index access abstractions.

Phase 6 does **not** implement the physical KG storage architecture or the final caching/performance architecture. Those remain Phase 7 concerns.

---

#### 2. Architectural Position

The conceptual flow is:

```text
Application
    ↓
Typed Knowledge Query API
    ↓
QueryRequest
    ↓
KG Query Planner
    ↓
KG Query Execution
    ↓
┌───────────────┬────────────────┬──────────────────┐
│ Lookup        │ Traversal      │ Retrieval        │
│               │                │                  │
│               │                │ exact            │
│               │                │ lexical          │
│               │                │ conceptual       │
│               │                │ relational       │
│               │                │ semantic         │
└───────────────┴────────────────┴──────────────────┘
    ↓
Typed filters / semantic expansion
    ↓
Optional reasoning profile
    ↓
KG-owned ranking
    ↓
Explainable QueryResult
```

The query system uses the Indexing Engine as the normal indexing/retrieval infrastructure through a KG-owned access abstraction. The KG remains the semantic authority: Indexing accelerates/retrieves indexed representations but does not own KG meaning or KG semantic identity.

Core remains responsible for universal runtime, Control Plane, context, security, cancellation, deadlines, and inter-engine communication. KG owns query semantics and KG-local planning/execution.

---

#### 3. Inherited Boundaries

##### 3.1 Core boundary

Phase 6 reuses:

```text
Core Control Plane
Universal Request/Response
OperationContext
EngineContext
capability registration/dispatch
security
cancellation
deadlines
inter-engine communication
```

KG must not create a second Control Plane, transport protocol, universal envelope, or universal operation context.

##### 3.2 Semantic model boundary

Phase 6 queries the semantic model created by earlier phases:

```text
Phase 1 → identities / semantic objects
Phase 2 → KnowledgeAssertion / relationships
Phase 3 → ontology / semantic types / resolution
Phase 4 → evidence / provenance / authority / uncertainty / contradiction / temporal knowledge
Phase 5 → governed canonical publication
```

Phase 6 does not redefine those models.

##### 3.3 Reasoning boundary

Phase 6 may select and orchestrate reasoning through a reasoning profile, but Phase 8 owns the actual reasoning engine and inference execution.

```text
Phase 6
    = decide whether/how a query requests reasoning

Phase 8
    = execute controlled deterministic reasoning
```

##### 3.4 Storage boundary

Phase 6 is storage-independent.

```text
KG query semantics
        ↓
KG storage/index abstraction
        ↓
initial in-memory/reference implementation
        ↓
future physical implementation in Phase 7
```

No physical database provider is selected in Phase 6.

---

#### 4. Decision 1 — Canonical Query Request Model

##### Decision

Use **Option B initially**: one typed `QueryRequest` with distinct query variants.

Conceptually:

```text
QueryRequest
├── Lookup(...)
├── Traversal(...)
└── SemanticRetrieval(...)
```

Shared query concerns are represented once where applicable:

```text
context
filters
pagination
ordering
ranking profile
reasoning profile
```

The model must be designed so that it can evolve toward a more general query AST without breaking the public semantic boundary.

##### Future evolution

When the request model becomes large, the implementation may move toward Option C or a better representation discovered from implementation evidence.

The migration must preserve semantic request compatibility rather than making callers understand internal AST changes.

---

#### 5. Decision 2 — General Graph Query AST Strategy

##### Decision

Build the **general graph-query AST framework from the beginning**, but implement it progressively.

This means the architecture establishes the AST/seam now, while the first usable node set is intentionally small.

Initial AST support may be equivalent to the fixed traversal operations already identified in Option A:

```text
start node/reference
relationship predicate
semantic direction
maximum depth
path constraints
node/result constraints
```

Later additions may include richer pattern composition, nested patterns, alternatives, optional branches, joins, and other operators.

##### Important interpretation

“General AST from the beginning” means:

```text
stable extensible AST architecture
```

not:

```text
feature-complete graph query language on day one
```

The initial AST therefore contains only the operations actually implemented in Phase 6.

---

#### 6. Decision 3 — Traversal Safety and Resource Budget

##### Decision

Use **Option C**: a gradual full traversal-budget model.

The initial implementation establishes the budget abstraction but expands capabilities gradually.

The conceptual budget includes:

```text
max_depth
max_nodes
max_edges
max_results
max_expansion
```

plus:

```text
cycle detection
relationship constraints
semantic direction
Core cancellation/deadline handling
```

##### Principle

Unbounded traversal is never the default.

A query must not be able to accidentally expand:

```text
A
 ↓
10 nodes
 ↓
100
 ↓
1,000
 ↓
10,000+
```

without explicit resource controls.

---

#### 7. Decision 4 — Semantic Retrieval Mechanism

##### Decision

Use **Option D**: a pluggable semantic-retrieval abstraction.

Initial implementations:

```text
symbolic KG semantic retrieval
+
search/lexical retrieval
```

Vector embeddings are intentionally **not** introduced as an initial retrieval dependency.

The abstraction must still leave room for future:

```text
vector retrieval
embedding retrieval
hybrid retrieval
```

without changing the semantic query contract.

##### Invariant

```text
vector similarity
    ≠
semantic equivalence
    ≠
canonical KG relationship
```

---

#### 8. Decision 5 — Semantic Expansion

##### Decision

Use **Option C**: controlled hybrid semantic expansion.

The query supplies the initial semantic target, while the KG decides which semantic expansions are permitted from its known semantics.

For example:

```text
sabr
 ↓
concept
 ↓
known lexical representations
 ↓
controlled related concepts
 ↓
curated mappings
 ↓
retrievable knowledge
```

Expansion may use:

```text
semantic types
ontology
lexical/concept mappings
known relationships
context
curated mappings
```

but must not silently turn arbitrary similarity into semantic truth.

---

#### 9. Decision 6 — KG and Indexing Ownership / Integration Contract

##### Decision

Use **Option C**, with one important clarification: the KG owns the **access abstraction**, while the **Indexing Engine is the normal runtime provider for indexed query access**.

The boundary is:

```text
KG semantic query
        ↓
KG Index Access Abstraction
        ↓
Indexing Engine
        ↓
Indexing-owned index structures / retrieval
        ↓
Indexing result / IndexAssignedId references
```

A local or in-memory implementation may exist only as a reference implementation, development/testing backend, or deliberate fallback path. It does not change ownership.

##### Identity ownership is completely separate from query integration

KG owns all KG semantic identities, including:

```text
EntityId
ConceptId
SourceId
ReferenceId
MentionId
KnowledgeAssertionId
...
```

Indexing owns its own internal Indexing identities, including identities for its operations, indexes, and index versions. Those identities remain entirely inside the Indexing Engine and have **no semantic role in KG**.

The only Indexing-owned identity that may cross the KG boundary is:

```text
IndexAssignedId
```

KG consumes `IndexAssignedId` only where an indexed source object must be referenced. KG does not generate, redefine, or use IndexId / IndexVersion identity as KG identity.

This gives us two independent boundaries:

```text
Query integration
    = KG may call Indexing

Identity integration
    = only IndexAssignedId may cross into KG
```

Indexing operation records and Indexing-native identity persistence remain Indexing responsibilities, including its own `.nizaam` operation storage. This does not make those identities part of the KG semantic model.

---

#### 10. Decision 7 — Index Availability and Query Fallback

##### Decision

Use **Option C**: the query planner classifies index access by dependency strength, while the normal query path uses the Indexing Engine.

Conceptually:

```text
Required
Preferred
Optional
```

For normal Phase 6 query execution:

```text
KG Query
   ↓
Indexing Engine access
```

If the requested index/access path is unavailable:

```text
Required
    → fail safely when no semantically equivalent access path exists

Preferred
    → fall back to another valid KG execution path

Optional
    → skip the accelerator
```

This does not transfer semantic ownership to Indexing. The Indexing Engine is an infrastructure dependency for efficient retrieval, not the owner of KG truth.

---

#### 11. Decision 8 — Query Filter Model

##### Decision

Use **Option C**: a typed composable filter tree.

Conceptually:

```text
Filter
├── Source
├── Authority
├── Evidence
├── Temporal
├── Epistemic
├── Publication / governance status
├── Context
├── Relationship
├── Semantic type
└── Entity/type constraints
```

Composition should support typed boolean structure such as:

```text
AND
OR
NOT
```

Example:

```text
AND
├── source = Quran
├── publication = published
└── temporal = valid_at(T)
```

The filter model must remain storage-independent.

---

#### 12. Decision 9 — Knowledge Visibility / Query Profile

##### Decision

Use **Option C**: a query/governance profile controls which classes of knowledge are visible.

A profile may select or constrain:

```text
epistemic states
authority classes
source classes
publication status
inferred vs observed knowledge
conflicted/disputed knowledge
```

Examples:

```text
PublicKnowledgeProfile
ScholarlyProfile
HistoricalProfile
ResearchProfile
```

The profile does not change the underlying KG state. It changes the permitted query view.

##### Relation to Phase 5

The profile must respect the governance decisions established during ingestion. It must not use retrieval to bypass publication/approval rules.

A non-authentic or disputed source can remain queryable when permitted by policy, but its authority/status must remain visible and must never be silently interpreted as endorsement.

---

#### 13. Decision 10 — Reasoning Integration

##### Decision

Use **Option B**: Phase 6 owns reasoning orchestration; Phase 8 owns reasoning execution.

Conceptually:

```text
Query
 ↓
ReasoningProfile
 ↓
KG Query Planner
 ↓
Relevant subgraph selection
 ↓
Phase 8 Reasoning Engine
 ↓
derived results
```

Ordinary traversal must never silently run unrestricted inference.

Phase 6 may request:

```text
no reasoning
basic reasoning profile
semantic reasoning profile
deep reasoning profile
```

but the actual deterministic rule execution belongs to Phase 8.

---

#### 14. Decision 11 — Ranking Architecture

##### Decision

Use **Option B**: KG-owned pluggable ranking profiles.

Ranking remains a KG concern, while different query contexts can use different deterministic ranking policies.

Potential initial profiles:

```text
General
Semantic
Scholarly
Historical
```

Ranking signals may eventually include:

```text
match quality
relationship relevance
path characteristics
authority/status
verification/evidence signals
temporal relevance
```

##### Invariants

```text
Ranking score
    ≠ confidence

Ranking score
    ≠ authority

Ranking score
    ≠ evidence quality
```

The ranking system may expose both an overall relevance score and interpretable ranking metadata.

---

#### 15. Decision 12 — Query Result Contract and Explainability

##### Decision

Use **Option B**: rich typed query results.

Conceptually:

```text
QueryResult
├── item
├── match_type
├── matched_path
├── ranking information
├── evidence references
├── provenance references
├── authority/status
├── inference status
└── explanation metadata
```

A result is a view over existing knowledge.

```text
QueryResult
    ≠
KnowledgeAssertion
```

Returning an item must never create a new permanent semantic relationship.

For result explanation, the system should be able to communicate:

```text
what matched
why it matched
which relationship/path was used
which evidence applies
where it came from
what authority/status applies
whether inference participated
```

---

#### 16. Decision 13 — Pagination and Deterministic Ordering

##### Decision

Use **Option C**: hybrid pagination with cursor-first semantics.

The conceptual API supports:

```text
cursor
limit
```

and may support offset semantics where appropriate for small/static result sets.

Ordering must be deterministic.

When ranking ties occur, deterministic secondary ordering must be defined so equivalent executions do not randomly reorder results.

The pagination design must be compatible with the future storage/index architecture without exposing physical pagination mechanics to applications.

---

#### 17. Decision 14 — Query Consistency / Temporal State

##### Decision

Use **Option A**: query against the current logical KG state only during Phase 6.

This does **not** remove temporal querying.

Queries may still ask:

```text
valid_at = T
valid_during = interval
```

against the current logical state.

What Phase 6 does not implement is a durable historical KG-version/snapshot query system.

That belongs to Phase 7.

Therefore:

```text
Temporal validity
    = when the knowledge applies

KG version/snapshot
    = which stored KG state contained it
```

Phase 6 supports the first and defers the second.

---

#### 18. Decision 15 — Cross-Engine Query Dependencies

##### Decision

Use **Option A for domain engines**, with an explicit Indexing exception: **normal Phase 6 queries use the Indexing Engine for indexed retrieval/access**.

The KG query path must not normally depend on another **domain/knowledge engine** such as Arabic, Quran, Hadith, or Fiqh in order to answer a query. The information needed for ordinary KG queries is expected to already exist in the canonical KG semantic model.

The normal infrastructure path is therefore:

```text
Application
    ↓
KG Query
    ↓
KG Query Planner
    ↓
KG Index Access Abstraction
    ↓
Indexing Engine
    ↓
indexed references / retrieval results
    ↓
KG semantic result
```

This is not treated as KG becoming dependent on another semantic authority. Indexing is infrastructure used by the KG query system.

###### Domain-engine boundary

```text
Arabic / Quran / Hadith / Fiqh
    → not normal Phase 6 query dependencies

Indexing
    → normal Phase 6 query infrastructure
```

Rare future cross-engine query workflows may exist, but they are not part of the normal Phase 6 query contract.

---

#### 19. Decision 16 — Phase 6 Execution Backing

##### Decision

Use **Option B**: a logical storage/access abstraction with an in-memory/reference implementation for Phase 6.

Conceptually:

```text
Query Execution
      ↓
KG Access Abstraction
      ↓
In-memory/reference implementation
```

This provides real executable behavior and meaningful tests without choosing a physical storage provider prematurely.

The abstraction must be designed so Phase 7 can replace or extend the implementation without changing the query semantics.

---

#### 20. Decision 17 — Cache and Materialization Hooks

##### Decision

Use **Option B**: define extension points only; do not implement final query caching/materialization in Phase 6.

Phase 6 may preserve seams such as:

```text
Query Execution
    ↓
optional cache/materialization hook
```

but actual cache technology, invalidation, materialized query-result lifecycle, and performance architecture belong to Phase 7.

Caching must never become an authority layer.

---

#### 21. Query Architecture Resulting from These Decisions

The resulting architecture is:

```text
                         APPLICATION
                              │
                              ▼
                    TYPED QUERY INTERFACE
                              │
                              ▼
                       QueryRequest
                              │
                              ▼
                       QUERY AST
                  (extensible from day one)
                              │
                              ▼
                       QUERY PLANNER
                              │
          ┌───────────────────┼────────────────────┐
          ▼                   ▼                    ▼
       LOOKUP             TRAVERSAL           RETRIEVAL
                              │                    │
                     traversal budgets      exact / lexical
                     path constraints       conceptual
                     direction              relational
                     relationships          semantic
                              │                    │
                              └─────────┬──────────┘
                                        ▼
                                  FILTER TREE
                                        │
                                        ▼
                               SEMANTIC EXPANSION
                                        │
                                        ▼
                                INDEX ACCESS LAYER
                                 │              │
                                 ▼              ▼
                              Local         Indexing
                              access        Engine
                                 │              │
                                 └──────┬───────┘
                                        ▼
                               OPTIONAL REASONING
                                  (Phase 8)
                                        │
                                        ▼
                               KG-OWNED RANKING
                                        │
                                        ▼
                                  QueryResult
                                        │
                                        ▼
                                  Explanation
```

---

#### 22. Lookup Model

Direct lookup is the simplest Phase 6 operation.

Examples:

```text
get Entity by EntityId
get Concept by ConceptId
get Source by SourceId
get KnowledgeAssertion by KnowledgeAssertionId
```

Lookup must avoid reasoning and unnecessary semantic expansion.

The planner should classify direct lookups separately so they remain cheap and predictable.

---

#### 23. Traversal Model

Traversal follows existing known relationships.

Example:

```text
Allah
  ↓ has-name
Ar-Rahman
```

The query layer must support:

```text
outgoing
incoming
inverse semantic direction
relationship filtering
bounded multi-hop traversal
path constraints
```

Physical inverse storage is not a Phase 6 semantic requirement.

Traversal must preserve matched paths when the result contract requests them.

---

#### 24. Traversal Is Not Inference

A path such as:

```text
A → R1 → B → R2 → C
```

does not automatically create:

```text
A → R3 → C
```

unless an explicit reasoning/composition rule is requested.

Therefore:

```text
Traversal
    = retrieve known graph paths

Inference
    = derive additional knowledge under explicit rules
```

This is a hard architectural invariant.

---

#### 25. Semantic Retrieval Model

Semantic retrieval supports questions such as:

```text
Find Quran passages about patience.
```

The initial retrieval architecture is:

```text
query term
   ↓
KG semantic interpretation
   ↓
controlled expansion
   ↓
KG semantic/search access
   ↓
filter
   ↓
rank
   ↓
explain
```

The initial system uses:

```text
symbolic semantics
+
search/lexical retrieval
```

Vector retrieval remains an extension point only.

---

#### 26. Evidence Retrieval

Evidence retrieval remains a first-class query capability.

For a knowledge assertion:

```text
Claim
 ↓
supporting evidence
 ↓
source/document/passage
```

the result can preserve:

```text
claim
support relation
source reference
passage/span
verification
provenance
authority/status
```

Evidence remains distinct from provenance.

---

#### 27. Provenance in Results

Published external knowledge should remain explainable through provenance.

Query results may therefore expose provenance references such as:

```text
source origin
activity/transformation
agent
pipeline/stage
historical lineage
```

The query layer does not create provenance; it retrieves and presents applicable provenance already attached to the knowledge.

---

#### 28. Authority, Uncertainty and Conflict Queries

Phase 6 must allow callers to query knowledge according to Phase 4 epistemic structures.

Examples:

```text
find uncertain relationships involving X
find disputed claims about X
find conflicting claims for subject/predicate pair
find knowledge from selected authority classes
```

Conflicting knowledge remains queryable when permitted.

Conflict is not treated as automatic absence.

---

#### 29. Inferred Knowledge Visibility

Queries must be able to distinguish:

```text
observed / sourced
curated
inferred
machine-generated candidate
```

Phase 6 must not silently merge inferred knowledge into observed knowledge in the result model.

A query profile may include or exclude inferred results.

---

#### 30. Query Security Boundary

The query API is not unrestricted graph access.

The effective flow remains:

```text
Core security
    ↓
KG query authorization boundary
    ↓
query validation
    ↓
planner
    ↓
execution
```

The query layer must honor the caller's permitted visibility profile and must not expose restricted unpublished/internal data merely because it exists in the KG execution backing.

Core remains the mechanism for authentication/authorization infrastructure.

---

#### 31. Query Planner Responsibilities

The KG Query Planner may decide:

```text
query shape
traversal strategy
semantic expansion
filter ordering
index requirements
fallback paths
reasoning profile orchestration
ranking profile
pagination strategy
```

It must not decide:

```text
global engine routing
global membership
transport
global scheduling
global lifecycle
```

Those remain Core responsibilities.

---

#### 32. Query Planning vs Query Execution

Keep planning and execution separate.

```text
QueryRequest
    ↓
Plan
    ↓
Execution
    ↓
Result
```

The plan may contain:

```text
retrieval operators
traversal operators
filter operators
semantic expansion operators
index access requirements
reasoning profile
ranking strategy
resource budgets
```

The exact physical access path is still hidden behind KG access abstractions.

---

#### 33. Search and Index Access

Phase 6 defines logical requirements for:

```text
search index
forward index
reverse index
```

but does not select their physical technology.

The conceptual architecture is:

```text
KG semantic model
        ↓
KG index-access abstraction
        ├── forward access
        ├── reverse access
        ├── text/search access
        └── specialized future access
```

The Indexing Engine may satisfy these requirements where appropriate.

Indexes are acceleration structures, not canonical knowledge.

---

#### 34. Result Ranking and Explanation

Ranking should expose meaningful metadata where possible.

Example:

```text
Result
  relevance = high
  match = direct semantic mapping
  path = 2 hops
  authority = curated
  evidence = verified
```

This is superior to exposing only:

```text
score = 0.873
```

The exact scoring formula remains an implementation detail and can evolve as retrieval behavior is measured.

---

#### 35. Pagination and Determinism

For large result sets:

```text
cursor
limit
```

are the preferred default.

Query execution must be deterministic for the same effective logical input, including at minimum:

```text
same current logical KG state
same query
same filters
same semantic expansion rules
same reasoning profile
same ranking profile
same configuration
```

This does not require Phase 6 to implement durable historical KG versions.

---

#### 36. Temporal Query Boundary

Phase 6 supports temporal filtering using the Phase 4 temporal model.

Examples:

```text
valid_at = T
valid_during = interval
```

The query layer does not implement durable historical KG snapshots.

Therefore a query asking:

```text
What knowledge was valid in 700 CE?
```

is a temporal-validity query against the current logical KG state, not automatically a Phase 7 historical-storage query.

---

#### 37. Cross-Phase Boundaries

##### Phase 5 → Phase 6

Phase 6 consumes published logical canonical knowledge produced by governed ingestion.

It does not bypass approval or reinterpret raw material as canonical knowledge.

##### Phase 6 → Phase 7

Phase 6 defines:

```text
query semantics
access abstractions
logical pagination
logical consistency assumptions
```

Phase 7 defines:

```text
physical storage
physical indexes
transactions
snapshots
cache architecture
materialization
persistent versioning
```

##### Phase 6 → Phase 8

Phase 6 selects and orchestrates reasoning profiles.

Phase 8 executes reasoning and inference.

##### Phase 6 → Phase 9

Python/ML is not required for initial semantic retrieval.

Future ML signals enter through the later governed Rust/Python boundary and remain distinct from canonical knowledge.

---

#### 38. Proposed Phase 6 Module Responsibilities

The current scaffold contains:

```text
src/query/
    execution.rs
    filter.rs
    plan.rs
    planner.rs
    ranking.rs
    request.rs
    result.rs

src/graph/
    edge.rs
    graph.rs
    node.rs
    path.rs
    traversal.rs

src/index/
    forward.rs
    reverse.rs
    search.rs
    index_state.rs
    mod.rs
```

The architecture recommends keeping the responsibilities roughly aligned as follows.

##### `src/query/request.rs`

Owns the typed public query request model and its variants.

##### `src/query/plan.rs`

Owns the typed query plan representation / AST execution plan boundary.

##### `src/query/planner.rs`

Transforms `QueryRequest` into a validated engine-local plan.

##### `src/query/execution.rs`

Executes the logical query plan against KG access abstractions.

##### `src/query/filter.rs`

Owns the composable typed filter tree.

##### `src/query/ranking.rs`

Owns ranking profiles and ranking metadata.

##### `src/query/result.rs`

Owns the rich explainable result model, pagination metadata, path metadata, and retrieval explanation.

##### `src/graph/`

Owns logical graph operations, path representation, bounded traversal, and semantic direction handling.

It must not become a physical database adapter.

##### `src/index/`

Owns KG-side logical index-access contracts and coordination with available index implementations.

It must not become a second Indexing Engine.

---

#### Testing Strategy

Phase 6 testing is behavior-driven.

##### Query request tests

Verify:

```text
valid lookup request
valid traversal request
valid retrieval request
invalid request combinations
query validation failures
```

##### AST / planning tests

Verify:

```text
initial traversal operators plan correctly
AST can be extended without changing existing semantics
invalid path constraints are rejected
unsupported operators fail explicitly
planner produces deterministic plans for equivalent input
```

##### Traversal tests

Verify:

```text
outgoing traversal
incoming traversal
inverse semantic traversal
multi-hop traversal
relationship filtering
path constraints
cycle handling
node/edge/result budgets
```

Negative tests must verify uncontrolled expansion is rejected or bounded.

##### Semantic retrieval tests

Verify:

```text
exact retrieval
lexical retrieval
conceptual retrieval
relational retrieval
semantic retrieval
controlled semantic expansion
vector-independent initial operation
```

##### Filter tests

Verify:

```text
source filters
authority filters
evidence filters
temporal filters
epistemic filters
publication filters
context filters
AND/OR/NOT composition
```

##### Reasoning integration tests

Verify:

```text
reasoning profile selection
no-reasoning path
delegation boundary toward Phase 8
inferred vs observed result distinction
```

Do not implement complete reasoning just to satisfy Phase 6 tests.

##### Ranking tests

Verify:

```text
profile selection
deterministic ordering
tie handling
ranking metadata
ranking remains distinct from confidence/authority
```

##### Result tests

Verify:

```text
rich result structure
matched path preservation
evidence references
provenance references
authority/status
inference status
explanation metadata
```

##### Index integration tests

Verify:

```text
local access path
Indexing-backed access path where available
required/preferred/optional dependency semantics
fallback behavior
IndexAssignedId is consumed only according to Indexing's contract
KG semantic IDs remain KG-owned
```

##### Pagination tests

Verify:

```text
cursor progression
limit behavior
deterministic ordering
no duplicate/omitted items across valid cursor pages
```

##### Storage-boundary tests

Verify:

```text
query API does not expose physical provider types
in-memory/reference access implementation can execute queries
physical storage can remain deferred to Phase 7
```

---

#### Non-Goals

Phase 6 must not implement:

```text
final database selection
final graph database architecture
physical storage schema
persistent KG versioning
full historical KG snapshots
final caching technology
final materialized-query architecture
vector database architecture
Python ML implementation
gRPC implementation
full reasoning engine
unrestricted inference
custom query language implementation
Nizaam-wide distributed query engine
second Control Plane
```

A general AST framework is allowed, but feature-complete query-language behavior is not required.

---

#### 41. Final Phase 6 Architectural Invariants

```text
Query != Storage
Query != Indexing
Query != Control Plane
Query Result != Knowledge Object
Traversal != Inference
Semantic Similarity != Semantic Truth
Ranking != Confidence
Ranking != Authority
Evidence != Provenance
Temporal Validity != KG Version
Indexing != Canonical KG
IndexAssignedId != KG Semantic Identity
```

The most important operational invariant is:

```text
KG query correctness
    must not depend on
one physical acceleration mechanism
```

The most important semantic invariant is:

```text
retrieval explains and exposes knowledge;
it does not create knowledge merely by returning it
```

---

#### Completion Criteria

Phase 6 is complete when applications can perform typed lookup, bounded traversal, explainable semantic retrieval, filtering, ranking, and pagination without exposing physical storage details or turning traversal into unrestricted inference.


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

Phase 6 establishes the KG as a **conceptual, explainable, bounded and storage-independent query system**.

The durable architecture is:

```text
Typed QueryRequest
      ↓
Extensible Query AST
      ↓
KG Query Planner
      ↓
Lookup / Traversal / Retrieval
      ↓
Typed Filter Tree
      ↓
Controlled Semantic Expansion
      ↓
KG/Index Access Abstraction
      ↓
Optional Reasoning Profile
      ↓
KG-owned Ranking
      ↓
Rich Explainable QueryResult
```

while preserving:

```text
Core owns universal runtime and global coordination.
KG owns query semantics, KG identities, and local query planning.
Indexing owns indexing infrastructure and Indexing-native identities.
IndexAssignedId is the only Indexing-owned identity consumed by KG.
Indexing is the normal query-time indexing/retrieval provider.
Phase 7 owns physical KG persistence and performance architecture.
Phase 8 owns reasoning execution.
```

Phase 6 is complete when applications can perform useful lookup, bounded traversal, evidence/provenance-aware retrieval, authority/temporal/epistemic filtering, and semantic retrieval through a typed API without depending on a physical KG storage technology.

------------------------------------------------------------------------

### Phase 7: Storage, Versioning & Performance Architecture

#### Status

**Planned — architecture decisions selected provisionally**

> These decisions define the initial Phase 7 implementation direction. They are intentionally evolutionary. The initial implementation should establish a correct semantic/storage boundary and a workable first physical architecture without freezing the KG to assumptions that have not yet been validated by real workloads.

---

#### Goal

Phase 7 turns the logical, queryable, governed Knowledge Graph established by Phases 0–6 into a **durable canonical knowledge system**.

The phase is responsible for:

```text
physical persistence
repository/storage boundary
transactions
change sets
consistency
snapshots
KG versioning
historical change tracking
retraction / supersession persistence
migration
compatibility
KG-owned physical indexes
synchronization of derived retrieval structures
recovery
performance foundations
```

The central question is:

> **How do we persist canonical KG knowledge safely and efficiently while keeping the semantic API independent from the physical storage implementation?**

Phase 7 therefore moves the system from:

```text
Phase 6
logical query / traversal / retrieval
```

to:

```text
Phase 7
logical KG
    ↓
 durable physical state
```

The semantic model remains authoritative.

Physical storage is an implementation of that model, not a replacement for it.

---

#### 2. Phase 7 Architectural Position

The long-term conceptual flow is:

```text
                    KG SEMANTIC MODEL
                           │
                           ▼
                  KG Storage Contract
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
          Repository   Transaction   Snapshot
              │            │            │
              └────────────┼────────────┘
                           ▼
                    Canonical Storage
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
         Current State   Versions     History
                           │
                           ▼
                       Change Set
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
         Forward Index  Reverse Index  Search Index
                           │
                           ▼
                  KG Retrieval Layer
```

The important ownership rule is:

```text
Canonical KG
    = semantic source of truth

KG physical indexes
    = derived acceleration structures

Cache
    = temporary acceleration

Indexing Engine
    = IndexAssignedId assignment / Indexing-specific operation storage
```

The KG does **not** delegate its search functionality to the Nizaam Indexing Engine.

The KG owns its own search, forward-index, reverse-index, and other KG-specific retrieval structures required by its semantic/query architecture.

---

#### 3. Inherited Architecture From Earlier Phases

Phase 7 must preserve all earlier semantic boundaries.

##### Phase 1

KG semantic identities remain KG-owned:

```text
EntityId
ConceptId
SourceId
ReferenceId
MentionId
KnowledgeAssertionId
...
```

They are not replaced by physical storage identities.

##### Phase 2

The canonical semantic statement remains:

```text
KnowledgeAssertion
```

with relationship semantics remaining separate from physical graph representation.

##### Phase 3

Storage must preserve:

```text
ontology
semantic types
constraints
resolution state
canonical identity
```

Physical storage must not weaken semantic validation boundaries.

##### Phase 4

Stored knowledge must preserve:

```text
Evidence
Provenance
Authority
Confidence
Epistemic State
Contradiction
Temporal Validity
```

In particular:

```text
Temporal validity
    !=
KG version
```

##### Phase 5

Only governed canonical knowledge is committed as canonical KG state.

The distinction remains:

```text
raw
normalized
mapped
validated
approved
published
```

Physical persistence must not bypass governance.

##### Phase 6

The query API remains storage-independent.

Phase 7 therefore implements the storage side of the abstraction without exposing its provider-specific details through the public query contract.

---

#### 4. Architectural Correction: Indexing Engine Ownership

Phase 7 adopts the following explicit ownership rule based on the approved architecture clarification:

```text
Indexing Engine
    └── owns Indexing-specific operations and Indexing-native identities

KG
    └── owns KG semantic identities and KG retrieval/search/index structures
```

For cross-boundary identity, the KG may consume:

```text
IndexAssignedId
```

when the semantic model needs to refer to an indexed source object.

The KG does **not** import Indexing's internal operation/version identity model into its own semantic model.

The following remain Indexing-internal:

```text
IndexId
IndexVersion
Index build/update/rebuild state
Indexing operation records
Indexing persistence records
other Indexing-native identifiers
```

They have no semantic role in the KG.

The relationship is therefore:

```text
Source Object
    ↓
Indexing Engine
    ↓
IndexAssignedId
    ↓
KG may reference this identifier where required
```

while:

```text
KG Search
KG Forward Index
KG Reverse Index
KG Semantic Retrieval
KG Query Execution
```

remain KG-owned.

This distinction is mandatory for Phase 7.

---

#### 5. Decision 1 — Physical Storage Architecture

##### Decision

Use **Option E**:

```text
Document-oriented canonical storage initially
        ↓
real KG workloads and measurements
        ↓
Hybrid architecture when justified
```

The initial physical implementation therefore follows the existing architecture's document-oriented direction.

This is **not** a commitment that the KG will remain document-only.

The migration path is:

```text
Document
    ↓
measure real workloads
    ↓
identify graph/search/transaction bottlenecks
    ↓
introduce specialized physical structures
    ↓
Hybrid where justified
```

The logical model must remain unchanged across that evolution.

##### What “Hybrid later” means

A future hybrid implementation may use different physical mechanisms for different workloads, for example:

```text
canonical document/transaction store
        +
KG relationship structures
        +
KG search index
        +
KG specialized indexes
```

The specific products/providers are deliberately not frozen in Phase 7's initial architecture.

---

#### 6. Decision 2 — Physical Aggregate / Record Layout

##### Decision

Use **Option C**: a hybrid aggregate model.

The storage layer may choose which objects naturally belong together based on the real access patterns, while heavily shared or independently versioned objects remain referentially separated.

For example, an initial logical storage layout may conceptually group:

```text
KnowledgeAssertion
    ├── core statement data
    ├── qualifiers
    ├── compact authority/status metadata
    └── references to evidence/provenance structures
```

while preserving separate storage records for objects such as:

```text
Evidence
Provenance history
Source
large textual material
shared semantic objects
```

The physical aggregation decision must never change semantic identity or ownership.

---

#### 7. Decision 3 — Repository Boundary

##### Decision

Use **Option B initially**: a semantic repository boundary, with a path toward a larger layered repository architecture later.

The initial design should expose semantic operations rather than raw database operations.

Conceptually:

```text
KG Repository
├── read semantic object
├── write semantic object
├── apply change set
├── read current state
├── read snapshot
└── publish logical state
```

The repository must not expose:

```text
MongoDB collection
SQL table
Cypher query
provider-specific cursor
provider-specific transaction type
```

Later, when the implementation grows, it may become:

```text
KG Semantic Repository
        ↓
Storage Adapter
        ↓
Physical Provider
```

The semantic repository remains the stable KG-facing boundary.

---

#### 8. Decision 4 — Transaction / Change Set Model

##### Decision

Use **Option B initially**: change-set transactions.

Later, when the requirements justify it, evolve toward **Option C**: a richer full KG transaction abstraction.

The initial model is:

```text
Logical KG operation
        ↓
ChangeSet
        ↓
validate
        ↓
commit atomically
```

A Change Set may contain changes to:

```text
entities
concepts
assertions
relationships
evidence
provenance
authority
uncertainty
temporal metadata
other canonical semantic objects
```

A committed Change Set represents the canonical semantic change that actually became part of the KG state.

It must not be confused with:

```text
WAL/journal records
transport events
audit records
temporary transaction mutations
```

Those are separate mechanisms.

##### Future evolution

A richer transaction engine may later support:

```text
nested semantic work
stronger read/write tracking
advanced conflict regions
multiple concurrency modes
larger transactional plans
```

without changing the fundamental meaning of a Change Set.

---

#### 9. Decision 5 — Consistency and Isolation

##### Decision

Use **Option B initially**: snapshot/MVCC-style reads with controlled concurrent writes.

Later, move toward **Option C** only when real workloads require configurable isolation behavior.

The initial conceptual model is:

```text
Reader
   ↓
stable committed snapshot

Writer
   ↓
construct pending Change Set
   ↓
validate conflicts
   ↓
commit
```

Readers therefore do not observe partial writes.

A failed write must not leave a partially committed semantic graph.

##### Future evolution

A later implementation may support stronger or configurable isolation profiles when:

```text
workload
contention
latency
multi-writer behavior
```

justify that complexity.

The semantic contract remains independent from the physical concurrency mechanism.

---

#### 10. Decision 6 — Snapshots and KG Versioning

##### Decision

Use **Option C**: a hybrid versioning/snapshot model.

The system should support:

```text
current canonical state
+
object-level change history
+
logical KG snapshots
```

The implementation should remain simple initially but preserve a clear route to richer historical reconstruction.

Conceptually:

```text
KG
 ├── Current State
 ├── Snapshot V1
 ├── Snapshot V2
 └── Snapshot V3
```

while individual semantic objects may also carry historical change information.

##### Version distinctions

The following must never be conflated:

```text
KG Version
    = canonical KG state/version

Source Version
    = version of source material

Schema Version
    = storage/semantic schema version

Model Version
    = model/pipeline version

Temporal Validity
    = when the represented knowledge is valid
```

A historical KG version does not automatically mean a historical temporal truth, and temporal truth does not require a separate KG snapshot for every moment.

---

#### 11. Decision 7 — Retraction, Supersession and Physical Deletion

##### Decision

Use **Option B initially**: tombstones/retraction records for semantic removal or withdrawal.

This lets the KG distinguish:

```text
retracted
withdrawn
superseded
inactive
```

from:

```text
never existed
```

A physical storage `DELETE` must not automatically mean semantic retraction.

##### Initial behavior

For example:

```text
Assertion A
    ↓
superseded by Assertion B
```

The storage layer preserves the relationship required to reconstruct the semantic history.

##### Future evolution

A later implementation may evolve toward **immutable historical objects plus a current-state projection**, especially if historical reconstruction or audit requirements become stronger.

---

#### 12. Decision 8 — Change Propagation and Derived-Index Synchronization

##### Decision

Use **Option C**: a hybrid consistency policy.

The canonical KG commit and derived index synchronization remain separate concerns.

Conceptually:

```text
KG transaction
    ↓
canonical commit
    ↓
committed Change Set
    ↓
update derived KG indexes
```

Different derived structures may have different synchronization requirements.

For example:

```text
critical KG index
    → stronger synchronization guarantee

secondary search structure
    → eventual synchronization
```

The exact policy must be explicitly declared rather than hidden inside the storage implementation.

##### Important ownership rule

These are **KG-owned derived indexes**, not Nizaam Indexing Engine search indexes.

The Indexing Engine is not responsible for KG search synchronization.

---

#### 13. Decision 9 — Stale Derived Index Behavior

##### Decision

Use **Option B initially**: stale derived-index results are allowed only with explicit staleness metadata.

Conceptually:

```text
canonical KG = V42
KG search index = V41
```

A result may carry:

```text
index_state = stale
indexed_version = V41
canonical_version = V42
```

The system must never silently present stale derived data as if it were guaranteed current.

##### Future evolution

A later implementation may add:

```text
automatic canonical fallback
strict consistency mode
query-specific freshness requirements
```

when real workloads demonstrate the need.

---

#### 14. Decision 10 — KG Physical Index Ownership

##### Decision

Use **Option C**, refined according to the approved ownership rule:

```text
KG
    = owns logical index requirements and KG physical index structures

Indexing Engine
    = does not provide KG search/retrieval
    = does not own KG forward/reverse/search indexes
    = only supplies IndexAssignedId where its contract requires one
```

The KG's physical index families may include:

```text
canonical identity indexes
entity/type indexes
relationship lookup indexes
forward indexes
reverse indexes
text/search indexes
source indexes
language indexes
temporal indexes
authority/status indexes
```

The actual structures may change over time.

The public KG semantic/query API must not depend on their physical representation.

##### IndexAssignedId boundary

The KG may store:

```text
IndexAssignedId
```

as an external/indexing reference when required.

But it is not the canonical KG identity.

```text
EntityId
    !=
IndexAssignedId
```

Likewise:

```text
KG Version
    !=
IndexVersion

KG identity
    !=
IndexId
```

No Indexing-native identity should leak into the semantic identity model beyond the explicit `IndexAssignedId` reference boundary.

---

#### 15. Decision 11 — Schema Migration and Compatibility

##### Decision

Use **Option B initially**: a versioned migration engine.

Later, extend toward **Option C**: compatibility layers for difficult/large migrations.

Conceptually:

```text
Storage Schema V1
       ↓
Migration V1 → V2
       ↓
Storage Schema V2
```

Migrations must be explicit and testable.

The migration subsystem must distinguish:

```text
schema migration
semantic migration
physical-storage migration
```

and must not silently change knowledge meaning.

##### Future compatibility layer

For large changes, we may temporarily support:

```text
old physical representation
        ↓
compatibility adapter
        ↓
new logical contract
```

while migration proceeds separately.

---

#### 16. Decision 12 — Durability and Recovery

##### Decision

Use **Option B initially**: durable journal/WAL-style recovery plus snapshots.

Future evolution may move toward **Option C** with a fuller backup/restore and disaster-recovery architecture.

Initial conceptual model:

```text
Change Set
    ↓
durable journal
    ↓
commit
    ↓
canonical state
```

with periodic or controlled snapshots:

```text
Snapshot
    +
subsequent journal records
    ↓
recovery
```

##### Recovery requirements

The initial implementation should protect against:

```text
process crash
partial physical write
incomplete commit
restart after committed change
corrupted/incomplete candidate state
```

A committed semantic change must not disappear simply because the process crashed after commit acknowledgement.

##### Future evolution

Later infrastructure may add:

```text
backup
restore
checkpoint management
point-in-time recovery
disaster recovery
```

when operational deployment requires it.

---

#### 17. Decision 13 — Performance Architecture

##### Decision

Use **Option A initially**: correctness and durability first.

Phase 7 initially implements the minimum performance structures required by the actual workload.

That includes appropriate indexes for common access paths, but does not immediately introduce a large caching/materialization subsystem.

Initial priorities are:

```text
correct persistence
correct transactions
correct snapshots
correct recovery
correct canonical reads
correct required KG indexes
```

Only after those are stable should we optimize aggressively.

##### Deferred optimization areas

The architecture remains open for later:

```text
query-result caching
materialized paths
specialized storage layouts
compaction strategies
advanced batching
memory-tiering
sharding
replication
read replicas
```

This matches the earlier decision that caching and advanced performance architecture belong principally to Phase 7's later maturation rather than being smuggled into Phase 6.

---

#### 18. Physical Storage Model

The initial storage model should preserve the semantic graph without forcing the semantic model into a database-specific structure.

Conceptually:

```text
                         KG OBJECTS
                             │
            ┌────────────────┼────────────────┐
            ▼                ▼                ▼
         Entities       Assertions        Sources
            │                │                │
            └────────────────┼────────────────┘
                             ▼
                       KG Repository
                             │
                    Transaction Boundary
                             │
                             ▼
                  Document-oriented Store
                             │
               ┌─────────────┼─────────────┐
               ▼             ▼             ▼
            Current       History       Snapshots
                             │
                             ▼
                       Derived Indexes
```

The exact physical document shapes remain an implementation detail.

A physical record must not become the public semantic contract merely because it is convenient for the initial provider.

---

#### 19. Canonical State vs Derived State

Phase 7 establishes a strict distinction between:

```text
CANONICAL STATE
```

and:

```text
DERIVED STATE
```

##### Canonical state

Contains the authoritative published KG semantic model.

Examples:

```text
Entity
Concept
KnowledgeAssertion
Relationship semantics
Evidence
Provenance
Authority
Temporal validity
Epistemic state
```

###### Derived state

Used only to accelerate access.

Examples:

```text
forward index
reverse index
search index
materialized path
cache
```

Derived state can be rebuilt.

Canonical state cannot be treated as disposable merely because a derived index exists.

---

#### 20. Forward and Reverse Index Architecture

The KG must support both forward and reverse relationship access efficiently.

Conceptually:

```text
(subject, predicate)
        ↓
forward relationship access
```

and:

```text
(predicate, object)
        ↓
reverse relationship access
```

This is an optimization of the semantic model, not a second relationship meaning.

The Phase 2 decision remains:

```text
one canonical semantic relationship
```

while Phase 7 may physically maintain:

```text
forward index
reverse index
```

without duplicating semantic truth.

---

#### 21. Search Index Architecture

The KG owns its search functionality.

The conceptual structure is:

```text
Canonical KG
      ↓
KG Search Index
      ↓
lexical / text retrieval
      ↓
KG Retrieval / Ranking
```

Search index records are not canonical knowledge.

For example:

```text
search score
    !=
confidence
```

and:

```text
search ranking
    !=
authority
```

The search index may be rebuilt from canonical KG state at any time, subject to the consistency/version rules.

---

#### 22. Snapshot Model

The initial snapshot model should support a coherent read of canonical state.

Conceptually:

```text
Current KG State
      ↓
Snapshot
      ↓
consistent read view
```

A snapshot should have a clear relation to:

```text
KG Version
```

without conflating it with:

```text
source version
schema version
temporal validity
```

Snapshots exist to establish consistent storage state, not to redefine the meaning of temporal knowledge.

---

#### 23. Change Tracking

Phase 7 introduces explicit historical change tracking.

Conceptually:

```text
KG State V1
    ↓
Change Set C1
    ↓
KG State V2
    ↓
Change Set C2
    ↓
KG State V3
```

A Change Set should capture semantic changes such as:

```text
created
updated
retracted
superseded
linked
unlinked
metadata changed
```

It should not be used as a raw physical-log dump.

The physical journal/WAL remains a separate recovery mechanism.

---

#### 24. Publication of a New KG Version

A canonical KG state transition should conceptually be:

```text
Current State
      ↓
Transaction
      ↓
Change Set
      ↓
Validation
      ↓
Durable Commit
      ↓
New KG State / Version
      ↓
Derived-index synchronization
```

If the transaction fails:

```text
Current State
      ↓
unchanged
```

A partially constructed state must never become the canonical current state.

---

#### 25. Concurrent Reads and Writes

The initial consistency architecture should support:

```text
Reader A
    → stable snapshot

Writer B
    → pending Change Set

Reader A
    → continues seeing its valid read view

Writer B
    → conflict validation
    → commit
```

The reader must not observe:

```text
half-updated assertion
missing evidence reference
partially committed relationship
inconsistent provenance attachment
```

All related semantic changes that belong to one Change Set must become visible according to the same logical commit boundary.

---

#### 26. Failure Semantics

Storage failures should remain distinguishable from semantic failures.

Conceptually:

```text
Semantic validation failure
    !=
storage failure

transaction conflict
    !=
corruption

provider unavailable
    !=
invalid knowledge
```

Examples of Phase 7-specific failure categories may include:

```text
StorageUnavailable
TransactionConflict
SnapshotUnavailable
VersionConflict
MigrationFailed
RecoveryFailed
PersistenceCorruption
IndexSynchronizationFailure
```

The exact Rust error taxonomy should grow only with implemented behavior and should map into the Core error model where the Core boundary requires it.

---

#### 27. Recovery Workflow

A basic recovery path should conceptually be:

```text
Process restart
      ↓
load latest valid snapshot
      ↓
read durable journal after snapshot
      ↓
replay committed changes
      ↓
reconstruct canonical current state
      ↓
reconcile derived KG indexes
      ↓
serve queries
```

Recovery must distinguish:

```text
committed change
```

from:

```text
aborted/incomplete transaction
```

An incomplete candidate must not be promoted as canonical simply because bytes exist on disk.

---

#### 28. Migration Safety

Schema migrations should follow a controlled sequence:

```text
detect current schema
      ↓
validate migration path
      ↓
create migration boundary
      ↓
apply migration
      ↓
verify result
      ↓
activate new schema
```

Migration must be safe against interruption.

If a migration fails, the system must not silently continue using a partially migrated state.

The migration system should preserve enough compatibility to allow explicit rollback/recovery strategies where supported by the physical architecture.

---

#### 29. Compatibility Principles

Phase 7 must distinguish:

```text
API compatibility
semantic-model compatibility
schema compatibility
storage compatibility
index compatibility
version compatibility
```

Changing one does not automatically imply changing all others.

For example:

```text
physical storage provider changes
```

should not require:

```text
application query API changes
```

provided the KG semantic and repository contracts remain compatible.

---

#### 30. KG Version vs Derived-Index Version

The following must remain separate:

```text
KG Version
    = canonical semantic state

KG Search Index Version
    = derived search state

KG Forward Index Version
    = derived relationship access state

KG Reverse Index Version
    = derived reverse relationship access state
```

A derived index may temporarily lag behind the canonical KG version.

That does not create a second canonical truth.

---

#### 31. Index Rebuilds

Because KG indexes are derived, they must be rebuildable from canonical state.

The conceptual workflow is:

```text
Canonical KG Snapshot
       ↓
build new derived index
       ↓
validate
       ↓
ready
       ↓
promote
       ↓
replace/retire old derived index
```

The currently active derived index should remain usable during a major rebuild whenever the physical implementation permits it.

A failed index rebuild must not corrupt canonical KG state.

---

#### 32. Cache Boundary

Caching remains explicitly non-canonical.

The future architecture is:

```text
Query
  ↓
Cache
  ├── hit → result
  └── miss
         ↓
      KG canonical/derived access
         ↓
       cache result
```

A cache must never become the authoritative state store.

Cache invalidation must be driven from canonical state/change semantics rather than from arbitrary timer-only assumptions once caching is implemented fully.

The initial Phase 7 implementation may keep caching minimal or deferred until storage correctness is established.

---

#### 33. Materialized Paths and Other Derived Structures

Materialized paths may later accelerate frequently requested graph traversals.

For example:

```text
Concept
   ↓
frequently traversed relationship chain
   ↓
materialized access structure
```

However:

```text
materialized path
    !=
canonical relationship
```

It must be rebuildable and invalidatable when canonical state changes.

Materialized structures therefore remain derived performance mechanisms.

---

#### 34. Physical Provider Abstraction

The storage provider boundary should be explicit:

```text
KG Semantic Repository
        ↓
Storage Provider Interface
        ↓
Document Provider
        ↓
future alternative provider(s)
```

The provider interface should expose only the physical operations necessary for the repository contract.

Provider-specific concerns must not escape into:

```text
KnowledgeAssertion
Entity
QueryRequest
QueryResult
Ontology
Evidence
Provenance
```

---

#### 35. Initial Implementation Strategy

Phase 7 should not try to implement every future optimization at once.

The recommended progression is:

```text
1. Storage abstraction
        ↓
2. Initial document-oriented provider
        ↓
3. Semantic repository
        ↓
4. Change-set transaction boundary
        ↓
5. Snapshot/read consistency
        ↓
6. Current-state persistence
        ↓
7. KG versioning
        ↓
8. Historical change tracking
        ↓
9. Durable journal + recovery
        ↓
10. KG forward/reverse indexes
        ↓
11. KG search index
        ↓
12. Index synchronization
        ↓
13. Migration/compatibility
        ↓
14. Performance measurements
        ↓
15. Evidence-driven optimization
```

The actual source-file order may differ for Rust dependency reasons, but the architectural order should remain understandable.

---

#### 36. Proposed Module Responsibilities

The Phase 7 scaffold already contains:

```text
src/storage/
src/versioning/
src/index/
```

A practical initial responsibility split is:

##### `src/storage/`

Owns:

```text
repository boundary
storage provider abstraction
document persistence
transaction integration
snapshot access
durable state/recovery integration
```

Possible internal files may include:

```text
repository.rs
transaction.rs
snapshot.rs
document_store.rs
mod.rs
```

These are implementation suggestions, not immutable file requirements.

##### `src/versioning/`

Owns:

```text
KG version identity/state
change tracking
Change Set integration
migration
compatibility
historical state metadata
```

Possible files:

```text
version.rs
change.rs
migration.rs
compatibility.rs
mod.rs
```

##### `src/index/`

Owns **KG-specific derived indexes**, not the Nizaam Indexing Engine.

Responsibilities may include:

```text
forward index
reverse index
search index abstraction
index state/version relationship to KG state
index rebuild
index synchronization
```

The module must not redefine Indexing Engine semantics.

---

#### 37. Interaction With Phase 6

Phase 6 expects:

```text
QueryRequest
    ↓
Query Planner
    ↓
KG execution
```

Phase 7 provides the persistent backing required by that execution.

The dependency therefore becomes:

```text
Phase 6 Query
        ↓
KG semantic access abstraction
        ↓
Phase 7 Repository / Index access
        ↓
canonical state + derived structures
```

The public query API does not change simply because the storage provider changes.

---

#### 38. Interaction With Phase 5

Phase 5 produces governed canonical knowledge.

Phase 7 persists the result of successful publication.

Conceptually:

```text
Phase 5
approval/publication
        ↓
logical canonical Change Set
        ↓
Phase 7 transaction
        ↓
persistent canonical state
```

A failed storage commit must not be reported as successful canonical persistence.

A successful storage commit must preserve the semantic provenance and governance state produced upstream.

---

#### 39. Interaction With Core

Core remains responsible for shared infrastructure such as:

```text
OperationContext
EngineContext
Control Plane
capability infrastructure
security
transport
universal communication
observability
shared artifact/provenance mechanisms
```

KG Phase 7 owns:

```text
KG storage semantics
KG repository
KG persistence
KG versioning
KG transactions
KG change tracking
KG physical indexes
```

Core does not become the KG database.

KG does not recreate Core's runtime infrastructure.

---

#### 40. Interaction With Indexing Engine

The Phase 7 relationship is intentionally narrow.

```text
Indexing Engine
    = generic Indexing subsystem
```

The KG does not delegate its search or graph indexes to Indexing.

The KG may consume:

```text
IndexAssignedId
```

where an indexed source object must be referenced.

Other Indexing identities and lifecycle states remain inside Indexing.

Therefore:

```text
KG
 ├── EntityId
 ├── ConceptId
 ├── KnowledgeAssertionId
 ├── KG Version
 ├── KG Search Index
 └── KG Reverse/Forward Index

Indexing
 ├── IndexId
 ├── IndexVersion
 ├── Index operation records
 └── IndexAssignedId
```

The two identity/state systems must not be merged.

---

#### 41. Observability Requirements

Phase 7 operations should expose sufficient observability for:

```text
storage latency
transaction latency
commit failures
conflicts
snapshot creation
recovery
migration
index rebuild
index synchronization
storage capacity
```

Observability mechanisms remain Core-owned.

KG supplies KG-specific semantic dimensions and operation metadata.

---

#### Testing Strategy

Phase 7 testing must verify actual storage and versioning behavior rather than merely successful compilation.

##### Storage tests

Verify:

```text
semantic objects can be persisted
objects can be loaded
updates are durable
canonical state survives restart
storage provider remains behind abstraction
```

##### Transaction tests

Verify:

```text
atomic Change Set commit
failed transaction leaves canonical state unchanged
concurrent conflicting changes are detected
partial semantic updates are not exposed
```

##### Snapshot/version tests

Verify:

```text
snapshot consistency
version creation
current-state selection
historical change tracking
KG version != temporal validity
```

##### Retraction/supersession tests

Verify:

```text
retraction survives restart
supersession relationships survive persistence
physical deletion does not silently erase semantic history
```

##### Recovery tests

Verify:

```text
restart recovery
journal replay
snapshot + journal reconstruction
incomplete transaction does not become canonical
corruption/failure is reported explicitly
```

##### Migration tests

Verify:

```text
version detection
migration execution
post-migration validation
interrupted migration behavior
compatibility boundary
```

##### KG index tests

Verify:

```text
forward lookup
reverse lookup
search indexing
index rebuild
index synchronization
stale-index metadata
failed index rebuild preserves canonical state
```

##### Performance tests

Measure rather than assume:

```text
read latency
write latency
transaction throughput
snapshot cost
recovery cost
index rebuild cost
search latency
storage size
```

The initial Phase 7 test suite should establish a baseline for later optimization rather than claiming final scalability.

---

#### 43. Negative Testing Requirements

Every major Phase 7 operation must have negative coverage.

Examples:

```text
invalid transaction state
conflicting transaction
invalid snapshot
unknown KG version
corrupt persisted record
failed journal write
failed commit
failed migration
failed recovery
failed index publication
stale derived index
provider unavailable
```

The key rule is:

```text
failure
    ↓
canonical KG state remains protected
```

A failed optimization must never corrupt canonical knowledge.

---

#### Non-Goals

Phase 7 does not implement:

```text
new semantic ontology
new relationship semantics
new evidence model
new provenance semantics
new ingestion governance
full reasoning engine
Python ML
final gRPC architecture
Nizaam-wide search engine
unrestricted distributed KG architecture
```

It also does not redefine Indexing Engine semantics.

The following remain outside the initial Phase 7 commitment:

```text
specific database provider beyond the chosen initial implementation
final graph database migration decision
sharding
full distributed replication
final cache technology
vector database
advanced ML retrieval
```

Those may be investigated or introduced only when implementation evidence justifies them.

---

#### 46. Mandatory Architectural Invariants

The following must remain true:

```text
Semantic Model
    !=
Physical Storage Model
```

```text
Canonical KG
    !=
Derived Index
```

```text
KG Version
    !=
Source Version
```

```text
KG Version
    !=
Temporal Validity
```

```text
KG Version
    !=
Storage Schema Version
```

```text
KG Search Index
    !=
Canonical KG
```

```text
IndexAssignedId
    !=
KG Semantic Identity
```

```text
IndexId
    !=
KG Version
```

```text
IndexVersion
    !=
KG Version
```

```text
Transaction
    !=
WAL
```

```text
Change Set
    !=
Audit Record
```

```text
Retraction
    !=
Physical Delete
```

```text
Cache
    !=
Canonical State
```

```text
KG Storage
    !=
Core Storage Infrastructure
```

```text
KG Search
    !=
Nizaam Indexing Engine Search
```

---

#### 47. Evolution Path

The intended evolution remains:

```text
Document storage
      ↓
real workload evidence
      ↓
Hybrid physical architecture
```

```text
Simple repository
      ↓
richer repository/storage layering
```

```text
Change-set transactions
      ↓
richer KG transaction abstraction
```

```text
Snapshot/MVCC-style consistency
      ↓
configurable isolation where justified
```

```text
Basic tombstones/retraction
      ↓
strong immutable historical state model
```

```text
Basic recovery
      ↓
full backup/restore / point-in-time recovery
```

```text
Basic KG indexes
      ↓
specialized physical structures
```

```text
Correctness-first performance
      ↓
measured caching/materialization/partitioning/etc.
```

None of these future migrations are automatic commitments. They require implementation evidence and explicit architectural review.

---

#### 48. Final Phase 7 Architecture

The complete initial Phase 7 model is:

```text
                         CANONICAL KG
                              │
                              ▼
                     Semantic Repository
                              │
                     Change Set / Transaction
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
        Snapshot          Versioning         Recovery
            │                 │                 │
            └─────────────────┼─────────────────┘
                              ▼
                    Document-Oriented Store
                              │
                              ▼
                     Canonical Current State
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
                Forward    Reverse    Search
                 Index      Index      Index
                    │         │         │
                    └─────────┼─────────┘
                              ▼
                       KG Query Layer
```

The architecture is deliberately designed so the physical layer can later evolve without changing the semantic layer.

---

#### 50. Phase 8 Boundary

Phase 8 starts only after the canonical storage and versioning foundations established here are usable.

Phase 8 will consume:

```text
canonical KG state
KG versions/snapshots where applicable
relationships
ontology
evidence
provenance
authority
uncertainty
temporal knowledge
```

and perform:

```text
controlled deterministic reasoning
bounded inference
relevant-subgraph reasoning
inference provenance
query-aware reasoning execution
```

Phase 7 does not implement the reasoning engine.

---

#### 51. Final Phase 7 Boundary Summary

```text
Phase 0
    → Core-backed engine foundation

Phase 1
    → KG semantic identities

Phase 2
    → assertions and relationships

Phase 3
    → ontology, semantics, resolution

Phase 4
    → evidence, provenance, authority,
      uncertainty, contradiction, temporal knowledge

Phase 5
    → governed ingestion and publication

Phase 6
    → query, traversal, search, retrieval

Phase 7
    → durable storage, transactions,
      snapshots, versions, recovery,
      KG physical indexes, performance

Phase 8
    → controlled reasoning
```

The phase boundary is therefore:

> **Phase 6 decides how the KG is queried; Phase 7 makes the resulting canonical knowledge physically durable without changing what that knowledge means.**

------------------------------------------------------------------------

#### Completion Criteria

Phase 7 is complete when the KG can:

1. persist canonical semantic knowledge through a storage abstraction;
2. retrieve persisted canonical knowledge through the repository boundary;
3. commit logical Change Sets atomically according to the selected consistency model;
4. prevent partial semantic state from becoming canonical;
5. create and use coherent KG snapshots;
6. maintain logical KG versions;
7. preserve historical change information according to the selected model;
8. represent retraction and supersession without confusing them with physical deletion;
9. recover canonical state after process/storage failure;
10. execute explicit storage migrations;
11. preserve semantic API compatibility while physical storage evolves;
12. maintain KG-owned forward and reverse indexes;
13. maintain the KG-owned search index abstraction;
14. synchronize derived indexes according to declared consistency policies;
15. expose explicit stale-index state when stale results are allowed;
16. keep KG semantic identities distinct from Indexing-native identities;
17. consume `IndexAssignedId` only where required without importing Indexing's internal identity model;
18. preserve Phase 1–6 contracts and semantics;
19. avoid leaking provider-specific types through the public KG API;
20. establish measurable performance baselines for future optimization.

---


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

Phase 7 establishes the KG as a **durable, versionable, recoverable, physically implemented knowledge system** without allowing physical storage mechanics to become semantic contracts.

The durable architecture is:

```text
Canonical KG
      ↓
Semantic Repository
      ↓
Change Set / Transaction
      ↓
Snapshot + Versioning
      ↓
Durable Canonical Storage
      ↓
KG-owned Forward / Reverse / Search Indexes
      ↓
KG Query / Retrieval
```

while preserving:

```text
Canonical KG            ≠ Derived Index
Temporal Validity       ≠ KG Version
KG Version              ≠ Source Version
KG Version              ≠ Schema Version
KG Semantic Identity    ≠ Indexing Identity
IndexAssignedId         ≠ KG Identity
Retraction              ≠ Physical Delete
Transaction             ≠ WAL
Change Set              ≠ Audit Record
Cache                   ≠ Canonical State
KG Search               ≠ Indexing Engine
```

The most important architectural principle is:

> **Physical storage may evolve from document-oriented to hybrid or another better architecture, but the KG semantic model and public query contract must remain stable.**

Phase 7 is complete when canonical KG knowledge is durably persisted, safely versioned, recoverable after failure, queryable through the existing semantic boundary, supported by rebuildable KG-owned indexes, and protected from physical-storage leakage.

---

### Phase 8: Controlled Reasoning

#### Status

**Planned — architecture decisions selected provisionally**

> These decisions define the initial Phase 8 implementation direction. The architecture is intentionally strong at the semantic and safety boundaries while leaving the exact execution technology open until implementation evidence justifies it.

---

#### Goal

Phase 8 introduces the Knowledge Graph's controlled reasoning and inference execution layer.

The phase consumes the semantic and durable foundations established by Phases 0–7 and adds the ability to derive additional knowledge from existing knowledge under explicitly approved semantic rules.

The goal is **not** to make the graph automatically invent relationships. The goal is:

```text
canonical knowledge
      ↓
approved semantic rules
      ↓
controlled reasoning
      ↓
derived knowledge
      ↓
explainable derivation
```

Phase 8 must preserve the distinction between:

```text
explicit knowledge
    ≠
derived knowledge
    ≠
ML prediction / speculation
```

The current Phase 8 scope explicitly calls for:

```text
reasoning engine
rules
inference
derived knowledge
reasoning profiles
query-aware inference
relevant-subgraph selection
inference provenance
distinction between observed and inferred knowledge
```

The phase is complete when approved reasoning profiles can execute while preserving provenance, distinguishing inferred knowledge, remaining bounded, and avoiding unrestricted inference during normal traversal.

---

#### 2. Architectural Position

Phase 8 sits after storage/versioning and before the future Python/ML phase.

```text
Phase 0
Core-backed engine foundation
        ↓
Phase 1
KG identities + semantic objects
        ↓
Phase 2
Knowledge Assertions + relationships
        ↓
Phase 3
Ontology + semantics + entity resolution
        ↓
Phase 4
Evidence + provenance + authority
+ uncertainty + contradiction + temporal validity
        ↓
Phase 5
Ingestion + validation + governance
        ↓
Phase 6
Query + traversal + search + retrieval
        ↓
Phase 7
Canonical storage + versioning + recovery
+ KG-owned physical indexes
        ↓
Phase 8
CONTROLLED REASONING
        ↓
Phase 9
Rust ↔ Python / ML integration
```

Phase 6 already establishes the orchestration boundary:

```text
Query
 ↓
ReasoningProfile
 ↓
KG Query Planner
 ↓
Relevant subgraph selection
 ↓
Phase 8 Reasoning Engine
 ↓
derived results
```

Phase 8 therefore owns **reasoning execution**, not the query planner, not the Control Plane, and not physical storage.

---

#### 3. Inherited Architectural Principles

Phase 8 must preserve all previous semantic decisions.

##### 3.1 Traversal is not inference

A graph path such as:

```text
A → B → C
```

does not itself imply:

```text
A → C
```

unless an explicit semantic rule authorizes that derivation.

The foundational principle is:

> **Connectivity permits traversal. Semantics permit inference. Rules permit derivation.**

Therefore:

```text
Traversal
    ≠
Inference
    ≠
Composition
    ≠
Speculation
```

##### 3.2 Explicit knowledge is not derived knowledge

An explicitly sourced assertion and a rule-derived assertion may represent the same semantic proposition, but their origins remain different.

```text
Explicit assertion
    → directly represented/sourced

Derived assertion
    → produced by an approved inference rule
```

Derived knowledge must never be presented as though a source explicitly stated it when the source did not.

##### 3.3 Deterministic inference is not ML prediction

Phase 8 executes deterministic semantic reasoning.

Future ML systems may produce:

```text
candidate
prediction
similarity
hypothesis
```

but those outputs are not automatically deterministic KG inference.

Phase 9 handles Python/ML integration and remains outside Phase 8.

##### 3.4 Open-world behavior remains authoritative

The absence of a relationship does not imply a negative fact.

```text
missing R(x,y)
    ≠
NOT R(x,y)
```

Negative inference requires explicitly established negative knowledge or another explicitly defined semantic scope that permits closed-world behavior.

##### 3.5 Evidence, provenance, authority and confidence remain distinct

Reasoning may consume assertions carrying these concepts, but inference does not collapse them.

In particular:

```text
Inference provenance
    ≠
source evidence

Reasoning ranking
    ≠
confidence

Confidence
    ≠
authority
```

---

#### 4. Final Phase 8 Decisions

Phase 8 uses the following eight merged architectural decisions.

```text
1. Rule model + rule families + conclusion capability
2. Rule ownership + scope + storage + versioning + approval + activation
3. Reasoning input scope + snapshot + context + profile-controlled expansion
4. Execution strategy + rule application model
5. Derived knowledge representation + virtual/materialized/rebuildable state
6. Derivation provenance + explanation depth
7. Conflict + negative inference + epistemic outcome behavior
8. Safety budgets + recursion + termination + incomplete results
```

All eight decisions selected **Option C**.

The selected options are deliberately compatible rather than independent choices that may create architectural conflicts.

---

#### 5. Decision 1 — Rule Model, Rule Families & Conclusion Capability

##### Decision

Use **Option C**:

```text
general declarative rule model
        +
controlled initial rule families
```

The rule representation is generic enough to evolve, but the initial implementation executes only approved rule families.

Conceptually:

```text
Rule
├── identity
├── premises
├── conditions
├── bindings
├── conclusion
├── scope
└── metadata
```

The rule describes semantic meaning rather than implementation mechanics.

Example:

```text
IF
    parent_of(x,y)
AND
    parent_of(y,z)

THEN
    grandparent_of(x,z)
```

The initial approved families may include:

```text
inverse
symmetric
transitive
type/class inheritance
property inheritance
property composition
controlled conditional rules
```

These are semantically different operations and must remain distinguishable.

For example:

```text
Transitivity:
R(x,y) + R(y,z) → R(x,z)

Composition:
R1(x,y) + R2(y,z) → R3(x,z)
```

The rule engine must not collapse all of these into one generic "connected path" operation.

##### Rule variables

Rules operate on patterns with explicit variable binding.

Example:

```text
parent_of(?x, ?y)
parent_of(?y, ?z)
```

where the shared `?y` is what connects the two premises.

A rule conclusion must be grounded by its permitted bindings. The engine must not silently invent arbitrary entities or terms as part of normal semantic inference.

##### Important boundary

The rule model is generic, but **generic does not mean unrestricted**.

The Phase 8 engine must reject or quarantine rule definitions that cannot be safely represented or executed under the active rule contract.

---

#### 6. Decision 2 — Rule Ownership, Scope, Storage, Versioning, Approval & Activation

##### Decision

Use **Option C**:

```text
hybrid governed rule registry
```

The architecture supports both foundational built-in rules and versioned declarative rule definitions.

Conceptually:

```text
Built-in foundational rules
        +
Versioned declarative rules
        ↓
Rule Registry
        ↓
validation
        ↓
approval
        ↓
activation
        ↓
Reasoning Profile
```

A rule is not executable merely because it exists as data.

The lifecycle is:

```text
defined
   ↓
validated
   ↓
approved
   ↓
versioned
   ↓
activated
   ↓
eligible for a reasoning profile
   ↓
executable
```

##### Rule scope

Rules must be explicitly scoped.

Possible scope dimensions include:

```text
global
domain
taxonomy/ontology
relationship family
dataset
application/profile
```

The implementation must never assume that a domain-specific rule is globally valid simply because it is syntactically valid.

##### Rule versioning

Rules are independently versioned semantic artifacts.

The following identities/versions remain distinct:

```text
Rule Version
    ≠
KG Version

Rule Version
    ≠
Source Version

Rule Version
    ≠
Schema Version

Rule Version
    ≠
Model Version
```

##### Rule activation

Only rules included in an approved reasoning configuration/profile are eligible for execution.

This gives the system a controlled boundary between:

```text
rule definition
```

and:

```text
active reasoning behavior
```

---

#### 7. Decision 3 — Reasoning Input Scope, Snapshot, Context & Expansion

##### Decision

Use **Option C**:

```text
relevant-subgraph reasoning by default
+
explicitly bounded expansion when required
```

Phase 8 consumes the semantic state selected by Phase 6, normally from a stable committed KG state or snapshot.

Conceptually:

```text
Query / Reasoning Request
        ↓
Reasoning Profile
        ↓
Relevant KG subgraph
        ↓
Stable committed state / snapshot
        ↓
Approved rules
        ↓
Reasoning
```

Normal reasoning must not mean:

```text
load entire KG
→ execute every rule
→ derive everything
```

A profile may explicitly request a broader bounded scope where the semantics require additional context.

For example:

```text
small query region
        OR
bounded ontology hierarchy
        OR
bounded domain region
        OR
bounded snapshot region
```

The expansion remains subject to the active reasoning budget.

##### Context

Reasoning must retain relevant semantic context when it matters to rule applicability.

Potential contextual constraints include:

```text
type
ontology
relationship family
source class
historical/temporal context
authority/governance context
semantic domain
```

The reasoning engine must not silently remove context merely to make a rule match.

##### Snapshot semantics

Phase 7 establishes that:

```text
KG Version / Snapshot
    = which canonical KG state is being reasoned over

Temporal Validity
    = when represented knowledge is valid
```

These remain separate.

---

#### 8. Decision 4 — Execution Strategy & Rule Application Model

##### Decision

Use **Option C**:

```text
hybrid query-aware execution
```

The execution model combines the benefits of goal-directed selection and controlled forward reasoning.

Conceptually:

```text
Reasoning goal
      ↓
identify relevant rules/premises
      ↓
build bounded reasoning region
      ↓
apply eligible rules
      ↓
continue while permitted
      ↓
fixed point or execution budget
```

The architecture therefore does not require the implementation to choose pure forward chaining or pure backward chaining for all workloads.

##### Forward-style execution

Forward execution is useful when a newly available premise can trigger known rules:

```text
new fact
   ↓
matching rules
   ↓
new derived fact
   ↓
other matching rules
```

##### Goal-directed execution

Goal-directed analysis is useful when the caller asks for a specific conclusion and irrelevant graph regions should not be explored.

```text
requested conclusion
   ↓
possible producing rules
   ↓
required premises
   ↓
bounded evaluation
```

##### Fixed-point behavior

For rule sets where closure is meaningful, execution should continue until:

```text
no new allowed semantic conclusions
```

or until the active safety budget is reached.

The semantic target is:

```text
same valid inputs
+
same active rules
=
same semantic closure
```

regardless of which matching rule happened to execute first.

The exact implementation technique is deliberately not frozen here.

---

#### 9. Decision 5 — Derived Knowledge Representation & Materialization

##### Decision

Use **Option C**:

```text
separate rebuildable derived layer
+
optional materialization for performance
```

The canonical KG remains the semantic source of truth.

```text
CANONICAL STATE
    = published semantic knowledge

DERIVED STATE
    = inference output / acceleration
```

A derived result may be represented using KG semantic assertion structures, but its derivation status must remain explicit.

Conceptually:

```text
Canonical assertion(s)
        +
Approved rule
        +
Derivation metadata
        ↓
Derived assertion/result
```

The derived result is therefore not silently promoted into source-backed truth.

##### Virtual inference

The system may derive conclusions at query time without physically retaining them.

```text
explicit knowledge
+
rules
        ↓
query-time derived result
```

##### Materialized inference

The system may materialize derived results where repeated access justifies it.

```text
reasoning
   ↓
rebuildable derived state
```

Materialized derived state is always subordinate to canonical state.

It may be rebuilt when:

```text
canonical state changes
rule version changes
reasoning profile changes
schema changes
```

The public semantic model must not depend on whether the derived result currently happens to be materialized.

This preserves:

```text
canonical knowledge
    ≠
derived acceleration
```

---

#### 10. Decision 6 — Derivation Provenance & Explanation Depth

##### Decision

Use **Option C**:

```text
mandatory compact derivation trace
+
optional expanded derivation graph
```

Every derived result must be explainable at least to the point of identifying:

```text
conclusion
rule
rule version
premises
reasoning profile
input KG version/snapshot
inference depth
```

Conceptually:

```text
Derived Assertion
│
├── conclusion
├── rule
├── rule version
├── premises
├── reasoning profile
├── input snapshot
└── depth
```

For deeper explanation, the engine may expose a richer derivation structure:

```text
Conclusion
    ↓
Rule R3
    ↓
Premise A
    ↓
Rule R1
    ↓
Original explicit fact
```

and similarly for each branch of a derivation graph.

##### Multiple derivations

If several reasoning paths produce the same semantic conclusion:

```text
A → R → D
```

the system should retain one semantic conclusion while preserving multiple derivation paths as metadata.

Therefore:

```text
Semantic Assertion
    ≠
Derivation Path
```

This prevents semantic duplication while retaining explainability.

##### Evidence and provenance boundary

A rule-derived result may depend on evidence-bearing premises, but the inference engine must not fabricate new source evidence.

For example:

```text
Source S supports A
        ↓
Rule R derives B
```

does not mean:

```text
Source S directly supports B
```

unless the source actually does.

---

#### 11. Decision 7 — Conflict, Negative Inference & Epistemic Outcomes

##### Decision

Use **Option C**:

```text
preserve conflicting derivations
+
represent contradiction explicitly
+
respect open-world semantics
```

If different valid rule paths yield incompatible conclusions, the reasoning engine must not resolve them through rule priority or execution order.

Example:

```text
Rule A
    ↓
A → R → C

Rule B
    ↓
A → NOT-R → C
```

The system may represent:

```text
conflict
├── derivation A
└── derivation B
```

and allow the existing epistemic/contradiction model to govern interpretation and visibility.

##### Rule priority is not semantic truth

```text
Rule execution order
    ≠
truth priority
```

A rule firing first does not make its conclusion more authoritative.

##### Negative inference

The default is open-world behavior.

```text
No R(x,y)
    ≠
NOT R(x,y)
```

Negative conclusions require an explicit negative assertion or another formally authorized semantic mechanism.

##### Epistemic outcome

The reasoning engine must preserve the existing distinction between:

```text
unknown
uncertain
ambiguous
disputed
conflicting
```

and:

```text
inferred / derived
```

Inference origin is not itself an epistemic truth value.

---

#### 12. Decision 8 — Safety Budgets, Recursion, Termination & Incomplete Results

##### Decision

Use **Option C**:

```text
profile-based reasoning budgets
+
global hard safety ceilings
+
explicit incomplete-result semantics
```

The active reasoning profile can define limits such as:

```text
max_depth
max_rule_applications
max_derived_assertions
max_expansion
max_execution_time
```

Core cancellation and deadlines remain authoritative execution controls.

No reasoning profile may disable global safety ceilings.

##### Cycle handling

Cycles over a finite set of already-existing semantic facts can be manageable:

```text
A → B
B → A
```

The engine must deduplicate derived conclusions and detect when no new allowed conclusions remain.

The dangerous case is a rule that continually generates new graph terms/entities:

```text
A → create B
B → create C
C → create D
...
```

Normal semantic inference should therefore primarily derive relationships among existing entities.

New-identity generation is tightly restricted.

##### Fixed-point termination

Where fixed-point closure is meaningful:

```text
G₀
 ↓
G₁
 ↓
G₂
 ↓
...
 ↓
Gₙ
```

execution reaches a semantic fixed point when:

```text
Gₙ₊₁ = Gₙ
```

No new permitted conclusions are produced.

Fixed-point semantics are a semantic goal, not the only safety mechanism.

##### Budget exhaustion

If a reasoning run reaches a limit:

```text
status = Incomplete
reason = budget exceeded
```

It must **not** be represented as complete closure.

This is crucial because:

```text
incomplete reasoning
    ≠
no additional knowledge exists
```

##### Safety principle

Depth limits are safety/performance controls, not semantic proof boundaries.

For example, a valid inference chain might require a depth greater than a caller's requested limit. The system should report incompleteness rather than asserting that deeper consequences do not exist.

---

#### 13. Reasoning Profiles

Phase 6 already established the concept of a reasoning profile. Phase 8 executes the active profile.

The initial profile model should remain typed and controlled.

Conceptually:

```text
ReasoningProfile
├── active rules
├── rule scopes
├── expansion policy
├── depth budget
├── execution budget
├── result limit
└── explanation policy
```

Possible conceptual profile classes include:

```text
Basic
Semantic
Deep
Research
```

These names are illustrative profile classes, not a requirement that all four become public profiles immediately.

The important boundary is:

```text
Profile
    = declares what reasoning is permitted

Reasoning Engine
    = performs that reasoning
```

A reasoning profile must never silently become an alternative query planner or governance system.

---

#### 14. Derived Assertion Model

A derived assertion should conceptually distinguish the semantic conclusion from its origin.

```text
DerivedKnowledge
├── assertion / conclusion
├── derivation
│   ├── rule id
│   ├── rule version
│   ├── premises
│   ├── depth
│   └── reasoning profile
├── input snapshot/version
└── status
```

The exact physical representation remains an implementation concern.

The semantic requirements are:

```text
derived status is visible
rule is traceable
premises are traceable
rule version is traceable
input state is traceable
```

---

#### 15. Reasoning and Canonical KG State

Phase 8 does not change the definition of canonical KG truth merely because a rule executes.

The important separation is:

```text
Canonical KG
    ↓
source of semantic knowledge

Reasoning Engine
    ↓
computes permitted consequences

Derived Layer
    ↓
rebuildable representation of those consequences
```

A later decision can determine when a derived result becomes visible to normal query consumers, but that visibility must never erase its derived origin.

Phase 8 must therefore not perform an implicit semantic promotion such as:

```text
derived
  ↓
pretend explicit
  ↓
canonical source fact
```

without a separately governed future operation.

---

#### 16. Incremental Reasoning

The architecture leaves room for incremental reasoning because Phase 7 introduces durable change sets and versioned canonical state.

Conceptually:

```text
Canonical Change Set
        ↓
identify affected relationships/rules
        ↓
recompute affected derived region
        ↓
update derived state
```

This is an important future optimization, but the initial semantic contract does not require a specific incremental engine such as RETE.

The implementation must preserve a path toward incremental reasoning without making the semantic model dependent on one execution algorithm.

---

#### 17. Rule Locality and Relevance

Rules should be scoped and targeted.

The engine should not treat every rule as applicable to every graph object.

Conceptually:

```text
Rule scope
    ↓
relevant semantic region
    ↓
candidate premise matching
```

Examples:

```text
subclass rule
    → ontology hierarchy

relationship composition rule
    → compatible relationship families

domain-specific rule
    → its declared domain/scope
```

Locality serves two purposes:

```text
semantic correctness
+
computational control
```

A rule that is valid in one domain must not accidentally contaminate unrelated semantic regions.

---

#### 18. Deduplication and Semantic Identity

Different derivation paths can produce the same semantic assertion.

Example:

```text
Path A → A ancestor_of D
Path B → A ancestor_of D
Path C → A ancestor_of D
```

The engine should not create three independent semantic facts merely because there are three proofs.

Instead:

```text
A ancestor_of D
│
├── derivation A
├── derivation B
└── derivation C
```

This preserves the Phase 1/2 identity architecture and prevents reasoning from creating semantic duplicates.

The conclusion identity remains KG-owned.

Reasoning metadata describes how the conclusion was obtained; it does not create a second KG identity system.

---

#### 19. Rule Evaluation Correctness

The reasoning engine should target semantic order independence.

Given:

```text
same canonical input state
+
same active rule set
+
same reasoning profile
```

the semantic output should not depend on incidental execution order.

Therefore:

```text
Rule order
    = implementation detail

Semantic result
    = architecture-level contract
```

This requirement becomes especially important when multiple rules produce the same conclusion or when derivations form recursive chains.

---

#### 20. Reasoning Result States

The exact public status vocabulary can evolve, but Phase 8 needs the semantic distinction between at least:

```text
Complete
Incomplete
Failed
```

and, separately, whether a returned conclusion is:

```text
Explicit
Derived
```

These axes must not be collapsed.

For example:

```text
result execution status = Complete
knowledge origin = Derived
```

is valid.

Likewise:

```text
result execution status = Incomplete
knowledge origin = Derived
```

is valid and must not be interpreted as complete closure.

---

#### 21. Phase 8 Interaction With Earlier Phases

##### Phase 2 — Relationships

Phase 8 executes semantic relationship behavior defined by Phase 2.

```text
Phase 2
    → defines relationship semantics

Phase 8
    → executes approved inferential consequences
```

##### Phase 3 — Ontology / Semantics

Phase 8 consumes:

```text
classes
subclass relations
semantic types
relationship characteristics
composition declarations
ontology semantics
```

and executes the relevant inference.

##### Phase 4 — Epistemic Foundation

Phase 8 consumes:

```text
evidence
provenance
authority
confidence
uncertainty
contradiction
temporal validity
```

without collapsing them into one reasoning score.

##### Phase 5 — Governance

Phase 8 reasons over governed canonical knowledge.

It does not bypass ingestion approval or turn quarantined/raw material into canonical reasoning input.

##### Phase 6 — Query

Phase 6 decides:

```text
whether reasoning is requested
what query needs reasoning
what relevant region is required
which reasoning profile applies
```

Phase 8 decides:

```text
how the approved reasoning is executed
```

##### Phase 7 — Storage / Versioning

Phase 8 can consume:

```text
stable canonical state
KG snapshots/versions
change sets
```

but does not redefine the storage architecture.

---

#### 22. Core Boundary

Phase 8 reuses Core for shared infrastructure.

Core continues to own:

```text
engine runtime
operation context
cancellation
deadlines
capability infrastructure
security
inter-engine communication
Control Plane
universal contracts
transport
observability
```

The reasoning engine must not implement:

```text
second Control Plane
second transport
second operation context
second universal identity framework
```

Core deadlines and cancellation are authoritative for the execution lifecycle.

---

#### 23. Indexing Boundary

Phase 8 does not transfer reasoning ownership to Indexing.

The KG remains the semantic owner.

```text
KG
    → semantic reasoning

KG-owned indexes
    → reasoning/query acceleration where required

Indexing Engine
    → IndexAssignedId boundary only in the KG context already established
```

Indexing-native identities remain outside the KG semantic model:

```text
IndexId
IndexVersion
Indexing operation identities
Indexing build/update/rebuild identities
```

They have no role in reasoning semantics.

---

#### 24. Storage Boundary

Phase 8 may produce derived state, but physical storage details remain behind the Phase 7 repository/storage boundary.

The reasoning engine must not assume:

```text
MongoDB
SQL tables
graph database
RDF store
specific search backend
```

The reasoning contract should work over semantic KG access abstractions.

---

#### Testing Strategy

Phase 8 requires testing at several levels.

##### 25.1 Rule-model unit tests

Test:

```text
rule construction
rule validation
premise validation
variable binding
conclusion validation
scope validation
rule compatibility
rule version handling
```

##### 25.2 Rule-family tests

Each approved rule family must have direct tests for:

```text
inverse
symmetric
transitive
type inheritance
property inheritance
composition
conditional rules
```

where implemented.

##### 25.3 Execution tests

Test:

```text
forward-style derivation
goal-directed relevance
hybrid execution
fixed-point closure
multiple rule interaction
rule-order independence
deduplication
```

##### 25.4 Safety tests

Test:

```text
cycle handling
recursive rules
rule explosion
max depth
max derivations
max rule applications
max expansion
execution deadlines
Core cancellation
new-identity generation restrictions
```

##### 25.5 Incomplete-result tests

Explicitly verify:

```text
budget exceeded
    → incomplete
```

and not:

```text
budget exceeded
    → complete with no more knowledge
```

##### 25.6 Provenance/explainability tests

Verify that a derived conclusion retains:

```text
rule
rule version
premises
profile
snapshot/version
depth
```

and that multiple derivation paths can coexist without duplicating semantic identity.

##### 25.7 Conflict tests

Verify that incompatible derivations are preserved and represented through the contradiction architecture rather than silently resolved by execution order.

##### 25.8 Open-world tests

Verify that:

```text
missing relation
    ≠
negative relation
```

unless explicit negative knowledge is present.

##### 25.9 Cross-phase tests

Verify:

```text
Phase 6 profile
        ↓
Phase 8 reasoning
        ↓
explainable derived result
```

and:

```text
Phase 7 snapshot
        ↓
reasoning input
        ↓
stable result provenance
```

---

#### Non-Goals

Phase 8 does not implement:

```text
Python ML
embeddings
vector models
ML-based prediction
unrestricted autonomous knowledge generation
final distributed reasoning architecture
final rule DSL programming language
final rule execution technology
final inference cache architecture
application UI
Nizaam-wide query language
second Control Plane
Indexing Engine search/retrieval
physical storage provider selection
```

These remain outside the Phase 8 implementation boundary unless a later phase explicitly moves them forward.

---

#### 27. Architectural Invariants

The following invariants are mandatory.

```text
Traversal != Inference

Inference != ML Prediction

Explicit Knowledge != Derived Knowledge

Connectivity != Semantic Entailment

Rule Execution Order != Semantic Truth

Rule Priority != Epistemic Authority

Absence != Negative Knowledge

Derived Assertion != Derivation Path

Inference Provenance != Source Evidence

Inference Provenance != Source Provenance

Rule Version != KG Version

KG Version != Temporal Validity

KG Version != Source Version

Canonical State != Derived State

Derived State != Canonical Truth by default

Indexing Identity != KG Semantic Identity

IndexAssignedId != KG Semantic Identity

Core Runtime != Reasoning Engine

Query Planner != Reasoning Engine

Reasoning Profile != Reasoning Engine

Reasoning Budget != Semantic Truth Boundary

Incomplete Reasoning != Complete Closure
```

---

#### 28. Final Phase 8 Architecture

The resulting architecture is:

```text
                         APPLICATION / QUERY
                                  │
                                  ▼
                         Phase 6 Query Planner
                                  │
                                  ▼
                         Reasoning Profile
                                  │
                    ┌─────────────┴─────────────┐
                    ▼                           ▼
             Relevant Subgraph             Bounded Expansion
                    │                           │
                    └─────────────┬─────────────┘
                                  ▼
                       Stable KG State / Snapshot
                                  │
                                  ▼
                         Approved Rule Registry
                                  │
                                  ▼
                        Rule Matching + Binding
                                  │
                                  ▼
                      Hybrid Reasoning Execution
                                  │
                 ┌────────────────┼────────────────┐
                 ▼                ▼                ▼
            Inverse /        Transitive /     Composition /
            Symmetric         Inheritance      Conditional
                 │                │                │
                 └────────────────┼────────────────┘
                                  ▼
                        Deduplication / Closure
                                  │
                     ┌────────────┴────────────┐
                     ▼                         ▼
              Derived Knowledge          Derivation Trace
                     │                         │
                     └────────────┬────────────┘
                                  ▼
                         Conflict / Epistemic
                              Handling
                                  │
                                  ▼
                         Bounded ReasoningResult
                                  │
                    ┌─────────────┴─────────────┐
                    ▼                           ▼
             Query-visible result        Rebuildable Derived State
```

The physical implementation underneath this architecture remains replaceable.

---

#### 29. Final Phase 8 Execution Principle

The central execution contract is:

```text
approved semantic rules
        ↓
controlled application to a defined KG state
        ↓
derived semantic conclusions
        ↓
explicit derivation trace
        ↓
bounded result
```

The engine must never reduce reasoning to:

```text
find connected nodes
        ↓
assume they imply something
```

Instead:

```text
semantic meaning
        ↓
approved rule
        ↓
valid premises
        ↓
controlled derivation
```

---

#### 32. Evolution Path

The initial architecture intentionally leaves room for later evolution:

```text
controlled built-in + declarative rules
        ↓
more expressive rule registry
        ↓
incremental reasoning
        ↓
more sophisticated derivation indexes
        ↓
advanced optimization
        ↓
optional specialized reasoning engines
```

Possible future implementation evolution includes:

```text
forward optimization
backward optimization
incremental closure
rule indexing
reasoning caches
materialized closure
parallel reasoning
distributed reasoning
specialized domain reasoners
```

None of these is required to change the semantic contract established here.

---

#### 33. Architectural Summary

The Phase 8 design can be summarized as:

```text
                    EXPLICIT KNOWLEDGE
                           │
                           ▼
                    APPROVED RULES
                           │
                           ▼
                 REASONING PROFILE
                           │
                           ▼
              RELEVANT / BOUNDED INPUT
                           │
                           ▼
                 CONTROLLED INFERENCE
                           │
                 ┌─────────┴─────────┐
                 ▼                   ▼
          DERIVED KNOWLEDGE   DERIVATION TRACE
                 │                   │
                 └─────────┬─────────┘
                           ▼
                 CONFLICT / EPISTEMIC
                      INTERPRETATION
                           │
                           ▼
                    QUERY / RETRIEVAL
```

The KG therefore evolves from:

```text
semantic graph
```

to:

```text
semantic graph
+
controlled inference system
+
explainable derivation
+
bounded execution
```

without collapsing the distinction between what the graph **contains**, what the rules **derive**, and what future ML systems may only **predict**.

---

##### Source Alignment

This Phase 8 plan follows the current Rust scope's Phase 8 boundary and the reasoning model already established in the architecture document, including the separation of traversal from inference, declarative rule semantics, derivation provenance, bounded reasoning, conflict preservation, open-world behavior, and the separation of deterministic inference from future ML prediction.

------------------------------------------------------------------------


---

#### Completion Criteria

Phase 8 is complete when:

```text
✓ A controlled declarative rule model exists.
✓ Initial approved rule families execute correctly.
✓ Rules have explicit scope and governed activation.
✓ Rules are versioned independently from KG/source/schema/model versions.
✓ Reasoning consumes a defined KG state/snapshot.
✓ Relevant-subgraph reasoning is the normal path.
✓ Broader reasoning is explicitly bounded.
✓ Hybrid reasoning execution is supported by the architecture.
✓ Fixed-point behavior is available where semantically appropriate.
✓ Derived knowledge remains distinguishable from explicit knowledge.
✓ Derived knowledge is rebuildable and does not become canonical truth implicitly.
✓ Mandatory compact derivation traces are preserved.
✓ Expanded derivation explanation is possible.
✓ Multiple derivation paths do not duplicate semantic identity.
✓ Conflicting derivations are preserved rather than silently prioritized.
✓ Open-world semantics are preserved.
✓ Recursive/cyclic reasoning is controlled.
✓ Resource budgets protect the system.
✓ Core cancellation and deadlines remain authoritative.
✓ Budget-exhausted reasoning is explicitly marked incomplete.
✓ Reasoning does not become a second query planner.
✓ Reasoning does not become an ML prediction system.
✓ Reasoning does not become an Indexing Engine responsibility.
✓ Physical storage details remain behind the Phase 7 storage boundary.
✓ Phase 6 reasoning-profile orchestration integrates cleanly with Phase 8 execution.
✓ Cross-phase tests protect the established semantic and ownership boundaries.
```

---


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

Phase 8 establishes Nizaam's **controlled reasoning layer**.

It turns the semantic structures created by Phases 1–7 into a system capable of deriving additional knowledge without sacrificing trust, traceability, or bounded execution.

The durable architecture is:

```text
Canonical KG
      ↓
Approved Rules
      ↓
Reasoning Profile
      ↓
Relevant / bounded knowledge region
      ↓
Hybrid controlled inference
      ↓
Derived knowledge
      +
Derivation trace
      ↓
Conflict-aware epistemic handling
      ↓
Explainable bounded result
```

The central principle is:

> **Reasoning may derive knowledge, but it must never erase where that knowledge came from, silently upgrade its authority, or pretend that an incomplete computation is a complete proof of absence.**

---

### Phase 9: Rust ↔ Python Integration

#### Status

**Deferred — detailed scope will be defined after the Python foundation is ready.**

#### Goal

Establish the approved Rust ↔ Python integration boundary without moving ML responsibilities into the Rust Knowledge Graph core.

#### Scope

Phase 9 is limited to the future inter-process integration boundary, including the agreed gRPC surface and controlled exchange of ML-ready inputs and validated ML results.

#### Planned Modules

```text
src/integration/
├── grpc.rs
└── mod.rs

python/
```

#### Testing Strategy

Integration tests will verify contract compatibility, request/result exchange, error propagation, and the rule that Python cannot directly mutate canonical KG state.

#### Non-Goals

Phase 9 does not redesign KG semantics, canonical storage, reasoning, or query behavior established by earlier phases.

#### Completion Criteria

Phase 9 is complete when the approved Rust ↔ Python boundary is implemented and verified without violating Core ownership or canonical KG ownership.


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

> **Python extends the KG through an explicit integration boundary; it does not become the owner of canonical KG state.**


---

### Phase 10: Nizaam Integration, Conformance & Hardening

#### Status

**Planned**

#### Goal

Prove that the complete KG Engine operates correctly inside the Nizaam
ecosystem.

#### Scope

``` text
Core integration
Arabic Engine integration
Indexing Engine integration
gRPC integration
capability contracts
standard result envelope
cross-engine workflows
security integration
observability
health/readiness
configuration
conformance
failure handling
regression testing
```

#### Planned Modules

``` text
src/integration/
tests/
```

#### Testing Strategy

Integration and conformance tests will verify cross-engine contracts, security and observability boundaries, failure handling, and regression behavior.

#### Non-Goals

Phase 10 does not redefine completed KG semantics or move ownership from Core, KG, or other established engine boundaries.

#### Completion Criteria

The KG:

``` text
participates correctly in Nizaam
preserves Core boundaries
preserves KG semantic ownership
communicates correctly with other engines
uses the agreed gRPC boundary
produces standard Nizaam results
preserves evidence/provenance
passes complete regression verification
```

------------------------------------------------------------------------


#### Verification Checklist
- [ ] The phase goal and approved scope are satisfied.
- [ ] The implementation and module boundaries match the approved architecture.
- [ ] Positive and negative behavior is covered by appropriate tests.
- [ ] Previously verified phases remain intact and regression-safe.
- [ ] Required verification and quality checks pass before completion is declared.
- [ ] No future-phase functionality was implemented prematurely.

#### Final Architectural Principle

> **Final integration validates the established architecture; it does not replace it.**
---

## 21. Testing Architecture

Testing is mandatory throughout implementation.

### 21.1 Source-file unit tests

Every implementation source file must contain unit tests for its
testable behavior.

Tests should cover:

``` text
constructors
validation
normalization
state transitions
successful paths
error paths
boundary conditions
identity behavior
serialization/deserialization where applicable
```

### 21.2 Module-level tests

Each module's `mod.rs` may contain tests that verify interactions
between multiple files within that module.

`mod.rs` must not replace source-file unit tests.

### 21.3 Repository integration tests

The `tests/` directory must contain integration tests for:

``` text
public KG API
cross-module behavior
capability behavior
query behavior
ingestion workflows
storage behavior
cross-engine integration
end-to-end KG workflows
```

### 21.4 Conformance tests

Conformance tests must protect architectural boundaries:

``` text
KG does not recreate Control Plane
KG does not own global routing
KG does not own Arabic morphology
KG does not become Indexing
Python cannot directly mutate canonical KG state
Traversal does not silently imply inference
Inferred knowledge remains distinguishable
Evidence/provenance remain attached to applicable knowledge
Physical storage does not leak into the semantic API
```

------------------------------------------------------------------------

## 22. Mandatory Architectural Boundaries

### Core

``` text
Core
→ universal infrastructure and execution mechanisms
```

### KG

``` text
KG
→ knowledge semantics and KG-specific execution
```

### Arabic Engine

``` text
Arabic Engine
→ Arabic linguistic analysis
```

### Indexing Engine

``` text
Indexing
→ indexing/retrieval infrastructure
```

### Python ML

``` text
Python
→ ML computation
```

### Storage

``` text
Storage
→ persistence mechanism
```

### Search Index

``` text
Search Index
→ retrieval acceleration
```

### Control Plane

``` text
Control Plane
→ global Nizaam coordination
```

No module may silently absorb another layer's responsibility merely
because doing so is convenient.

------------------------------------------------------------------------

## 23. Implementation Rules

### 23.1 Scope is authoritative

This scope defines the current implementation contract.

An implementation must not silently reinterpret architectural decisions.

### 23.2 Ask before proceeding when requirements are unclear

The implementation must stop and ask when:

``` text
requirements conflict
architecture is ambiguous
a new dependency is required
a provider must be chosen
a Core responsibility is suspected
a future phase must be pulled forward
a previous phase must be changed
a public contract must change
```

Default behavior:

``` text
STOP → EXPLAIN → ASK → WAIT → IMPLEMENT
```

Never:

``` text
GUESS → IMPLEMENT → HOPE
```

### 23.3 Do not silently change architecture

Do not silently:

``` text
move responsibilities between Core and KG
change public contracts
introduce domain semantics into Core
turn KG into a global scheduler
turn Python into canonical storage owner
change the storage abstraction
remove provenance
collapse observed and inferred knowledge
implement future-phase functionality early
```

### 23.4 Protect completed phases

Once a phase is verified, later phases must preserve:

``` text
behavior
contracts
boundaries
tests
architectural invariants
```

A previous phase may only be modified after explaining why the change is
required and obtaining approval.

### 23.5 Scaffold is not implementation

A file existing in `src/` does not mean that its functionality belongs
to the current phase.

Future-phase files may remain empty scaffolding.

### 23.6 Do not invent deferred technologies

If a technology is not authorized, do not silently choose:

``` text
database
search provider
vector provider
ML framework
Python runtime architecture
storage provider
external service
serialization format
```

when the current phase does not require that decision.

------------------------------------------------------------------------

## 24. Verification Requirements

Compilation alone is not completion.

A phase must be verified against:

``` text
Goal
Scope
Architecture
Files/modules
Boundary
Done When
Checklist
Regression tests
```

The complete verification suite should include, where applicable:

``` bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo check --workspace
cargo test --workspace --all-targets
cargo test --workspace --doc
```

Focused tests may be used during development, but they do not replace
full repository verification before declaring a phase complete.

------------------------------------------------------------------------

## 25. Phase Completion Report

When a phase is completed, the completion report should state:

``` text
what was implemented
what files were changed
what tests were added/updated
full verification results
architectural boundaries verified
previous-phase files/behavior modified or not
new decisions made
unresolved issues
```

A phase must not be declared complete if:

``` text
required behavior is missing
a previous phase regressed
architectural boundaries were violated
required verification failed
future-phase functionality was accidentally implemented
```

------------------------------------------------------------------------

## 26. Current Implementation Philosophy

The KG Engine will follow:

``` text
Strong semantic foundation
        +
Simple initial implementation
        +
Explicit abstraction boundaries
        +
Future migration paths
        +
Evidence-driven evolution
```

The initial implementation should not attempt to implement the entire
long-term KG vision immediately.

The objective is to build the foundations correctly while keeping the
system capable of evolving.

The current intended evolution paths are:

``` text
Document → Hybrid
B inverse representation → C / better representation
Global approval → policy-based governance
Simple search → Nizaam search infrastructure
KG ranking → potentially reassigned later
Conceptual API → possible future query language
Simple versioning → richer lifecycle/versioning
Rust only → future Python ML
```

These are evolution paths, not promises that every migration will
happen.

------------------------------------------------------------------------

## 27. Current High-Level Implementation Order

``` text
Phase 0
Engine workspace & integration foundation
        ↓
Phase 1
Knowledge identity & semantic objects
        ↓
Phase 2
Knowledge Assertions & relationships
        ↓
Phase 3
Ontology, semantics & entity resolution
        ↓
Phase 4
Evidence, provenance, authority, uncertainty & temporal knowledge
        ↓
Phase 5
Ingestion, validation & governance
        ↓
Phase 6
Query, traversal, search & retrieval
        ↓
Phase 7
Storage, versioning & performance
        ↓
Phase 8
Controlled reasoning
        ↓
Phase 9
Python ML / gRPC
        ↓
Phase 10
Nizaam integration, conformance & hardening
```

This ordering intentionally establishes semantic foundations before
committing to physical storage and ML implementation.

------------------------------------------------------------------------

## 28. Initial Non-Goals

The initial KG implementation does not attempt to immediately provide:

``` text
full LLM system
autonomous knowledge generation
fully automated scholarly governance
full policy-based governance
large-scale distributed graph infrastructure
final hybrid storage architecture
final Nizaam search engine
vector database architecture
advanced ML model architecture
custom Nizaam query language
unbounded inference
application-specific workflows
application UI
```

These remain future capabilities or later architectural decisions.

------------------------------------------------------------------------

## 29. Scope Status

``` text
Architecture        → established
Project scaffold    → created
Initial decisions   → recorded
Implementation      → in progress (Phase 1)

Phase 0             → Completed
Phase 1             → Completed
Phase 2             → In progress
Phase 3             → Planned
Phase 4             → Planned
Phase 5             → Planned
Phase 6             → Planned
Phase 7             → Planned
Phase 8             → Planned
Phase 9             → Deferred until Python foundation is ready
Python scope        → Separate scope.md
Rust Phase 9        → Deferred until Python foundation is ready
Phase 10            → Planned
```

The detailed implementation plan for each phase will be developed and
reviewed separately before that phase is implemented.

------------------------------------------------------------------------

## 30. Final Architectural Principle

The Knowledge Graph must not become:

``` text
a database with many connected records
```

It should become:

``` text
a governed semantic knowledge system
```

where:

``` text
knowledge objects
      ↓
meaningful relationships
      ↓
context + qualifiers
      ↓
evidence + provenance
      ↓
authority + uncertainty
      ↓
temporal validity
      ↓
controlled inference
      ↓
query / retrieval
      ↓
Nizaam applications
```

The physical implementation may evolve substantially.

The semantic contract and ownership boundaries are the foundation that
must remain protected.
