//! Language-neutral temporal values and interval boundary semantics.
//!
//! `Instant` uses a Unix timestamp represented by whole seconds and a
//! nanosecond fraction. The seconds value is floor-normalized, including for
//! instants before the Unix epoch. This module intentionally does not parse
//! calendar strings, choose time zones, or implement storage/versioning.

use core::cmp::Ordering;
use core::fmt;

/// One precise point on the Unix timeline.
///
/// `unix_seconds` is the whole-second component and `nanoseconds` is a
/// fractional component in `0..1_000_000_000`. For example, `(-1, 500_000_000)`
/// represents half a second before the Unix epoch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Instant {
    unix_seconds: i64,
    nanoseconds: u32,
}

impl Instant {
    /// Creates an instant from a Unix timestamp with nanosecond precision.
    pub fn new(unix_seconds: i64, nanoseconds: u32) -> Result<Self, TemporalError> {
        if nanoseconds >= 1_000_000_000 {
            return Err(TemporalError::InvalidNanosecond { nanoseconds });
        }

        Ok(Self {
            unix_seconds,
            nanoseconds,
        })
    }

    /// Creates an instant at an exact whole Unix second.
    #[must_use]
    pub const fn from_unix_seconds(unix_seconds: i64) -> Self {
        Self {
            unix_seconds,
            nanoseconds: 0,
        }
    }

    /// Returns the whole-second component of the Unix timestamp.
    #[must_use]
    pub const fn unix_seconds(self) -> i64 {
        self.unix_seconds
    }

    /// Returns the fractional nanosecond component.
    #[must_use]
    pub const fn nanoseconds(self) -> u32 {
        self.nanoseconds
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unix:{}.{:09}",
            self.unix_seconds, self.nanoseconds
        )
    }
}

/// Meaning of an interval endpoint.
///
/// `Open` denotes an unbounded endpoint. `Unknown` means the endpoint exists
/// conceptually but its value is not known. These cases are deliberately
/// distinct: an unknown endpoint is not treated as infinitely distant.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IntervalBoundary {
    /// The endpoint instant belongs to the interval.
    Inclusive(Instant),
    /// The endpoint instant does not belong to the interval.
    Exclusive(Instant),
    /// There is no bound in this direction.
    Open,
    /// A bound exists, but its value is unknown.
    Unknown,
}

impl IntervalBoundary {
    /// Returns the concrete instant, if this boundary has one.
    #[must_use]
    pub const fn instant(&self) -> Option<Instant> {
        match self {
            Self::Inclusive(instant) | Self::Exclusive(instant) => Some(*instant),
            Self::Open | Self::Unknown => None,
        }
    }

    /// Returns whether the boundary represents an unbounded endpoint.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Returns whether the boundary value is unknown.
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    const fn is_inclusive(&self) -> bool {
        matches!(self, Self::Inclusive(_))
    }
}

/// A validated temporal interval.
///
/// The start boundary describes the lower end of the interval and the end
/// boundary describes the upper end. For a concrete equal-point interval,
/// both boundaries must be inclusive; other equal-point combinations are empty
/// and are rejected. Open and unknown boundaries are not conflated.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Interval {
    start: IntervalBoundary,
    end: IntervalBoundary,
}

impl Interval {
    /// Creates and validates an interval.
    pub fn new(start: IntervalBoundary, end: IntervalBoundary) -> Result<Self, TemporalError> {
        if let (Some(start_instant), Some(end_instant)) = (start.instant(), end.instant()) {
            match start_instant.cmp(&end_instant) {
                Ordering::Greater => {
                    return Err(TemporalError::ReversedInterval {
                        start: start_instant,
                        end: end_instant,
                    });
                }
                Ordering::Equal if !(start.is_inclusive() && end.is_inclusive()) => {
                    return Err(TemporalError::EmptyInterval {
                        instant: start_instant,
                    });
                }
                Ordering::Equal | Ordering::Less => {}
            }
        }

        Ok(Self { start, end })
    }

    /// Returns the lower boundary.
    #[must_use]
    pub const fn start(&self) -> &IntervalBoundary {
        &self.start
    }

    /// Returns the upper boundary.
    #[must_use]
    pub const fn end(&self) -> &IntervalBoundary {
        &self.end
    }

    /// Returns whether either end is explicitly unbounded.
    #[must_use]
    pub const fn has_open_boundary(&self) -> bool {
        self.start.is_open() || self.end.is_open()
    }

    /// Returns whether either endpoint is unknown.
    #[must_use]
    pub const fn has_unknown_boundary(&self) -> bool {
        self.start.is_unknown() || self.end.is_unknown()
    }

    /// Checks membership when the boundary information allows a definite answer.
    ///
    /// Returns `Some(true)` when the instant is definitely contained,
    /// `Some(false)` when it is definitely outside, and `None` when an unknown
    /// boundary prevents a definitive result.
    #[must_use]
    pub fn contains(&self, instant: Instant) -> Option<bool> {
        let lower = match self.start {
            IntervalBoundary::Inclusive(bound) => Some(instant >= bound),
            IntervalBoundary::Exclusive(bound) => Some(instant > bound),
            IntervalBoundary::Open => Some(true),
            IntervalBoundary::Unknown => None,
        };
        let upper = match self.end {
            IntervalBoundary::Inclusive(bound) => Some(instant <= bound),
            IntervalBoundary::Exclusive(bound) => Some(instant < bound),
            IntervalBoundary::Open => Some(true),
            IntervalBoundary::Unknown => None,
        };

        match (lower, upper) {
            (Some(false), _) | (_, Some(false)) => Some(false),
            (Some(true), Some(true)) => Some(true),
            _ => None,
        }
    }
}

/// A temporal value whose representative instant is explicitly approximate.
///
/// The center is retained rather than silently converting an approximate date
/// into an exact `Instant`. The enclosing `TemporalValue::Approximate` variant
/// carries the approximation semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Approximate {
    center: Instant,
}

impl Approximate {
    /// Creates an approximate value with a representative center instant.
    #[must_use]
    pub const fn new(center: Instant) -> Self {
        Self { center }
    }

    /// Returns the representative center instant.
    #[must_use]
    pub const fn center(self) -> Instant {
        self.center
    }
}

/// Direction of a one-sided, open-ended interval.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OpenEndedDirection {
    /// Extends without a lower bound and stops at a concrete upper endpoint.
    Before,
    /// Starts at a concrete lower endpoint and extends without an upper bound.
    After,
}

/// A validated one-sided interval with one concrete endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenEnded {
    interval: Interval,
    direction: OpenEndedDirection,
}

impl OpenEnded {
    /// Creates an open-ended value from an interval that has exactly one open
    /// boundary and one concrete inclusive/exclusive endpoint.
    pub fn new(interval: Interval) -> Result<Self, TemporalError> {
        let direction = match (interval.start(), interval.end()) {
            (
                IntervalBoundary::Open,
                IntervalBoundary::Inclusive(_) | IntervalBoundary::Exclusive(_),
            ) => OpenEndedDirection::Before,
            (
                IntervalBoundary::Inclusive(_) | IntervalBoundary::Exclusive(_),
                IntervalBoundary::Open,
            ) => OpenEndedDirection::After,
            _ => return Err(TemporalError::InvalidOpenEndedInterval),
        };

        Ok(Self {
            interval,
            direction,
        })
    }

    /// Creates an interval extending toward the past up to `end`.
    #[must_use]
    pub fn before(end: Instant, inclusive: bool) -> Self {
        let end = if inclusive {
            IntervalBoundary::Inclusive(end)
        } else {
            IntervalBoundary::Exclusive(end)
        };
        Self {
            interval: Interval {
                start: IntervalBoundary::Open,
                end,
            },
            direction: OpenEndedDirection::Before,
        }
    }

    /// Creates an interval beginning at `start` and extending toward the future.
    #[must_use]
    pub fn after(start: Instant, inclusive: bool) -> Self {
        let start = if inclusive {
            IntervalBoundary::Inclusive(start)
        } else {
            IntervalBoundary::Exclusive(start)
        };
        Self {
            interval: Interval {
                start,
                end: IntervalBoundary::Open,
            },
            direction: OpenEndedDirection::After,
        }
    }

    /// Returns the normalized interval represented by this value.
    #[must_use]
    pub const fn interval(&self) -> &Interval {
        &self.interval
    }

    /// Returns the direction in which the interval is open-ended.
    #[must_use]
    pub const fn direction(&self) -> OpenEndedDirection {
        self.direction
    }
}

/// The Phase 4 temporal value vocabulary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TemporalValue {
    /// A precise point on the timeline.
    Instant(Instant),
    /// A possibly bounded or partially unknown interval.
    Interval(Interval),
    /// An approximate point with a preserved representative center.
    Approximate(Approximate),
    /// A one-sided interval with a known endpoint.
    OpenEnded(OpenEnded),
    /// A temporal value that is not known.
    Unknown,
}

impl TemporalValue {
    /// Returns whether this value represents explicitly unknown time.
    #[must_use]
    pub const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Returns whether this value is approximate rather than exact.
    #[must_use]
    pub const fn is_approximate(self) -> bool {
        matches!(self, Self::Approximate(_))
    }
}

/// Errors produced while creating temporal primitives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalError {
    /// A nanosecond fraction must be less than one second.
    InvalidNanosecond {
        /// The supplied invalid fractional component.
        nanoseconds: u32,
    },
    /// A concrete start instant occurs after a concrete end instant.
    ReversedInterval {
        /// Supplied start instant.
        start: Instant,
        /// Supplied end instant.
        end: Instant,
    },
    /// Equal interval endpoints would exclude every instant.
    EmptyInterval {
        /// The shared endpoint.
        instant: Instant,
    },
    /// An open-ended value requires one open boundary and one concrete endpoint.
    InvalidOpenEndedInterval,
}

impl fmt::Display for TemporalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNanosecond { nanoseconds } => write!(
                formatter,
                "nanosecond fraction must be less than 1,000,000,000; got {nanoseconds}"
            ),
            Self::ReversedInterval { start, end } => {
                write!(formatter, "interval starts after it ends: {start} > {end}")
            }
            Self::EmptyInterval { instant } => {
                write!(
                    formatter,
                    "interval is empty at its equal endpoints: {instant}"
                )
            }
            Self::InvalidOpenEndedInterval => formatter.write_str(
                "open-ended interval requires exactly one open boundary and one concrete endpoint",
            ),
        }
    }
}

impl std::error::Error for TemporalError {}

#[cfg(test)]
mod tests {
    use super::{
        Approximate, Instant, Interval, IntervalBoundary, OpenEnded, OpenEndedDirection,
        TemporalError, TemporalValue,
    };

    fn instant(seconds: i64) -> Instant {
        Instant::from_unix_seconds(seconds)
    }

    #[test]
    fn instant_preserves_seconds_and_nanosecond_precision() {
        let instant = Instant::new(12, 345_000_000).expect("valid timestamp fraction");
        assert_eq!(instant.unix_seconds(), 12);
        assert_eq!(instant.nanoseconds(), 345_000_000);
    }

    #[test]
    fn instant_rejects_fraction_outside_one_second() {
        assert_eq!(
            Instant::new(0, 1_000_000_000),
            Err(TemporalError::InvalidNanosecond {
                nanoseconds: 1_000_000_000,
            })
        );
    }

    #[test]
    fn instant_ordering_handles_negative_timestamps_and_fractional_seconds() {
        let before_epoch = Instant::new(-1, 500_000_000).expect("valid pre-epoch instant");
        let after_epoch = Instant::new(0, 0).expect("valid epoch instant");
        assert!(before_epoch < after_epoch);
    }

    #[test]
    fn interval_rejects_reversed_concrete_bounds() {
        assert_eq!(
            Interval::new(
                IntervalBoundary::Inclusive(instant(20)),
                IntervalBoundary::Inclusive(instant(10)),
            ),
            Err(TemporalError::ReversedInterval {
                start: instant(20),
                end: instant(10),
            })
        );
    }

    #[test]
    fn interval_allows_single_instant_only_when_both_boundaries_are_inclusive() {
        let point = Interval::new(
            IntervalBoundary::Inclusive(instant(10)),
            IntervalBoundary::Inclusive(instant(10)),
        )
        .expect("inclusive equal endpoints represent one instant");
        assert_eq!(point.contains(instant(10)), Some(true));

        assert_eq!(
            Interval::new(
                IntervalBoundary::Inclusive(instant(10)),
                IntervalBoundary::Exclusive(instant(10)),
            ),
            Err(TemporalError::EmptyInterval {
                instant: instant(10)
            })
        );
    }

    #[test]
    fn interval_contains_respects_inclusive_and_exclusive_boundaries() {
        let interval = Interval::new(
            IntervalBoundary::Inclusive(instant(10)),
            IntervalBoundary::Exclusive(instant(20)),
        )
        .expect("valid interval");

        assert_eq!(interval.contains(instant(10)), Some(true));
        assert_eq!(interval.contains(instant(15)), Some(true));
        assert_eq!(interval.contains(instant(20)), Some(false));
        assert_eq!(interval.contains(instant(9)), Some(false));
    }

    #[test]
    fn unknown_boundaries_are_not_interpreted_as_open_boundaries() {
        let interval = Interval::new(
            IntervalBoundary::Unknown,
            IntervalBoundary::Inclusive(instant(20)),
        )
        .expect("unknown boundaries are representable");

        assert_eq!(interval.contains(instant(10)), None);
        assert_eq!(interval.contains(instant(25)), Some(false));
        assert!(interval.has_unknown_boundary());
        assert!(!interval.has_open_boundary());
    }

    #[test]
    fn open_ended_values_preserve_direction_and_endpoint_inclusion() {
        let before = OpenEnded::before(instant(20), false);
        assert_eq!(before.direction(), OpenEndedDirection::Before);
        assert_eq!(before.interval().contains(instant(19)), Some(true));
        assert_eq!(before.interval().contains(instant(20)), Some(false));

        let after = OpenEnded::after(instant(10), true);
        assert_eq!(after.direction(), OpenEndedDirection::After);
        assert_eq!(after.interval().contains(instant(10)), Some(true));
        assert_eq!(after.interval().contains(instant(9)), Some(false));
    }

    #[test]
    fn open_ended_constructor_rejects_unknown_or_two_open_boundaries() {
        let unknown_endpoint = Interval::new(IntervalBoundary::Open, IntervalBoundary::Unknown)
            .expect("general interval can preserve unknown bound");
        assert_eq!(
            OpenEnded::new(unknown_endpoint),
            Err(TemporalError::InvalidOpenEndedInterval)
        );

        let fully_open = Interval::new(IntervalBoundary::Open, IntervalBoundary::Open)
            .expect("fully unbounded interval is representable");
        assert_eq!(
            OpenEnded::new(fully_open),
            Err(TemporalError::InvalidOpenEndedInterval)
        );
    }

    #[test]
    fn approximate_temporal_value_keeps_its_representative_center() {
        let approximate = Approximate::new(instant(100));
        let value = TemporalValue::Approximate(approximate);

        assert_eq!(approximate.center(), instant(100));
        assert!(value.is_approximate());
        assert!(!TemporalValue::Instant(instant(100)).is_approximate());
    }

    #[test]
    fn unknown_temporal_value_is_explicit() {
        assert!(TemporalValue::Unknown.is_unknown());
        assert!(!TemporalValue::Instant(instant(0)).is_unknown());
    }
}
