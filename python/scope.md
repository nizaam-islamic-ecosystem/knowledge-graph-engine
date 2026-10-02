# Scope: Nizaam Knowledge Graph Engine — Python ML Layer

The Nizaam Knowledge Graph Engine has two planned computational parts:

```text
KG ENGINE
├── Rust KG Core
└── Python ML Layer
```

This document defines the scope of the **Python ML part** of the Knowledge Graph Engine.

The Rust side remains the authoritative owner of KG semantics, canonical knowledge, graph state, validation, provenance, storage, deterministic reasoning, and Nizaam engine integration.

The Python side is an ML subsystem. Its responsibility is to perform approved machine-learning workloads over representations supplied by the Rust KG and return ML results or derived signals to Rust.

The Python layer must therefore remain subordinate to the KG semantic authority:

```text
Rust KG
    ↓
ML-ready representation
    ↓
Python ML Layer
    ↓
ML result / derived signal
    ↓
Rust validation / governance
    ↓
candidate or derived knowledge
```

> **Important:** This scope defines the Python implementation independently from the Rust implementation phases. The Python layer is intentionally started only after the Rust KG has established its semantic and execution foundation through the appropriate Rust phases.

---

# 1. Mission, Scope & Responsibilities

## 1.1 Mission

The Python ML Layer provides machine-learning capabilities for the Knowledge Graph without becoming the owner of canonical KG knowledge.

Its purpose is to support workloads such as:

```text
feature preparation
embedding generation
similarity
clustering
anomaly detection
pattern discovery
model training
model evaluation
future approved ML workloads
```

The Python layer should answer questions such as:

```text
Which objects are similar?
Which structural patterns appear?
Which objects form clusters?
Which records appear anomalous?
Which representations can be generated for semantic retrieval?
```

It must not independently decide:

```text
what canonical knowledge is
which entity identity is authoritative
which relationship is semantically valid
which claim is approved
which source is authoritative
which evidence is acceptable
```

Those remain Rust/KG responsibilities.

---

# 2. Architectural Position

The Python subsystem is part of the KG Engine rather than a separate Nizaam engine.

Conceptually:

```text
Nizaam Control Plane
        ↓
KG Engine
        ↓
Rust KG Core
        ↓
ML Interface
        ↓
gRPC
        ↓
Python ML Layer
        ↓
ML Result
        ↓
Rust KG Core
        ↓
Validation / Governance
```

The Python layer does not participate directly in the global Nizaam Control Plane.

Core remains responsible for:

```text
engine coordination
routing
universal transport
framing
universal request/response infrastructure
lifecycle
security
global capability infrastructure
```

The KG Rust side remains responsible for:

```text
KG semantics
KG capability
ML request construction
ML result interpretation
validation
governance
canonical KG state
```

Python is responsible for:

```text
ML computation
```

---

# 3. Mandatory Rust / Python Boundary

The boundary is intentionally strict.

## Rust owns

```text
canonical KG objects
Knowledge Assertions
entities
concepts
relationships
graph state
ontology
provenance
evidence
authority
uncertainty
temporal semantics
deterministic reasoning
validation
approval
publication
canonical storage
KG version
KG contracts
ML request semantics
ML result acceptance
```

## Python owns

```text
feature computation
ML model execution
training
embedding generation
similarity computation
clustering
anomaly detection
pattern discovery
model evaluation
ML-specific artifacts
```

## Python must not own

```text
canonical KG storage
canonical entity identity
canonical relationship meaning
approval state
source authority
evidence validity
global KG version
Nizaam Control Plane
```

The following invariant is mandatory:

```text
Python output
    ≠
automatic canonical knowledge
```

---

# 4. ML Result Model

Python must return structured ML results rather than direct KG mutations.

Conceptually:

```text
MLResult
├── result identity
├── operation type
├── model identity
├── model version
├── input reference
├── output
├── confidence / score where applicable
├── execution metadata
└── ML provenance
```

The exact field-level contract will be finalized during the Rust/Python contract phase.

Examples:

```text
EmbeddingResult
SimilarityResult
ClusterResult
AnomalyResult
PatternResult
PredictionResult
```

A result may be consumed by Rust as:

```text
candidate knowledge
derived signal
retrieval signal
ranking signal
feature
```

It must not automatically become a canonical assertion.

---

# 5. ML and Knowledge Distinctions

The Python layer must preserve these distinctions:

```text
Model
    ≠
Prediction
    ≠
Canonical Knowledge
```

```text
Anomaly
    ≠
Incorrectness
```

```text
Pattern
    ≠
Fact
```

```text
Similarity
    ≠
Semantic Equivalence
```

```text
ML Confidence
    ≠
Authority / Reliability
```

```text
Model Version
    ≠
KG Version
```

These distinctions are architectural invariants.

---

# 6. Python Project Structure

The Python project begins under:

```text
python/
```

The initial implementation structure is intentionally small.

A provisional structure is:

```text
python/
├── README.md
├── pyproject.toml
├── src/
│   └── nizaam_kg_ml/
│       ├── __init__.py
│       ├── client/
│       ├── contracts/
│       ├── features/
│       ├── models/
│       ├── embeddings/
│       ├── similarity/
│       ├── clustering/
│       ├── anomaly/
│       ├── patterns/
│       ├── evaluation/
│       ├── provenance/
│       └── runtime/
│
└── tests/
```

This is a provisional scaffold.

It may be:

```text
split
merged
renamed
moved
expanded
```

when implementation demonstrates a better boundary.

The scaffold does not imply that every module must be implemented immediately.

---

# 7. Python Runtime Model

The Python layer must have an explicit execution boundary.

The initial conceptual model is:

```text
Rust KG
    ↓
gRPC request
    ↓
Python ML runtime
    ↓
ML execution
    ↓
gRPC response
    ↓
Rust KG
```

The Python runtime must support:

```text
startup
readiness
request handling
ML operation dispatch
controlled execution
result serialization
failure reporting
shutdown
```

Python runtime lifecycle must remain independent from canonical KG lifecycle state.

A Python ML failure must not corrupt KG state.

---

# 8. gRPC Boundary

The approved Rust ↔ Python communication mechanism is:

```text
gRPC
```

The exact protobuf/message definitions remain an implementation task.

The contract must eventually represent:

```text
ML operation
input references
ML-ready input
model identity
model version
operation configuration
execution constraints
result
status
error
provenance
```

The gRPC layer must not expose the Python implementation details to the Rust semantic layer.

The boundary should remain:

```text
KG semantic request
        ↓
ML contract
        ↓
Python execution
        ↓
ML result
```

rather than:

```text
Rust
   ↓
Python internal class calls
```

---

# 9. ML Input Architecture

Rust must prepare the representation supplied to Python.

The Python layer should receive only the information required for the approved ML operation.

Possible input categories include:

```text
entity features
concept features
graph-derived features
relationship features
lexical features
embedding-ready text
structural graph representation
temporal features
source metadata where explicitly approved
```

Python must not assume unrestricted access to the KG.

The input contract must make the relevant references explicit.

---

# 10. Feature Preparation

Feature preparation is a Python responsibility when it is ML-specific.

It may include:

```text
normalization
encoding
numerical feature construction
categorical representation
graph-derived features
text representation
embedding preparation
feature selection
feature transformation
```

Feature preparation must preserve enough provenance to determine:

```text
which KG objects produced the features
which KG/version supplied them
which feature pipeline produced them
which feature version was used
```

Feature provenance must not be confused with source authority.

---

# 11. Embeddings

The Python layer may provide embedding generation.

Possible embedding targets include:

```text
entities
concepts
lexical forms
assertions
documents
passages
graph neighborhoods
other approved KG representations
```

The exact embedding model and dimensions are not frozen by this scope.

Embedding artifacts must retain:

```text
model identity
model version
feature/input version
KG reference/version
generation metadata
```

An embedding must not automatically imply semantic equivalence.

---

# 12. Similarity

Similarity workloads may operate over:

```text
embeddings
feature vectors
approved KG representations
```

Similarity results may be returned as:

```text
object A
object B
similarity score
model / representation identity
execution metadata
```

A similarity result is an ML-derived signal.

It must not automatically become:

```text
same entity
equivalent concept
semantic relationship
canonical assertion
```

Those interpretations remain under Rust/KG governance.

---

# 13. Clustering

The Python layer may support clustering.

Possible approaches remain open, including:

```text
K-Means
DBSCAN
other approved clustering algorithms
```

The exact algorithm must be selected based on the actual workload.

A cluster represents:

```text
ML-discovered grouping
```

not automatically:

```text
ontology class
semantic category
canonical entity type
```

Cluster outputs must retain:

```text
algorithm
model/configuration
version
input representation
execution metadata
```

---

# 14. Anomaly Detection

The Python layer may support anomaly detection.

Possible approaches remain open.

An anomaly result must be interpreted as:

```text
the model identified an unusual pattern
```

and not:

```text
the KG object is wrong
```

Anomaly outputs must be returned to Rust for interpretation and governance when they may influence KG state.

---

# 15. Pattern Discovery

The Python layer may discover recurring or unusual patterns in approved representations.

Possible outputs include:

```text
frequent structures
similar subgraphs
co-occurrence patterns
embedding neighborhoods
candidate relationships
candidate mappings
```

Pattern discovery is exploratory.

A discovered pattern is not automatically canonical knowledge.

---

# 16. Model Training

Training is separate from inference.

Conceptually:

```text
KG data / approved training representation
        ↓
feature preparation
        ↓
Python training
        ↓
model artifact
        ↓
evaluation
        ↓
approved model
        ↓
inference
```

Training must not directly modify canonical KG knowledge.

The exact training strategy remains open:

```text
batch training
incremental training
scheduled training
manual training
change-volume-triggered training
version-triggered training
```

The important invariant is:

```text
KG writes
    ≠
automatic retraining
```

---

# 17. Model Versioning

Python models must have identities and versions separate from KG versions.

The system must distinguish:

```text
KG Version
Model Version
Feature Version
Training Dataset Version
Embedding Version
```

A model result should make it possible to determine:

```text
which model produced it
which version produced it
which representation was supplied
which KG version was used where applicable
```

---

# 18. Model Provenance

Every important ML artifact should preserve provenance.

This includes:

```text
training data reference
KG version
feature pipeline version
model identity
model version
training configuration
execution metadata
output generation metadata
```

Model provenance is not the same as scholarly/source provenance.

The distinction must remain explicit:

```text
Knowledge Provenance
    ≠
ML Provenance
```

---

# 19. Evaluation

ML implementations must have explicit evaluation boundaries.

Depending on the workload, evaluation may include:

```text
accuracy
precision
recall
F1
ROC-AUC
clustering metrics
similarity evaluation
reconstruction/error metrics
human evaluation
domain-specific evaluation
```

The exact metrics are not frozen.

Evaluation must be appropriate to the actual ML operation.

A model must not be treated as authoritative merely because its numerical evaluation is strong.

---

# 20. ML Storage and Artifacts

The Python layer may eventually require storage for:

```text
model artifacts
feature artifacts
embedding artifacts
evaluation artifacts
training metadata
```

The exact storage technology is not frozen.

These artifacts must not become a second canonical KG store.

Conceptually:

```text
KG Canonical Storage
        ≠
ML Artifact Storage
```

---

# 21. Failure and Isolation Model

Python ML must be isolated from canonical KG integrity.

If Python fails:

```text
ML request fails
        ↓
Rust receives controlled failure
        ↓
KG remains valid
```

The Python layer must not partially mutate canonical state.

Failures should be represented through the approved ML contract.

Potential failure classes include:

```text
invalid input
unsupported operation
model unavailable
model loading failure
resource exhaustion
timeout
serialization failure
internal ML failure
version incompatibility
```

The exact error mapping will be finalized with the Rust integration contract.

---

# 22. Security Boundary

The Python layer must not bypass KG security boundaries.

It must not receive unrestricted access merely because it is part of the same repository.

Access should be limited to:

```text
approved gRPC operations
approved inputs
approved model operations
approved result paths
```

Python must not directly access:

```text
canonical KG database
KG private storage internals
Core Control Plane internals
unrestricted engine state
```

unless a later architectural decision explicitly authorizes such access.

---

# 23. Governance of ML Output

The Rust KG remains responsible for deciding what an ML result means.

Conceptually:

```text
Python
   ↓
ML result
   ↓
Rust
   ↓
validation
   ↓
provenance
   ↓
authority / policy
   ↓
approval
   ↓
candidate / derived knowledge
```

The Python layer must never implement a hidden publication path.

In particular:

```text
ML prediction
    ≠
approved assertion
```

and:

```text
ML candidate relationship
    ≠
canonical relationship
```

---

# 24. Python Testing Architecture

Testing is required at every Python phase.

## Unit tests

Test:

```text
feature preparation
model wrappers
embedding generation
similarity
clustering
anomaly detection
pattern discovery
serialization
validation
provenance construction
```

## Integration tests

Test:

```text
gRPC request
Python dispatch
ML execution
result serialization
Rust-compatible response
failure propagation
version compatibility
```

## Boundary tests

Must verify:

```text
Python cannot directly mutate canonical KG state
ML result is not automatically canonical knowledge
model version remains distinct from KG version
ML provenance remains distinct from knowledge provenance
failure does not corrupt KG state
```

---

# 25. Python Implementation Phases

The Python implementation is intentionally separate from the Rust phase numbering.

---

## Python Phase 0: Python Workspace & Runtime Foundation

### Status

**Planned**

### Goal

Create a clean, independently testable Python ML workspace.

### Scope

```text
Python package
pyproject.toml
package structure
runtime entry point
configuration foundation
logging foundation
testing foundation
health/readiness foundation
basic operation dispatch abstraction
```

### Must not implement

```text
ML algorithms
training pipelines
embeddings
canonical KG mutation
production model registry
```

### Done when

The Python package can:

```text
install
 ↓
start
 ↓
initialize
 ↓
report readiness
 ↓
accept an internal operation
 ↓
return a structured response
 ↓
shutdown
```

without requiring a live KG database.

---

## Python Phase 1: ML Contract & gRPC Foundation

### Status

**Planned**

### Goal

Establish the Rust ↔ Python contract.

### Scope

```text
protobuf definitions
ML request
ML response
ML operation identity
input references
result identity
status
error model
model identity
model version
provenance metadata
gRPC server
gRPC client test harness
```

### Done when

Rust and Python can exchange a minimal valid ML request and structured ML result through gRPC.

---

## Python Phase 2: Input & Feature Representation

### Status

**Planned**

### Goal

Establish the ML-ready representation supplied to Python.

### Scope

```text
input adapters
feature representation
feature metadata
feature version
KG reference/version metadata
feature provenance
normalization
approved transformation pipeline
```

### Done when

Python can consume deterministic ML-ready input and reproduce the same feature representation under the same input/version conditions.

---

## Python Phase 3: Embedding & Similarity Foundation

### Status

**Planned**

### Goal

Implement the first useful ML workloads without making ML output canonical KG knowledge.

### Scope

```text
embedding interface
embedding generation
embedding metadata
similarity computation
similarity result contract
embedding provenance
```

### Done when

Python can generate versioned embeddings and similarity results and return them through the approved ML contract.

---

## Python Phase 4: Clustering, Anomaly Detection & Pattern Discovery

### Status

**Planned**

### Goal

Introduce exploratory ML capabilities over approved KG representations.

### Scope

```text
clustering interface
clustering implementation
anomaly detection interface
anomaly implementation
pattern discovery interface
pattern discovery implementation
result provenance
```

### Done when

The supported ML workloads can produce structured results while preserving the distinction between:

```text
ML signal
    ≠
canonical knowledge
```

---

## Python Phase 5: Training & Model Lifecycle

### Status

**Planned**

### Goal

Establish repeatable training and model lifecycle management.

### Scope

```text
training input
training pipeline
model artifact
model identity
model version
training metadata
training dataset reference
evaluation handoff
model loading
model lifecycle
```

### Done when

A model can be trained, identified, versioned, stored as an ML artifact, and loaded for inference without directly changing canonical KG state.

---

## Python Phase 6: Evaluation & ML Provenance

### Status

**Planned**

### Goal

Make ML outputs reproducible and evaluable.

### Scope

```text
evaluation framework
operation-specific metrics
model evaluation artifacts
ML provenance
feature provenance
KG-version association
reproducibility metadata
```

### Done when

An ML result can be traced back to:

```text
model
model version
feature representation
input/KG version
execution metadata
```

and evaluated using appropriate metrics.

---

## Python Phase 7: Rust Governance Integration

### Status

**Planned**

### Goal

Connect Python ML results to the Rust KG governance boundary.

### Scope

```text
ML result ingestion by Rust
result validation
result provenance
candidate result handling
approval boundary
failure handling
version compatibility
controlled ML-to-KG workflows
```

### Done when

The complete path works:

```text
Rust KG
   ↓
ML request
   ↓
Python
   ↓
ML result
   ↓
Rust validation
   ↓
governance
   ↓
candidate / derived knowledge
```

without Python directly mutating canonical KG state.

---

## Python Phase 8: Production Hardening

### Status

**Planned**

### Goal

Make the Python ML subsystem operationally reliable.

### Scope

```text
resource limits
timeouts
concurrency
failure isolation
observability
security hardening
configuration
performance tests
regression tests
compatibility tests
```

### Done when

Python ML failures remain isolated, supported operations are observable, resource behavior is bounded, and the Rust integration remains stable.

---

# 26. Relationship to Rust Phase 9

Rust `Phase 9` is:

```text
Python ML Layer & Rust ↔ Python Integration
```

It should **not** be implemented as an isolated Rust feature before the Python foundation exists.

The intended order is:

```text
RUST
Phase 0
   ↓
Phase 1
   ↓
Phase 2
   ↓
Phase 3
   ↓
Phase 4
   ↓
Phase 5
   ↓
Phase 6
   ↓
Phase 7
   ↓
Phase 8
   ↓
Python foundation
   ↓
Python ML phases
   ↓
Rust Phase 9
   ↓
Full Rust/Python integration
```

Rust Phase 9 then becomes the integration phase that connects the already-established Python ML subsystem to the mature Rust KG.

This prevents the Rust implementation from inventing Python contracts prematurely.

---

# 27. Initial Python Non-Goals

The Python implementation does not initially attempt to provide:

```text
canonical KG storage
canonical graph ownership
entity authority
ontology ownership
KG validation
scholarly authority decisions
canonical relationship creation
automatic publication
unbounded autonomous knowledge generation
global Nizaam orchestration
Core Control Plane implementation
Arabic linguistic analysis
final ML model architecture
final GPU architecture
final model registry technology
final feature-store technology
```

---

# 28. Deferred Python Decisions

The following remain open until the relevant implementation phase:

```text
exact ML algorithms
neural architecture
GNN vs Node2Vec vs other graph representation methods
K-Means vs DBSCAN vs other clustering methods
embedding dimensions
training frequency
incremental vs full retraining
GPU architecture
model storage technology
model registry
feature storage
online vs offline inference
exact evaluation metrics
Python package layout refinements
deployment topology
resource allocation
```

These decisions must be made from actual KG workloads rather than frozen prematurely.

---

# 29. Mandatory Python Architectural Invariants

The Python implementation must preserve:

```text
Rust owns canonical KG semantics
Python owns ML computation
Python does not directly mutate canonical KG storage
ML output is not automatically canonical knowledge
Model version is distinct from KG version
ML provenance is distinct from knowledge provenance
ML confidence is distinct from authority
Anomaly is not synonymous with incorrectness
Pattern is not synonymous with fact
Similarity is not synonymous with semantic equivalence
Training is separate from KG writes
Inference failure does not corrupt KG state
```

---

# 30. Verification Requirements

A Python phase is not complete merely because the code runs.

Verification should include, where applicable:

```text
package installation
unit tests
integration tests
gRPC tests
failure-path tests
serialization tests
provenance tests
version compatibility tests
Rust/Python boundary tests
performance/resource tests
```

For the complete repository, Rust verification remains required as well.

The final integrated system should be verified with:

```text
Rust tests
Python tests
Rust ↔ Python integration tests
Nizaam conformance tests
```

---

# 31. Phase Completion Report

Each Python phase completion report should state:

```text
what was implemented
what Python files were changed
what Rust contract files were changed, if any
what tests were added
verification results
ML algorithms selected
model/version decisions
provenance behavior
boundary invariants verified
unresolved issues
```

A Python phase must not be declared complete if:

```text
Python can mutate canonical KG state
ML output bypasses governance
model/version metadata is missing where required
provenance is lost
integration tests fail
Rust KG behavior regresses
```

---

# 32. High-Level Python Implementation Order

```text
Python Phase 0
Workspace & runtime foundation
        ↓
Python Phase 1
ML contract & gRPC foundation
        ↓
Python Phase 2
Input & feature representation
        ↓
Python Phase 3
Embedding & similarity
        ↓
Python Phase 4
Clustering, anomaly detection & pattern discovery
        ↓
Python Phase 5
Training & model lifecycle
        ↓
Python Phase 6
Evaluation & ML provenance
        ↓
Python Phase 7
Rust governance integration
        ↓
Python Phase 8
Production hardening
        ↓
Rust Phase 9
Final Rust/Python integration
```

---

# 33. Scope Status

```text
Python architecture       → established
Python workspace           → scaffold exists
Python implementation      → not started

Python Phase 0             → Planned
Python Phase 1             → Planned
Python Phase 2             → Planned
Python Phase 3             → Planned
Python Phase 4             → Planned
Python Phase 5             → Planned
Python Phase 6             → Planned
Python Phase 7             → Planned
Python Phase 8             → Planned

Rust Phase 9               → Deferred until Python foundation is ready
```

---

# 34. Final Principle

The Python layer exists to extend the Knowledge Graph with machine-learning capabilities without weakening the KG's semantic authority.

The intended relationship is:

```text
Rust
→ knows what the knowledge means

Python
→ computes ML signals about approved representations

Rust
→ decides how those signals may participate in the knowledge system
```

Therefore:

```text
Python is an ML subsystem,
not a second Knowledge Graph.
```
