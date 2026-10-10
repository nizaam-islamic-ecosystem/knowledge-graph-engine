//! Phase 4 temporal primitives and semantics.
//!
//! This module models temporal values, interval boundary semantics, valid-time
//! qualification, and declared temporal relationships. Valid time remains
//! distinct from system time: system-time history is owned by provenance and
//! activity records. Physical persistence and KG versioning are out of scope.

mod interval;
mod temporal_relation;
mod validity;

pub use interval::{
    Approximate, Instant, Interval, IntervalBoundary, OpenEnded, OpenEndedDirection, TemporalError,
    TemporalValue,
};
pub use temporal_relation::{TemporalRelation, TemporalRelationship};
pub use validity::TemporalValidity;

#[cfg(test)]
mod tests {
    use super::{
        Approximate, Instant, Interval, IntervalBoundary, OpenEnded, TemporalRelation,
        TemporalRelationship, TemporalValidity, TemporalValue,
    };
    use std::any::TypeId;

    #[test]
    fn temporal_module_public_boundary_exposes_all_phase4_primitives() {
        let instant = Instant::from_unix_seconds(1_000);
        let interval = Interval::new(IntervalBoundary::Inclusive(instant), IntervalBoundary::Open)
            .expect("valid open-ended interval");
        let approximate = Approximate::new(instant);
        let open_ended = OpenEnded::after(instant, true);
        let validity = TemporalValidity::during(interval);
        let relation = TemporalRelationship::new("event-a", TemporalRelation::Before, "event-b");

        assert_eq!(validity.value(), TemporalValue::Interval(interval));
        assert_eq!(approximate.center(), instant);
        assert_eq!(
            open_ended.interval().start(),
            &IntervalBoundary::Inclusive(instant)
        );
        assert_eq!(relation.relation(), TemporalRelation::Before);
        assert_ne!(
            TypeId::of::<TemporalValidity>(),
            TypeId::of::<TemporalValue>()
        );
    }
}
