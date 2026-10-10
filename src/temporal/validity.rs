//! Valid-time qualification for canonical semantic objects.
//!
//! Temporal validity answers when represented knowledge applies. It does not
//! model when the KG recorded or changed that knowledge; system-time/history
//! semantics belong to Phase 4 provenance/activity records.

use super::interval::{Approximate, OpenEnded};
use super::interval::{Instant, Interval, TemporalValue};

/// Temporal qualification describing when a semantic object is valid/applicable.
///
/// The wrapper intentionally contains valid-time semantics only. It does not
/// add physical versioning or system-time storage fields.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TemporalValidity {
    value: TemporalValue,
}

impl TemporalValidity {
    /// Creates a temporal-validity value from any supported temporal form.
    #[must_use]
    pub const fn new(value: TemporalValue) -> Self {
        Self { value }
    }

    /// Expresses validity at one exact instant.
    #[must_use]
    pub const fn at(instant: Instant) -> Self {
        Self::new(TemporalValue::Instant(instant))
    }

    /// Expresses validity over an interval.
    #[must_use]
    pub const fn during(interval: Interval) -> Self {
        Self::new(TemporalValue::Interval(interval))
    }

    /// Expresses validity using an approximate representative instant.
    #[must_use]
    pub const fn approximately(value: Approximate) -> Self {
        Self::new(TemporalValue::Approximate(value))
    }

    /// Expresses validity over a one-sided open-ended interval.
    #[must_use]
    pub const fn open_ended(value: OpenEnded) -> Self {
        Self::new(TemporalValue::OpenEnded(value))
    }

    /// Expresses that the valid time is currently unknown.
    #[must_use]
    pub const fn unknown() -> Self {
        Self::new(TemporalValue::Unknown)
    }

    /// Returns the represented valid-time value.
    #[must_use]
    pub const fn value(self) -> TemporalValue {
        self.value
    }

    /// Returns whether valid time is explicitly unknown.
    #[must_use]
    pub const fn is_unknown(self) -> bool {
        self.value.is_unknown()
    }
}

#[cfg(test)]
mod tests {
    use super::{TemporalValidity, TemporalValue};
    use crate::temporal::interval::{Approximate, Instant, Interval, IntervalBoundary, OpenEnded};

    #[test]
    fn validity_can_qualify_a_single_instant_or_interval() {
        let instant = Instant::from_unix_seconds(100);
        assert_eq!(
            TemporalValidity::at(instant).value(),
            TemporalValue::Instant(instant)
        );

        let interval = Interval::new(
            IntervalBoundary::Inclusive(Instant::from_unix_seconds(100)),
            IntervalBoundary::Exclusive(Instant::from_unix_seconds(200)),
        )
        .expect("valid interval");
        assert_eq!(
            TemporalValidity::during(interval).value(),
            TemporalValue::Interval(interval)
        );
    }

    #[test]
    fn validity_preserves_approximate_and_open_ended_semantics() {
        let center = Instant::from_unix_seconds(100);
        let approximate = Approximate::new(center);
        assert_eq!(
            TemporalValidity::approximately(approximate).value(),
            TemporalValue::Approximate(approximate)
        );

        let open_ended = OpenEnded::after(center, true);
        assert_eq!(
            TemporalValidity::open_ended(open_ended).value(),
            TemporalValue::OpenEnded(open_ended)
        );
    }

    #[test]
    fn unknown_valid_time_is_explicit_and_not_a_system_timestamp() {
        let validity = TemporalValidity::unknown();
        assert!(validity.is_unknown());
        assert_eq!(validity.value(), TemporalValue::Unknown);
    }
}
