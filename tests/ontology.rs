//! Level 3 Phase 3 public-boundary tests for ontology behavior.

use std::collections::BTreeSet;

use nizaam_knowledge_graph::concept::Concept;
use nizaam_knowledge_graph::identity::ConceptId;
use nizaam_knowledge_graph::ontology::{
    Class, ClassId, Ontology, OntologyConstraint, OntologyProperty, OntologyRegistry,
    OntologySnapshotId, Taxonomy,
};
use nizaam_knowledge_graph::relationship::RelationshipPredicate;

fn class(id: &str, label: &str) -> Class {
    Class::new(ClassId::new(id).expect("valid class identity"), label)
}

fn class_id(id: &str) -> ClassId {
    ClassId::new(id).expect("valid class identity")
}

fn property(name: &str, domain: &[&str], range: &[&str]) -> OntologyProperty {
    OntologyProperty::new(
        RelationshipPredicate::new(name).expect("valid relationship predicate"),
        domain.iter().map(|id| class_id(id)).collect(),
        range.iter().map(|id| class_id(id)).collect(),
    )
    .expect("valid ontology property")
}

#[test]
fn class_is_first_class_and_distinct_from_concept() {
    let class = class("class-person", "Person");
    let concept = Concept::new(ConceptId::new("concept-person").unwrap(), "Person");

    assert_eq!(class.label(), "Person");
    assert_eq!(class.id(), &class_id("class-person"));
    assert_ne!(class.id().as_str(), concept.id().as_str());
}

#[test]
fn ontology_supports_primary_and_additional_semantic_categories_via_distinct_types() {
    let class = class("class-person", "Person");
    let concept = Concept::new(ConceptId::new("concept-person").unwrap(), "Person");
    assert_ne!(class.id().as_str(), concept.id().as_str());
}

#[test]
fn ontology_property_formalizes_one_phase2_relationship_predicate_with_domain_and_range() {
    let p = property("has-name", &["class-person"], &["class-name"]);
    assert_eq!(p.predicate().as_str(), "kg.relationship.has-name");
    assert!(p.applies_to_domain(&class_id("class-person")));
    assert!(p.accepts_range(&class_id("class-name")));
}

#[test]
fn generic_relationship_predicate_can_exist_without_ontology_property_constraints() {
    let predicate = RelationshipPredicate::new("related-to").unwrap();
    assert_eq!(predicate.as_str(), "kg.relationship.related-to");
    assert!(OntologyProperty::new(predicate, BTreeSet::new(), BTreeSet::new()).is_err());
}

#[test]
fn ontology_supports_multiple_class_parents_without_inference() {
    let mut ontology = Ontology::new();
    for (id, label) in [
        ("object", "Object"),
        ("agent", "Agent"),
        ("person", "Person"),
    ] {
        ontology.add_class(class(id, label)).unwrap();
    }
    assert!(
        ontology
            .add_class_parent(class_id("person"), class_id("object"))
            .unwrap()
    );
    assert!(
        ontology
            .add_class_parent(class_id("person"), class_id("agent"))
            .unwrap()
    );
    assert_eq!(
        ontology
            .class_taxonomy()
            .parents_of(&class_id("person"))
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn taxonomy_is_structural_and_class_and_concept_taxonomies_are_type_distinct() {
    let mut class_taxonomy = Taxonomy::<ClassId>::new();
    class_taxonomy.insert_node(class_id("person"));
    class_taxonomy.insert_node(class_id("agent"));
    class_taxonomy
        .add_parent(class_id("person"), class_id("agent"))
        .unwrap();

    assert!(class_taxonomy.contains(&class_id("person")));
    assert_eq!(
        class_taxonomy
            .parents_of(&class_id("person"))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn ontology_constraints_are_structurally_representable() {
    let mut ontology = Ontology::new();
    ontology.add_class(class("person", "Person")).unwrap();
    ontology.add_class(class("animal", "Animal")).unwrap();

    let constraint = OntologyConstraint::disjoint_with([class_id("animal")]).unwrap();
    assert!(
        ontology
            .add_class_constraint(class_id("person"), constraint.clone())
            .unwrap()
    );
    assert!(
        ontology
            .class_constraints(&class_id("person"))
            .unwrap()
            .contains(&constraint)
    );
}

#[test]
fn ontology_validation_reports_unknown_domain_or_range_classes() {
    let mut ontology = Ontology::new();
    ontology.add_class(class("person", "Person")).unwrap();
    ontology
        .add_property(property("has-name", &["person"], &["name"]))
        .unwrap();

    let report = ontology.validate();
    assert!(!report.is_valid());
    assert!(
        report
            .errors()
            .iter()
            .any(|e| e.code() == "ontology.property.range.unknown_class")
    );
}

#[test]
fn registry_registers_activates_and_exposes_immutable_snapshot() {
    let mut ontology = Ontology::new();
    ontology.add_class(class("person", "Person")).unwrap();
    ontology.add_class(class("name", "Name")).unwrap();
    ontology
        .add_property(property("has-name", &["person"], &["name"]))
        .unwrap();

    let mut registry = OntologyRegistry::new();
    let id = registry.register(ontology).unwrap();
    registry.activate(&id).unwrap();

    assert!(registry.contains(&id));
    assert_eq!(registry.active_snapshot_id(), Some(&id));
    assert_eq!(
        registry.active_snapshot().unwrap().ontology().class_count(),
        2
    );
    assert!(!id.as_str().is_empty());
}

#[test]
fn snapshot_identity_is_a_distinct_core_backed_type_without_asserting_generated_value() {
    let id = OntologySnapshotId::generate();
    assert!(!id.as_str().is_empty());
}

#[test]
fn islamic_seed_loads_and_contains_foundational_vocabulary() {
    let seed = nizaam_knowledge_graph::ontology::load_islamic_seed().unwrap();
    let ontology = seed.ontology();

    for key in [
        "person",
        "prophet",
        "companion",
        "place",
        "organization",
        "religious_text",
        "quran",
        "hadith",
        "event",
        "concept",
    ] {
        let class_id = seed
            .class_id(key)
            .expect("seed key should resolve to a generated class identity");

        assert!(
            ontology.class(class_id).is_some(),
            "seed key '{key}' should resolve to a registered ontology class"
        );
    }

    assert_eq!(ontology.class_count(), 10);
}

#[test]
fn islamic_seed_establishes_initial_taxonomy() {
    let seed = nizaam_knowledge_graph::ontology::load_islamic_seed().unwrap();
    let ontology = seed.ontology();

    let prophet = seed
        .class_id("prophet")
        .expect("prophet seed key should resolve");
    let person = seed
        .class_id("person")
        .expect("person seed key should resolve");
    let quran = seed
        .class_id("quran")
        .expect("quran seed key should resolve");
    let religious_text = seed
        .class_id("religious_text")
        .expect("religious_text seed key should resolve");

    assert!(
        ontology
            .class_taxonomy()
            .parents_of(prophet)
            .unwrap()
            .contains(person)
    );

    assert!(
        ontology
            .class_taxonomy()
            .parents_of(quran)
            .unwrap()
            .contains(religious_text)
    );
}
