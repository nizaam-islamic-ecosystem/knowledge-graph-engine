//! Level 3 integration tests for Phase 4 temporal primitives.
//!
//! These tests use only the public crate API and cover temporal variants,
//! interval boundaries, open/unknown distinctions, valid-time qualification,
//! and temporal relationships between strongly typed targets.

use nizaam_knowledge_graph::identity::{KnowledgeAssertionId, SourceId};
use nizaam_knowledge_graph::temporal::{
    Approximate, Instant, Interval, IntervalBoundary, OpenEnded, OpenEndedDirection, TemporalError,
    TemporalRelation, TemporalRelationship, TemporalValidity, TemporalValue,
};

#[test]
fn instant_preserves_nanosecond_precision_and_rejects_invalid_fraction() {
    let instant = Instant::new(-1, 500_000_000).expect("valid nanosecond fraction");
    assert_eq!(instant.unix_seconds(), -1);
    assert_eq!(instant.nanoseconds(), 500_000_000);

    assert_eq!(
        Instant::new(1, 1_000_000_000),
        Err(TemporalError::InvalidNanosecond {
            nanoseconds: 1_000_000_000,
        })
    );
}

#[test]
fn interval_obeys_inclusive_exclusive_open_and_unknown_boundaries() {
    let ten = Instant::from_unix_seconds(10);
    let twenty = Instant::from_unix_seconds(20);
    let interval = Interval::new(
        IntervalBoundary::Exclusive(ten),
        IntervalBoundary::Inclusive(twenty),
    )
    .expect("valid interval");

    assert_eq!(interval.contains(ten), Some(false));
    assert_eq!(
        interval.contains(Instant::from_unix_seconds(11)),
        Some(true)
    );
    assert_eq!(interval.contains(twenty), Some(true));
    assert_eq!(
        interval.contains(Instant::from_unix_seconds(21)),
        Some(false)
    );

    let open_interval = Interval::new(IntervalBoundary::Open, IntervalBoundary::Inclusive(twenty))
        .expect("valid open interval");
    assert!(open_interval.has_open_boundary());
    assert_eq!(
        open_interval.contains(Instant::from_unix_seconds(-100)),
        Some(true)
    );

    let unknown_interval = Interval::new(
        IntervalBoundary::Unknown,
        IntervalBoundary::Inclusive(twenty),
    )
    .expect("unknown boundary remains representable");
    assert!(unknown_interval.has_unknown_boundary());
    assert_eq!(
        unknown_interval.contains(Instant::from_unix_seconds(15)),
        None
    );
    assert_eq!(
        unknown_interval.contains(Instant::from_unix_seconds(21)),
        Some(false)
    );
}

#[test]
fn invalid_reversed_and_empty_intervals_are_rejected() {
    let ten = Instant::from_unix_seconds(10);
    let twenty = Instant::from_unix_seconds(20);

    assert!(matches!(
        Interval::new(
            IntervalBoundary::Inclusive(twenty),
            IntervalBoundary::Inclusive(ten)
        ),
        Err(TemporalError::ReversedInterval { .. })
    ));

    assert_eq!(
        Interval::new(
            IntervalBoundary::Exclusive(ten),
            IntervalBoundary::Inclusive(ten)
        ),
        Err(TemporalError::EmptyInterval { instant: ten })
    );
}

#[test]
fn approximate_open_ended_and_unknown_values_remain_distinct() {
    let center = Instant::from_unix_seconds(1_700_000_000);
    let approximate = Approximate::new(center);
    let open_ended = OpenEnded::after(center, true);

    assert_eq!(approximate.center(), center);
    assert_eq!(open_ended.direction(), OpenEndedDirection::After);
    assert!(open_ended.interval().has_open_boundary());
    assert_ne!(
        TemporalValue::Instant(center),
        TemporalValue::Approximate(approximate)
    );
    assert_ne!(TemporalValue::Unknown, TemporalValue::OpenEnded(open_ended));
}

#[test]
fn valid_time_is_separate_from_temporal_relationships_and_keeps_target_types() {
    let start = Instant::from_unix_seconds(1_000);
    let end = Instant::from_unix_seconds(2_000);
    let interval = Interval::new(
        IntervalBoundary::Inclusive(start),
        IntervalBoundary::Exclusive(end),
    )
    .expect("validity interval");
    let validity = TemporalValidity::during(interval);

    assert_eq!(validity.value(), TemporalValue::Interval(interval));
    assert!(!validity.is_unknown());
    assert!(TemporalValidity::unknown().is_unknown());

    let assertion_id =
        KnowledgeAssertionId::new("assertion-temporal-target").expect("valid assertion identity");
    let source_id = SourceId::new("source-temporal-target").expect("valid source identity");
    let relationship = TemporalRelationship::new(
        assertion_id.clone(),
        TemporalRelation::Before,
        source_id.clone(),
    );
    let inverse = relationship.inverse();

    assert_eq!(relationship.relation(), TemporalRelation::Before);
    assert_eq!(inverse.relation(), TemporalRelation::After);
    assert_eq!(inverse.subject(), &source_id);
    assert_eq!(inverse.object(), &assertion_id);
}
