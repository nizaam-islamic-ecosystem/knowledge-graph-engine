//! Shared structural taxonomy infrastructure for Phase 3.
//!
//! Taxonomy stores explicit parent/child structure only. It does not execute
//! inheritance, inference, relationship composition, or graph traversal.
//!
//! The implementation is generic so the same structural mechanism can be used
//! for classes and concepts while the concrete node types remain distinct:
//! `Taxonomy<ClassId>` cannot be mixed with `Taxonomy<ConceptId>`.
//!
//! Part-whole semantics remain relationship semantics and are therefore not
//! encoded as taxonomy edges.

use core::fmt;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::concept::ConceptId;

use super::class::ClassId;

/// A structural hierarchy over an existing strongly typed node identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Taxonomy<N> {
    parents: BTreeMap<N, BTreeSet<N>>,
    children: BTreeMap<N, BTreeSet<N>>,
}

impl<N> Default for Taxonomy<N> {
    fn default() -> Self {
        Self {
            parents: BTreeMap::new(),
            children: BTreeMap::new(),
        }
    }
}

impl<N> Taxonomy<N>
where
    N: Clone + Ord,
{
    /// Creates an empty taxonomy.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a node into the taxonomy.
    ///
    /// Returns `true` when the node was not already present.
    pub fn insert_node(&mut self, node: N) -> bool {
        let inserted = !self.parents.contains_key(&node);

        self.parents.entry(node.clone()).or_default();
        self.children.entry(node).or_default();

        inserted
    }

    /// Adds an explicit parent relationship from `child` to `parent`.
    ///
    /// Multiple parents are supported. Cycles are rejected structurally, but
    /// no transitive closure or inferred hierarchy is materialized.
    pub fn add_parent(&mut self, child: N, parent: N) -> Result<bool, TaxonomyError<N>> {
        if child == parent {
            return Err(TaxonomyError::SelfParent { node: child });
        }

        if !self.contains(&child) {
            return Err(TaxonomyError::MissingNode { node: child });
        }

        if !self.contains(&parent) {
            return Err(TaxonomyError::MissingNode { node: parent });
        }

        if self
            .parents
            .get(&child)
            .is_some_and(|parents| parents.contains(&parent))
        {
            return Ok(false);
        }

        if self.has_path(&child, &parent) {
            return Err(TaxonomyError::Cycle { child, parent });
        }

        self.parents
            .get_mut(&child)
            .expect("child was checked above")
            .insert(parent.clone());

        self.children
            .get_mut(&parent)
            .expect("parent was checked above")
            .insert(child);

        Ok(true)
    }

    /// Returns whether a node exists in the taxonomy.
    #[must_use]
    pub fn contains(&self, node: &N) -> bool {
        self.parents.contains_key(node)
    }

    /// Returns the number of nodes in the taxonomy.
    #[must_use]
    pub fn len(&self) -> usize {
        self.parents.len()
    }

    /// Returns whether the taxonomy contains no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parents.is_empty()
    }

    /// Returns the direct parents of a node.
    #[must_use]
    pub fn parents_of(&self, node: &N) -> Option<&BTreeSet<N>> {
        self.parents.get(node)
    }

    /// Returns the direct children of a node.
    #[must_use]
    pub fn children_of(&self, node: &N) -> Option<&BTreeSet<N>> {
        self.children.get(node)
    }

    /// Iterates over all taxonomy nodes in deterministic order.
    pub fn nodes(&self) -> impl Iterator<Item = &N> {
        self.parents.keys()
    }

    fn has_path(&self, start: &N, target: &N) -> bool {
        let mut queue = VecDeque::new();
        let mut visited = BTreeSet::new();

        queue.push_back(start.clone());

        while let Some(current) = queue.pop_front() {
            if &current == target {
                return true;
            }

            if !visited.insert(current.clone()) {
                continue;
            }

            if let Some(children) = self.children.get(&current) {
                queue.extend(children.iter().cloned());
            }
        }

        false
    }
}

/// Structural taxonomy failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaxonomyError<N> {
    /// A node cannot be its own parent.
    SelfParent {
        /// The invalid node.
        node: N,
    },

    /// A referenced child or parent has not been inserted into the taxonomy.
    MissingNode {
        /// The missing node.
        node: N,
    },

    /// Adding the relation would create a cycle.
    Cycle {
        /// The child side of the attempted relation.
        child: N,

        /// The parent side of the attempted relation.
        parent: N,
    },
}

impl<N> fmt::Display for TaxonomyError<N>
where
    N: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfParent { node } => {
                write!(
                    formatter,
                    "taxonomy node cannot be its own parent: {node:?}"
                )
            }
            Self::MissingNode { node } => {
                write!(formatter, "taxonomy node is not registered: {node:?}")
            }
            Self::Cycle { child, parent } => {
                write!(
                    formatter,
                    "taxonomy relation would create a cycle: \
                     child={child:?}, parent={parent:?}"
                )
            }
        }
    }
}

impl<N> std::error::Error for TaxonomyError<N> where N: fmt::Debug {}

/// Structural class taxonomy.
pub type ClassTaxonomy = Taxonomy<ClassId>;

/// Structural concept taxonomy.
pub type ConceptTaxonomy = Taxonomy<ConceptId>;

#[cfg(test)]
mod tests {
    use super::{ClassTaxonomy, ConceptTaxonomy, Taxonomy, TaxonomyError};

    use crate::concept::ConceptId;
    use crate::ontology::class::ClassId;

    fn class_id(value: &str) -> ClassId {
        ClassId::new(value).expect("valid class identity")
    }

    fn concept_id(value: &str) -> ConceptId {
        ConceptId::new(value).expect("valid concept identity")
    }

    #[test]
    fn taxonomy_supports_multiple_direct_parents() {
        let parent_a = class_id("class-semantic-object");
        let parent_b = class_id("class-agent");
        let child = class_id("class-person");

        let mut taxonomy = ClassTaxonomy::new();

        taxonomy.insert_node(parent_a.clone());
        taxonomy.insert_node(parent_b.clone());
        taxonomy.insert_node(child.clone());

        assert!(
            taxonomy
                .add_parent(child.clone(), parent_a.clone())
                .expect("first parent should be accepted")
        );

        assert!(
            taxonomy
                .add_parent(child.clone(), parent_b.clone())
                .expect("second parent should be accepted")
        );

        let parents = taxonomy.parents_of(&child).expect("child should exist");

        assert_eq!(parents.len(), 2);
        assert!(parents.contains(&parent_a));
        assert!(parents.contains(&parent_b));
    }

    #[test]
    fn duplicate_parent_relations_are_idempotent() {
        let parent = class_id("class-agent");
        let child = class_id("class-person");

        let mut taxonomy = ClassTaxonomy::new();

        taxonomy.insert_node(parent.clone());
        taxonomy.insert_node(child.clone());

        assert!(
            taxonomy
                .add_parent(child.clone(), parent.clone())
                .expect("first relation should be accepted")
        );

        assert!(
            !taxonomy
                .add_parent(child, parent)
                .expect("duplicate relation should be harmless")
        );
    }

    #[test]
    fn self_parent_relations_are_rejected() {
        let node = class_id("class-person");

        let mut taxonomy = ClassTaxonomy::new();

        taxonomy.insert_node(node.clone());

        assert_eq!(
            taxonomy.add_parent(node.clone(), node.clone()),
            Err(TaxonomyError::SelfParent { node })
        );
    }

    #[test]
    fn cyclic_hierarchy_relations_are_rejected() {
        let grandparent = class_id("class-thing");
        let parent = class_id("class-agent");
        let child = class_id("class-person");

        let mut taxonomy = ClassTaxonomy::new();

        taxonomy.insert_node(grandparent.clone());
        taxonomy.insert_node(parent.clone());
        taxonomy.insert_node(child.clone());

        taxonomy
            .add_parent(parent.clone(), grandparent)
            .expect("first hierarchy relation should succeed");

        taxonomy
            .add_parent(child.clone(), parent.clone())
            .expect("second hierarchy relation should succeed");

        assert_eq!(
            taxonomy.add_parent(parent.clone(), child.clone()),
            Err(TaxonomyError::Cycle {
                child: parent,
                parent: child,
            })
        );
    }

    #[test]
    fn missing_nodes_are_rejected() {
        let child = class_id("class-person");
        let parent = class_id("class-agent");

        let mut taxonomy = ClassTaxonomy::new();

        taxonomy.insert_node(child.clone());

        assert_eq!(
            taxonomy.add_parent(child, parent.clone()),
            Err(TaxonomyError::MissingNode { node: parent })
        );
    }

    #[test]
    fn class_and_concept_taxonomies_are_distinct_types() {
        let class_taxonomy = ClassTaxonomy::new();
        let concept_taxonomy = ConceptTaxonomy::new();

        assert_ne!(
            std::any::TypeId::of::<ClassTaxonomy>(),
            std::any::TypeId::of::<ConceptTaxonomy>()
        );

        assert!(class_taxonomy.is_empty());
        assert!(concept_taxonomy.is_empty());
    }

    #[test]
    fn shared_generic_structure_can_be_instantiated_for_concepts() {
        let parent = concept_id("concept-abstract");
        let child = concept_id("concept-person");

        let mut taxonomy = Taxonomy::<ConceptId>::new();

        taxonomy.insert_node(parent.clone());
        taxonomy.insert_node(child.clone());

        taxonomy
            .add_parent(child.clone(), parent.clone())
            .expect("concept hierarchy relation should succeed");

        assert_eq!(taxonomy.parents_of(&child).unwrap().len(), 1);

        assert!(taxonomy.parents_of(&child).unwrap().contains(&parent));
    }

    #[test]
    fn taxonomy_is_structural_and_does_not_encode_part_whole_semantics() {
        let parent = class_id("class-object");
        let child = class_id("class-component");

        let mut taxonomy = ClassTaxonomy::new();

        taxonomy.insert_node(parent.clone());
        taxonomy.insert_node(child.clone());

        taxonomy
            .add_parent(child.clone(), parent.clone())
            .expect("structural hierarchy relation should succeed");

        assert!(taxonomy.parents_of(&child).unwrap().contains(&parent));

        // `part-of` remains a relationship predicate in Phase 2; this taxonomy
        // stores only the structural parent relation represented by node types.
    }
}
