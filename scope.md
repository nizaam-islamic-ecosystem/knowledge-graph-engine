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

# 1. Mission, Scope & Responsibilities

## 1.1 Mission

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

## 1.2 Primary responsibilities

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

## 1.3 What the KG does not own

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

# 2. Architectural Position

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

# 3. Rust / Python Architecture

The engine has two planned computational parts:

``` text
KG ENGINE
├── Rust KG Core
└── Python ML Layer
```

## 3.1 Rust responsibility

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

## 3.2 Python responsibility

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

# 4. Provisional Architectural Decisions

These decisions establish the initial implementation direction. They may
be revisited later when implementation evidence justifies a change.

## Decision 1 --- Physical storage architecture

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

## Decision 2 --- Inverse relationship representation

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

## Decision 3 --- Canonical knowledge unit

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

## Decision 4 --- Initial governance

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

## Decision 5 --- Rust ↔ Python communication

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

## Decision 6 --- Python implementation timing

The Python layer is intentionally deferred. Initial implementation
focuses on the Rust KG. The Python ML subsystem may be implemented much
later.

## Decision 7 --- Search, evidence, ranking, and query interface

### Search

A search index is required.

``` text
small/simple search index
        ↓
future Nizaam search infrastructure
```

### Evidence / provenance retrieval

Evidence and provenance are required from the beginning, but the first
implementation should remain simple and extensible.

### Ranking

Initial direction:

``` text
KG-owned ranking
```

Ownership may be moved later if actual workload analysis shows that
another engine or subsystem is more appropriate.

### Query interface

Initial direction:

``` text
Conceptual / typed KG API
```

No query language is required initially.

A future Nizaam-specific query language remains possible.

## Decision 8 --- Knowledge mutability / versioning

Initial direction:

``` text
simple initial model
```

The implementation must preserve a path toward richer versioning and
lifecycle semantics later.

------------------------------------------------------------------------

# 5. Semantic Knowledge Model

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

## 5.1 Entity

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

## 5.2 Concept

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

## 5.3 Lexical model

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

## 5.4 Knowledge Assertion

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

# 6. Relationship Architecture

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

# 7. Graph Traversal

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

# 8. Ontology and Semantics

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

# 9. Evidence, Provenance, Authority and Trust

The KG must preserve why knowledge exists and where it came from.

## Evidence

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

## Provenance

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

## Authority

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

## Uncertainty

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

# 10. Temporal Knowledge

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

# 11. Controlled Reasoning

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

# 12. Knowledge Ingestion and Curation

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

# 13. Entity Resolution

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

# 14. Query and Retrieval Architecture

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

# 15. Search and Indexing

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

# 16. Storage Architecture

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

# 17. Versioning and Evolution

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

# 18. Runtime and Standalone Execution

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

# 19. Module Structure

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

# 20. Implementation Phases

The architecture contains many conceptual sections. They are
intentionally grouped into a smaller number of implementation phases so
implementation remains manageable.

The exact internal file boundaries may change during phase
implementation.

------------------------------------------------------------------------

## Phase 0: Engine Workspace & Integration Foundation

### Status

**Planned**

### Goal

Establish the KG Engine as a valid Nizaam infrastructure engine using
Core mechanisms.

### Scope

``` text
package/library integration
KG engine identity
engine runtime integration
lifecycle startup/shutdown boundary
capability registration boundary
universal request/response integration
OperationContext / EngineContext propagation
runtime admission boundary
Control Plane integration boundary
health/readiness integration
error integration
KG-local planner extension point
standalone runtime entry point
test harness foundation
```

### Must not implement

``` text
Knowledge model
Entity resolution
Knowledge Assertions
Relationship semantics
Ontology
Inference
Physical storage
Search index
Python ML
Domain ingestion
```

### Planned modules

``` text
src/lib.rs
src/main.rs
src/engine/
src/contract/
src/integration/core.rs
src/integration/grpc.rs
```

### Done when

``` text
construct
 ↓
initialize runtime
 ↓
register
 ↓
reach READY
 ↓
serve
 ↓
receive a universal request
 ↓
dispatch a KG capability boundary
 ↓
produce a universal result
 ↓
shutdown cleanly
```

without duplicating Core infrastructure.

------------------------------------------------------------------------

## Phase 1: Knowledge Identity & Core Semantic Objects

### Status

**Planned**

### Goal

Establish the foundational identity and semantic object vocabulary of
the KG.

### Scope

``` text
canonical object identity
entity identity
concept identity
source identity
reference identity
entity names / aliases / mentions
lexical form foundations
semantic object categories
KnowledgeAssertion identity boundary
```

### Planned modules

``` text
src/identity/
src/entity/
src/concept/
src/lexical/
src/source/
src/assertion/
```

### Done when

The engine has stable, distinct representations for foundational
semantic objects and does not collapse all knowledge into generic nodes.

------------------------------------------------------------------------

## Phase 2: Knowledge Assertion & Relationship Model

### Status

**Planned**

### Goal

Implement the central semantic assertion model and relationship
vocabulary.

### Scope

``` text
KnowledgeAssertion
subject
predicate
object
context
qualifiers
status

relationship predicates
relationship direction
inverse relationships
relationship characteristics
relationship families
```

### Planned modules

``` text
src/assertion/
src/relationship/
src/graph/
```

### Done when

The KG can represent meaningful assertions and relationships while
preserving the distinction between:

``` text
assertion
relationship semantics
graph structure
```

and can traverse canonical and inverse relationships without requiring
physical duplication of every inverse edge.

------------------------------------------------------------------------

## Phase 3: Ontology, Semantics & Entity Resolution

### Status

**Planned**

### Goal

Prevent the graph from becoming an arbitrary collection of connected
objects.

### Scope

``` text
ontology
classes
properties
taxonomy
constraints
semantic types
meaning
context
interpretation
entity resolution
matching
disambiguation
canonical identity resolution
```

### Planned modules

``` text
src/ontology/
src/semantics/
src/resolution/
```

### Done when

The KG can distinguish semantic types, apply ontology constraints, and
resolve source-level references toward canonical entities without
erasing provenance differences.

------------------------------------------------------------------------

## Phase 4: Evidence, Provenance, Authority, Uncertainty & Temporal Knowledge

### Status

**Planned**

### Goal

Make knowledge quality, origin, trust, conflict, and time first-class
parts of the KG.

### Scope

``` text
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
```

### Planned modules

``` text
src/evidence/
src/provenance/
src/authority/
src/uncertainty/
src/temporal/
```

### Done when

The KG can answer not only:

``` text
What is the assertion?
```

but also:

``` text
Why does it exist?
Where did it come from?
How was it produced?
What authority applies?
How certain is it?
Is it disputed?
When is it valid?
```

------------------------------------------------------------------------

## Phase 5: Knowledge Ingestion, Validation & Governance

### Status

**Planned**

### Goal

Create the governed pipeline through which knowledge enters the
canonical KG.

### Scope

``` text
raw input
normalization
mapping
validation
entity resolution integration
approval
publication
manual curation
source adapters
conflict handling
reprocessing
```

Initial approval model:

``` text
single global approval rule
```

Future evolution:

``` text
policy-based governance
```

### Planned modules

``` text
src/ingestion/
```

### Done when

The KG can move knowledge through:

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

and machine-generated candidates cannot bypass governance.

------------------------------------------------------------------------

## Phase 6: Query, Traversal, Search & Retrieval

### Status

**Planned**

### Goal

Provide conceptual knowledge retrieval without exposing physical
database mechanics.

### Scope

``` text
lookup
traversal
semantic retrieval
query request
query planning
query execution
filters
evidence filters
authority filters
temporal filters
reasoning profile selection
ranking
search index integration
forward index
reverse index
```

Initial search direction:

``` text
simple search index
```

Initial ranking direction:

``` text
KG-owned ranking
```

Initial API direction:

``` text
conceptual / typed API
```

No query language is required in the initial implementation.

### Planned modules

``` text
src/query/
src/index/
src/graph/
```

### Done when

Applications can express conceptual KG operations such as:

``` text
find knowledge by entity
find knowledge by concept
find related assertions
traverse relationships
retrieve evidence
retrieve provenance
perform bounded semantic retrieval
```

without depending on physical storage technology.

------------------------------------------------------------------------

## Phase 7: Storage, Versioning & Performance Architecture

### Status

**Planned**

### Goal

Implement canonical KG persistence behind an abstraction and establish
the first physical storage architecture.

### Initial direction

``` text
Document-oriented implementation
```

### Evolution path

``` text
Document
   ↓
larger graph / workload
   ↓
Hybrid
```

### Scope

``` text
storage abstraction
document persistence
transactions
snapshots
KG versioning
change tracking
migration
compatibility
search index synchronization
reverse indexes
performance boundaries
storage consistency
```

### Planned modules

``` text
src/storage/
src/versioning/
src/index/
```

### Must remain open

``` text
specific database provider
graph database
RDF store
relational provider
hybrid implementation details
physical partitioning
sharding
cache technology
vector provider
```

### Done when

The semantic KG can persist and retrieve canonical knowledge without
exposing physical storage details through the public KG contract.

------------------------------------------------------------------------

## Phase 8: Controlled Reasoning

### Status

**Planned**

### Goal

Introduce deterministic, bounded, provenance-aware reasoning over the
KG.

### Scope

``` text
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

### Planned modules

``` text
src/reasoning/
```

### Done when

The KG can execute approved reasoning profiles while:

``` text
preserving provenance
distinguishing inferred knowledge
remaining bounded
avoiding unrestricted inference during normal traversal
```

------------------------------------------------------------------------

## Phase 10: Nizaam Integration, Conformance & Hardening

### Status

**Planned**

### Goal

Prove that the complete KG Engine operates correctly inside the Nizaam
ecosystem.

### Scope

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

### Planned modules

``` text
src/integration/
tests/
```

### Done when

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

# 21. Testing Architecture

Testing is mandatory throughout implementation.

## 21.1 Source-file unit tests

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

## 21.2 Module-level tests

Each module's `mod.rs` may contain tests that verify interactions
between multiple files within that module.

`mod.rs` must not replace source-file unit tests.

## 21.3 Repository integration tests

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

## 21.4 Conformance tests

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

# 22. Mandatory Architectural Boundaries

## Core

``` text
Core
→ universal infrastructure and execution mechanisms
```

## KG

``` text
KG
→ knowledge semantics and KG-specific execution
```

## Arabic Engine

``` text
Arabic Engine
→ Arabic linguistic analysis
```

## Indexing Engine

``` text
Indexing
→ indexing/retrieval infrastructure
```

## Python ML

``` text
Python
→ ML computation
```

## Storage

``` text
Storage
→ persistence mechanism
```

## Search Index

``` text
Search Index
→ retrieval acceleration
```

## Control Plane

``` text
Control Plane
→ global Nizaam coordination
```

No module may silently absorb another layer's responsibility merely
because doing so is convenient.

------------------------------------------------------------------------

# 23. Implementation Rules

## 23.1 Scope is authoritative

This scope defines the current implementation contract.

An implementation must not silently reinterpret architectural decisions.

## 23.2 Ask before proceeding when requirements are unclear

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

## 23.3 Do not silently change architecture

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

## 23.4 Protect completed phases

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

## 23.5 Scaffold is not implementation

A file existing in `src/` does not mean that its functionality belongs
to the current phase.

Future-phase files may remain empty scaffolding.

## 23.6 Do not invent deferred technologies

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

# 24. Verification Requirements

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

# 25. Phase Completion Report

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

# 26. Current Implementation Philosophy

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

# 27. Current High-Level Implementation Order

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

# 28. Initial Non-Goals

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

# 29. Scope Status

``` text
Architecture        → established
Project scaffold    → created
Initial decisions   → recorded
Implementation      → not started

Phase 0             → Planned
Phase 1             → Planned
Phase 2             → Planned
Phase 3             → Planned
Phase 4             → Planned
Phase 5             → Planned
Phase 6             → Planned
Phase 7             → Planned
Phase 8             → Planned
Python scope        → Separate scope.md
Rust Phase 9        → Deferred until Python foundation is ready
Phase 10            → Planned
```

The detailed implementation plan for each phase will be developed and
reviewed separately before that phase is implemented.

------------------------------------------------------------------------

# 30. Final Architectural Principle

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
