//! Module for nanosecond timestamp and duration structures: [`Nanoseconds`] and [`NsDuration`].
use chrono::{DateTime, Local, SecondsFormat, TimeZone, Utc};
use serde::de::{self, MapAccess};
use serde::{Deserialize, Serialize, de::Visitor};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::time::SystemTimeError;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

// Note to self - don't ever touch this again
// ...oh well... here we go again.

// TODO: examples/docs are outdated.

#[derive(Debug, Error)]
pub enum Error {
    #[error("system time conversion error: {0}")]
    SystemTimeError(#[from] SystemTimeError),
    #[error("out of range of SystemTime type")]
    OutOfSystemTimeRange,
    #[error("out of range of Duration type")]
    OutOfDurationRange,
    #[error("out of range")]
    OutOfRange,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializableStruct {
    pub seconds: i64,
    pub frac: u32,
}

impl SerializableStruct {
    pub fn nanos(self) -> Nanoseconds {
        Nanoseconds::from_secs(self.seconds) + (self.frac as i128)
    }
}

/// Timestamp expressed as nanoseconds since [`UNIX_EPOCH`].
///
/// This type uses [`Nanoseconds`] internally - all available methods are derived from [`Nanoseconds`].
/// Please look into the documentation of [`Nanoseconds`] for more examples.
///
/// # Serialization and deserialization
/// This type is serialized as a structure containing the amount of whole seconds and a nanosecond fraction remainder.
///
/// This type can be deserialized from any integer value (although if the integer is larger than [`i128::MAX`] then the conversion will fail),
/// as well as the same kind of structure that it gets serialized as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NsTimestamp(Nanoseconds);

impl NsTimestamp {
    pub const UNIX_EPOCH: NsTimestamp = NsTimestamp(Nanoseconds::ZERO);
    pub const MIN: NsTimestamp = NsTimestamp(Nanoseconds::MIN);
    pub const MAX: NsTimestamp = NsTimestamp(Nanoseconds::MAX);

    pub fn nanos(&self) -> Nanoseconds {
        self.0
    }

    pub fn now() -> Self {
        Self(Nanoseconds::now())
    }

    pub fn as_secs(self) -> i128 {
        self.0.as_secs()
    }

    pub fn frac(self) -> u32 {
        self.0.frac()
    }

    pub fn as_millis(self) -> i128 {
        self.0.as_millis()
    }

    pub fn as_micros(self) -> i128 {
        self.0.as_micros()
    }

    pub fn as_nanos(self) -> i128 {
        self.0.as_nanos()
    }

    pub fn since_epoch(self) -> NsDuration {
        NsDuration(self.0)
    }

    pub fn from_secs(secs: i64) -> Self {
        Self(Nanoseconds::from_secs(secs))
    }

    pub fn from_secs_f64(secs: f64) -> Self {
        Self(Nanoseconds::from_secs_f64(secs))
    }

    pub fn try_from_secs(secs: i128) -> Result<Self, Error> {
        Nanoseconds::try_from_secs(secs).map(Self)
    }

    pub fn from_millis(millis: i64) -> Self {
        Self(Nanoseconds::from_millis(millis))
    }

    pub fn try_from_millis(millis: i128) -> Result<Self, Error> {
        Nanoseconds::try_from_millis(millis).map(Self)
    }

    pub fn from_micros(micros: i64) -> Self {
        Self(Nanoseconds::from_micros(micros))
    }

    pub fn try_from_micros(micros: i128) -> Result<Self, Error> {
        Nanoseconds::try_from_micros(micros).map(Self)
    }

    pub fn from_nanos(nanos: i128) -> Self {
        Self(Nanoseconds::from_nanos(nanos))
    }

    pub fn from_since_epoch<D: Into<NsDuration>>(duration: D) -> Self {
        Self(duration.into().0)
    }

    pub fn to_date_time_string_utc(self) -> String {
        self.0.to_date_time_string_utc()
    }

    pub fn to_date_time_string_local(self) -> String {
        self.0.to_date_time_string_local()
    }

    pub fn invert_with_origin(self, origin: Self) -> Self {
        Self(self.0.invert_with_origin(origin.0))
    }

    fn as_serializable(self) -> SerializableStruct {
        SerializableStruct {
            seconds: self.as_secs() as i64,
            frac: self.frac(),
        }
    }
}

impl fmt::Display for NsTimestamp {
    /// Display [`NsTimestamp`] as a UTC datetime string, and the amount of nanoseconds since [`UNIX_EPOCH`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.to_date_time_string_local(), self.0)
    }
}

impl Add<i128> for NsTimestamp {
    type Output = Self;
    fn add(self, rhs: i128) -> Self::Output {
        Self(self.0.add(rhs))
    }
}

impl Add<Nanoseconds> for NsTimestamp {
    type Output = Self;
    fn add(self, rhs: Nanoseconds) -> Self::Output {
        Self(self.0.add(rhs.as_nanos()))
    }
}

impl Add<NsDuration> for NsTimestamp {
    type Output = Self;
    fn add(self, rhs: NsDuration) -> Self::Output {
        Self(self.0.add(rhs.as_nanos()))
    }
}

impl Sub<i128> for NsTimestamp {
    type Output = Self;
    fn sub(self, rhs: i128) -> Self::Output {
        Self(self.0.sub(rhs))
    }
}

impl Sub<Nanoseconds> for NsTimestamp {
    type Output = Self;
    fn sub(self, rhs: Nanoseconds) -> Self::Output {
        Self(self.0.sub(rhs.as_nanos()))
    }
}

impl Sub<NsDuration> for NsTimestamp {
    type Output = Self;
    fn sub(self, rhs: NsDuration) -> Self::Output {
        Self(self.0.sub(rhs.as_nanos()))
    }
}

impl Sub<NsTimestamp> for NsTimestamp {
    type Output = NsDuration;
    fn sub(self, rhs: NsTimestamp) -> Self::Output {
        NsDuration(self.0.sub(rhs.as_nanos()))
    }
}

impl From<i128> for NsTimestamp {
    fn from(value: i128) -> Self {
        Self(Nanoseconds::from(value))
    }
}

impl TryFrom<u128> for NsTimestamp {
    type Error = Error;
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Nanoseconds::try_from(value).map(Self)
    }
}

impl From<SystemTime> for NsTimestamp {
    fn from(value: SystemTime) -> Self {
        Self(Nanoseconds::from(value))
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for NsTimestamp {
    fn from(value: DateTime<Tz>) -> Self {
        Self(Nanoseconds::from(value))
    }
}

impl TryFrom<NsTimestamp> for SystemTime {
    type Error = Error;
    fn try_from(value: NsTimestamp) -> Result<Self, Self::Error> {
        Self::try_from(value.0)
    }
}

impl<Tz: TimeZone> TryFrom<NsTimestamp> for DateTime<Tz>
where
    DateTime<Tz>: From<SystemTime>,
{
    type Error = Error;
    fn try_from(value: NsTimestamp) -> Result<Self, Self::Error> {
        Self::try_from(value.0)
    }
}

impl From<Nanoseconds> for NsTimestamp {
    fn from(value: Nanoseconds) -> Self {
        Self(value)
    }
}

impl Serialize for NsTimestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_serializable().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NsTimestamp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // if let Ok(a) = deserializer.deserialize_i128(NanosecondsVisitor) {
        //     return Ok(NsTimestamp(a));
        // }
        Nanoseconds::deserialize(deserializer).map(Self)
    }
}

/// A timestamp relative to some local zero value.
///
/// This type should be used for storing timestamps local to some specific value, for example timestamps within a video.
/// It doesn't really make sense to think of "5 seconds and 200 milliseconds into the video" as a duration,
/// so that's why this alias exists.
///
/// This type is an alias for [`NsDuration`].
pub type NsLocalTimestamp = NsDuration;

/// Duration expressed using nanoseconds.
///
/// This type uses [`Nanoseconds`] internally - all available methods are derived from [`Nanoseconds`].
/// Please look into the documentation of [`Nanoseconds`] for more examples.
///
/// # Serialization and deserialization
/// This type is serialized as a structure containing the amount of whole seconds and a nanosecond fraction remainder.
///
/// This type can be deserialized from any integer value (although if the integer is larger than [`i128::MAX`] then the conversion will fail),
/// as well as the same kind of structure that it gets serialized as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NsDuration(Nanoseconds);

impl NsDuration {
    pub const ZERO: NsTimestamp = NsTimestamp(Nanoseconds::ZERO);
    pub const MIN: NsTimestamp = NsTimestamp(Nanoseconds::MIN);
    pub const MAX: NsTimestamp = NsTimestamp(Nanoseconds::MAX);

    pub fn nanos(&self) -> Nanoseconds {
        self.0
    }

    pub fn as_secs(self) -> i128 {
        self.0.as_secs()
    }

    pub fn frac(self) -> u32 {
        self.0.frac()
    }

    pub fn as_millis(self) -> i128 {
        self.0.as_millis()
    }

    pub fn as_micros(self) -> i128 {
        self.0.as_micros()
    }

    pub fn as_nanos(self) -> i128 {
        self.0.as_nanos()
    }

    pub fn as_timestamp(self) -> NsTimestamp {
        NsTimestamp(self.0)
    }

    pub fn try_as_std_duration(self) -> Result<(bool, Duration), Error> {
        self.0.try_into()
    }

    pub fn from_secs(secs: i64) -> Self {
        Self(Nanoseconds::from_secs(secs))
    }

    pub fn from_secs_f64(secs: f64) -> Self {
        Self(Nanoseconds::from_secs_f64(secs))
    }

    pub fn try_from_secs(secs: i128) -> Result<Self, Error> {
        Nanoseconds::try_from_secs(secs).map(Self)
    }

    pub fn from_millis(millis: i64) -> Self {
        Self(Nanoseconds::from_millis(millis))
    }

    pub fn try_from_millis(millis: i128) -> Result<Self, Error> {
        Nanoseconds::try_from_millis(millis).map(Self)
    }

    pub fn from_micros(micros: i64) -> Self {
        Self(Nanoseconds::from_micros(micros))
    }

    pub fn try_from_micros(micros: i128) -> Result<Self, Error> {
        Nanoseconds::try_from_micros(micros).map(Self)
    }

    pub fn from_nanos(nanos: i128) -> Self {
        Self(Nanoseconds::from_nanos(nanos))
    }

    pub fn from_timestamp<T: Into<NsTimestamp>>(timestamp: T) -> Self {
        Self(timestamp.into().0)
    }

    fn as_serializable(self) -> SerializableStruct {
        SerializableStruct {
            seconds: self.as_secs() as i64,
            frac: self.frac(),
        }
    }
}

impl Add<i128> for NsDuration {
    type Output = Self;
    fn add(self, rhs: i128) -> Self::Output {
        Self(self.0.add(rhs))
    }
}

impl Add<Nanoseconds> for NsDuration {
    type Output = Self;
    fn add(self, rhs: Nanoseconds) -> Self::Output {
        Self(self.0.add(rhs.as_nanos()))
    }
}

impl Add<NsDuration> for NsDuration {
    type Output = Self;
    fn add(self, rhs: NsDuration) -> Self::Output {
        Self(self.0.add(rhs.as_nanos()))
    }
}

impl Add<NsTimestamp> for NsDuration {
    type Output = NsTimestamp;
    fn add(self, rhs: NsTimestamp) -> Self::Output {
        NsTimestamp(self.0.add(rhs.as_nanos()))
    }
}

impl Sub<i128> for NsDuration {
    type Output = Self;
    fn sub(self, rhs: i128) -> Self::Output {
        Self(self.0.sub(rhs))
    }
}

impl Sub<Nanoseconds> for NsDuration {
    type Output = Self;
    fn sub(self, rhs: Nanoseconds) -> Self::Output {
        Self(self.0.sub(rhs.as_nanos()))
    }
}

impl Sub<NsDuration> for NsDuration {
    type Output = Self;
    fn sub(self, rhs: NsDuration) -> Self::Output {
        Self(self.0.sub(rhs.as_nanos()))
    }
}

impl Mul<f64> for NsDuration {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let float = self.0.0 as f64;
        let float_multiplied = float.mul(rhs);
        Self(Nanoseconds(float_multiplied as i128))
    }
}

impl Mul<i128> for NsDuration {
    type Output = Self;
    fn mul(self, rhs: i128) -> Self::Output {
        Self(Nanoseconds(self.as_nanos() * rhs))
    }
}

impl Neg for NsDuration {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(Nanoseconds(-self.0.0))
    }
}

impl From<i128> for NsDuration {
    fn from(value: i128) -> Self {
        Self(Nanoseconds::from(value))
    }
}

impl TryFrom<u128> for NsDuration {
    type Error = Error;
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Nanoseconds::try_from(value).map(Self)
    }
}
impl From<Duration> for NsDuration {
    fn from(value: Duration) -> Self {
        Self(Nanoseconds::from(value))
    }
}

impl TryFrom<NsDuration> for Duration {
    type Error = Error;
    fn try_from(value: NsDuration) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl From<Nanoseconds> for NsDuration {
    fn from(value: Nanoseconds) -> Self {
        Self(value)
    }
}

impl Serialize for NsDuration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_serializable().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NsDuration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // if let Ok(a) = deserializer.deserialize_i128(NanosecondsVisitor) {
        //     return Ok(NsDuration(a));
        // }
        Nanoseconds::deserialize(deserializer).map(Self)
    }
}

/// Timestamp expressed as nanoseconds since [`NsTimestamp::UNIX_EPOCH`], or a duration expressed in nanoseconds.
///
/// This type uses [`i128`] internally - it allows for both negative and positive numbers.
/// However, most of Rust's timestamp structures seem to be incapable of storing timestamps before [`UNIX_EPOCH`],
/// so many of the conversion methods between this type and Rust's types may fail.
///
/// # Serialization and deserialization
/// This type is serialized as [`i128`] when used with serde.
///
/// This type can be deserialized from any integer value, although if the integer is larger than [`i128::MAX`] then the conversion will fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nanoseconds(i128);

impl Nanoseconds {
    pub const ZERO: Nanoseconds = Nanoseconds(0);
    pub const MIN: Nanoseconds = Nanoseconds(i128::MIN);
    pub const MAX: Nanoseconds = Nanoseconds(i128::MAX);

    /// Create a new timestamp based on [`SystemTime::now`].
    ///
    /// # Example
    /// ```
    /// use std::time::{SystemTime, Duration};
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// let now_system_time = SystemTime::now();
    /// let now_ns = Nanoseconds::now();
    /// let now_ns_as_system_time: SystemTime = now_ns.try_into().unwrap();
    /// let difference = now_ns_as_system_time.duration_since(now_system_time).unwrap();
    /// assert!(difference < Duration::from_secs(1));
    /// ```
    pub fn now() -> Self {
        SystemTime::now().into()
    }

    /// Get the amount of seconds since [`UNIX_EPOCH`].
    ///
    /// This uses [`i128::div_euclid`] to divide the number of nanoseconds by `1_000_000_000i128`, which means it will always round down, towards `-Infinity`.
    /// It doesn't use the default dividing method, which rounds towards zero, because that would mean that the "zeroth" second is twice as long as all other ones.
    ///
    /// In short, this means that this function gives you the index of the second that *has already passed or is currently passing*,
    /// and will never give you a second that is in the future.
    ///
    /// # Example values
    /// | Nanosecond range                 | `.as_secs()` result |
    /// | -------------------------------- | ------------------- |
    /// | `-3_000_000_000..-2_000_000_001` |  `-3`               |
    /// | `-2_000_000_000..-1_000_000_001` |  `-2`               |
    /// | `-1_000_000_000..-1`             |  `-1`               |
    /// | `0..999_999_999`                 |  `0`                |
    /// | `1_000_000_000..1_999_999_999`   |  `1`                |
    /// | `2_000_000_000..2_999_999_999`   |  `2`                |
    /// | `3_000_000_000..3_999_999_999`   |  `3`                |
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_nanos(-3_000_000_000).as_secs(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_999_999_999).as_secs(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_000_000_001).as_secs(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_000_000_000).as_secs(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_999_999_999).as_secs(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_000_000_001).as_secs(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_000_000_000).as_secs(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(-999_999_999).as_secs(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(-1).as_secs(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(0).as_secs(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(1).as_secs(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(999_999_999).as_secs(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(1_000_000_000).as_secs(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(1_000_000_001).as_secs(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(1_999_999_999).as_secs(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(2_000_000_000).as_secs(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(2_000_000_001).as_secs(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(2_999_999_999).as_secs(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(3_000_000_000).as_secs(), 3);
    /// ```
    pub fn as_secs(self) -> i128 {
        self.0.div_euclid(1_000_000_000i128)
    }

    pub fn frac(self) -> u32 {
        self.0.rem_euclid(1_000_000_000i128) as u32
    }

    /// Get the amount of milliseconds since [`UNIX_EPOCH`].
    ///
    /// This uses [`i128::div_euclid`] to divide the number of nanoseconds by `1_000_000i128`, which means it will always round down, towards `-Infinity`.
    /// It doesn't use the default dividing method, which rounds towards zero, because that would mean that the "zeroth" millisecond is twice as long as all other ones.
    ///
    /// In short, this means that this function gives you the index of the millisecond that *has already passed or is currently passing*,
    /// and will never give you a millisecond that is in the future.
    ///
    /// # Example values
    /// | Nanosecond range         | `.as_millis()` result |
    /// | -------------------------| --------------------- |
    /// | `-3_000_000..-2_000_001` |  `-3`                 |
    /// | `-2_000_000..-1_000_001` |  `-2`                 |
    /// | `-1_000_000..-1`         |  `-1`                 |
    /// | `0..999_999`             |  `0`                  |
    /// | `1_000_000..1_999_999`   |  `1`                  |
    /// | `2_000_000..2_999_999`   |  `2`                  |
    /// | `3_000_000..3_999_999`   |  `3`                  |
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_nanos(-3_000_000).as_millis(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_999_999).as_millis(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_000_001).as_millis(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_000_000).as_millis(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_999_999).as_millis(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_000_001).as_millis(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_000_000).as_millis(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(-999_999).as_millis(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(-1).as_millis(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(0).as_millis(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(1).as_millis(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(999_999).as_millis(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(1_000_000).as_millis(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(1_000_001).as_millis(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(1_999_999).as_millis(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(2_000_000).as_millis(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(2_000_001).as_millis(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(2_999_999).as_millis(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(3_000_000).as_millis(), 3);
    /// ```
    pub fn as_millis(self) -> i128 {
        self.0.div_euclid(1_000_000i128)
    }

    /// Get the amount of microseconds since [`UNIX_EPOCH`].
    ///
    /// This uses [`i128::div_euclid`] to divide the number of nanoseconds by `1_000i128`, which means it will always round down, towards `-Infinity`.
    /// It doesn't use the default dividing method, which rounds towards zero, because that would mean that the "zeroth" microsecond is twice as long as all other ones.
    ///
    /// In short, this means that this function gives you the index of the microsecond that *has already passed or is currently passing*,
    /// and will never give you a microsecond that is in the future.
    ///
    /// # Example values
    /// | Nanosecond range | `.as_micros()` result |
    /// | -----------------| --------------------- |
    /// | `-3_000..-2_001` |  `-3`                 |
    /// | `-2_000..-1_001` |  `-2`                 |
    /// | `-1_000..-1`     |  `-1`                 |
    /// | `0..999`         |  `0`                  |
    /// | `1_000..1_999`   |  `1`                  |
    /// | `2_000..2_999`   |  `2`                  |
    /// | `3_000..3_999`   |  `3`                  |
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_nanos(-3_000).as_micros(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_999).as_micros(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_001).as_micros(), -3);
    /// assert_eq!(Nanoseconds::from_nanos(-2_000).as_micros(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_999).as_micros(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_001).as_micros(), -2);
    /// assert_eq!(Nanoseconds::from_nanos(-1_000).as_micros(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(-999).as_micros(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(-1).as_micros(), -1);
    /// assert_eq!(Nanoseconds::from_nanos(0).as_micros(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(1).as_micros(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(999).as_micros(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(1_000).as_micros(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(1_001).as_micros(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(1_999).as_micros(), 1);
    /// assert_eq!(Nanoseconds::from_nanos(2_000).as_micros(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(2_001).as_micros(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(2_999).as_micros(), 2);
    /// assert_eq!(Nanoseconds::from_nanos(3_000).as_micros(), 3);
    /// ```
    pub fn as_micros(self) -> i128 {
        self.0.div_euclid(1_000i128)
    }

    /// Get the amount of nanoseconds since [`UNIX_EPOCH`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_nanos(1234).as_nanos(), 1234);
    /// assert_eq!(Nanoseconds::from_nanos(0).as_nanos(), 0);
    /// assert_eq!(Nanoseconds::from_nanos(-1234).as_nanos(), -1234);
    /// ```
    pub fn as_nanos(self) -> i128 {
        self.0
    }

    /// Create [`Nanoseconds`] from the amount of seconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given second.
    /// For example, for the "zeroth" second, the resulting timestamp is `0` nanoseconds,
    /// for the "first" second, the resulting timestamp is `1_000_000_000` nanoseconds,
    /// for the "negative first" (-1st) second, the resulting timestamp is `-1_000_000_000` nanoseconds.
    ///
    /// Since this function takes the number of seconds as an [`i64`], this function will never fail,
    /// as the result of the multiplication always fits within a [`i128`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_secs(1234).as_nanos(), 1_234_000_000_000);
    /// assert_eq!(Nanoseconds::from_secs(0).as_nanos(), 0);
    /// assert_eq!(Nanoseconds::from_secs(-1234).as_nanos(), -1_234_000_000_000);
    /// ```
    pub fn from_secs(secs: i64) -> Self {
        Self((secs as i128) * 1_000_000_000i128)
    }

    /// Create [`Nanoseconds`] from the amount of seconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given second.
    /// For example, for the "zeroth" second, the resulting timestamp is `0` nanoseconds,
    /// for the "first" second, the resulting timestamp is `1_000_000_000` nanoseconds,
    /// for the "negative first" (-1st) second, the resulting timestamp is `-1_000_000_000` nanoseconds.
    ///
    /// This function takes the number of seconds as an [`f64`] and it accepts fractions of seconds.
    /// For very large values, the precision of [`f64`] may not be enough to store the timestamp accurately.
    /// In that case, it is recommended to use the [`Nanoseconds::from_secs`] or [`Nanoseconds::from_millis`]
    /// or similar functions instead.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_secs_f64(1.0).as_nanos(), 1_000_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(1.1).as_nanos(), 1_100_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(1.25).as_nanos(), 1_250_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(1.5).as_nanos(), 1_500_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(1.75).as_nanos(), 1_750_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(-1.0).as_nanos(), -1_000_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(-1.25).as_nanos(), -1_250_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(-1.5).as_nanos(), -1_500_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(-1.75).as_nanos(), -1_750_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(0.0).as_nanos(), 0);
    /// assert_eq!(Nanoseconds::from_secs_f64(1_000_000_000.0).as_nanos(), 1_000_000_000_000_000_000);
    /// assert_eq!(Nanoseconds::from_secs_f64(1_000_000_000.1).as_nanos(), 1_000_000_000_100_000_023); // Accuracy loss at large values
    /// ```
    pub fn from_secs_f64(secs: f64) -> Self {
        let seconds = (secs.floor() as i128) * 1_000_000_000;
        let frac = (secs.rem_euclid(1.0) * 1_000_000_000f64) as i128;
        Self(seconds + frac)
    }

    /// Try to create [`Nanoseconds`] from the amount of seconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given second.
    /// For example, for the "zeroth" second, the resulting timestamp is `0` nanoseconds,
    /// for the "first" second, the resulting timestamp is `1_000_000_000` nanoseconds,
    /// for the "negative first" (-1st) second, the resulting timestamp is `-1_000_000_000` nanoseconds.
    ///
    /// # Errors
    /// This function will return an [`Error::OutOfRange`], if the result of multiplying `secs` by `1_000_000_000` overflows [`i128`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    ///
    /// assert_eq!(Nanoseconds::try_from_secs(1234).unwrap().as_nanos(), 1_234_000_000_000);
    /// assert_eq!(Nanoseconds::try_from_secs(0).unwrap().as_nanos(), 0);
    /// assert_eq!(Nanoseconds::try_from_secs(-1234).unwrap().as_nanos(), -1_234_000_000_000);
    /// assert!(matches!(Nanoseconds::try_from_secs(i128::MAX), Err(Error::OutOfRange)));
    /// ```
    pub fn try_from_secs(secs: i128) -> Result<Self, Error> {
        Ok(Self(secs.checked_mul(1_000_000_000i128).ok_or(Error::OutOfRange)?))
    }

    /// Create [`Nanoseconds`] from the amount of milliseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given millisecond.
    /// For example, for the "zeroth" millisecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" millisecond, the resulting timestamp is `1_000_000` nanoseconds,
    /// for the "negative first" (-1st) millisecond, the resulting timestamp is `-1_000_000` nanoseconds.
    ///
    /// Since this function takes the number of milliseconds as an [`i64`], this function will never fail,
    /// as the result of the multiplication always fits within a [`i128`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_millis(1234).as_nanos(), 1_234_000_000);
    /// assert_eq!(Nanoseconds::from_millis(0).as_nanos(), 0);
    /// assert_eq!(Nanoseconds::from_millis(-1234).as_nanos(), -1_234_000_000);
    /// ```
    pub fn from_millis(millis: i64) -> Self {
        Self((millis as i128) * 1_000_000i128)
    }

    /// Try to create [`Nanoseconds`] from the amount of milliseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given millisecond.
    /// For example, for the "zeroth" millisecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" millisecond, the resulting timestamp is `1_000_000` nanoseconds,
    /// for the "negative first" (-1st) millisecond, the resulting timestamp is `-1_000_000` nanoseconds.
    ///
    /// # Errors
    /// This function will return an [`Error::OutOfRange`], if the result of multiplying `millis` by `1_000_000` overflows [`i128`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    ///
    /// assert_eq!(Nanoseconds::try_from_millis(1234).unwrap().as_nanos(), 1_234_000_000);
    /// assert_eq!(Nanoseconds::try_from_millis(0).unwrap().as_nanos(), 0);
    /// assert_eq!(Nanoseconds::try_from_millis(-1234).unwrap().as_nanos(), -1_234_000_000);
    /// assert!(matches!(Nanoseconds::try_from_millis(i128::MAX), Err(Error::OutOfRange)));
    /// ```
    pub fn try_from_millis(millis: i128) -> Result<Self, Error> {
        Ok(Self(millis.checked_mul(1_000_000i128).ok_or(Error::OutOfRange)?))
    }

    /// Create [`Nanoseconds`] from the amount of microseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given microsecond.
    /// For example, for the "zeroth" microsecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" microsecond, the resulting timestamp is `1_000` nanoseconds,
    /// for the "negative first" (-1st) microsecond, the resulting timestamp is `-1_000` nanoseconds.
    ///
    /// Since this function takes the number of microseconds as an [`i64`], this function will never fail,
    /// as the result of the multiplication always fits within a [`i128`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// assert_eq!(Nanoseconds::from_micros(1234).as_nanos(), 1_234_000);
    /// assert_eq!(Nanoseconds::from_micros(0).as_nanos(), 0);
    /// assert_eq!(Nanoseconds::from_micros(-1234).as_nanos(), -1_234_000);
    /// ```
    pub fn from_micros(micros: i64) -> Self {
        Self((micros as i128) * 1_000i128)
    }

    /// Try to create [`Nanoseconds`] from the amount of microseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given microsecond.
    /// For example, for the "zeroth" microsecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" microsecond, the resulting timestamp is `1_000` nanoseconds,
    /// for the "negative first" (-1st) microsecond, the resulting timestamp is `-1_000` nanoseconds.
    ///
    /// # Errors
    /// This function will return an [`Error::OutOfRange`], if the result of multiplying `micros` by `1_000` overflows [`i128`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    ///
    /// assert_eq!(Nanoseconds::try_from_micros(1234).unwrap().as_nanos(), 1_234_000);
    /// assert_eq!(Nanoseconds::try_from_micros(0).unwrap().as_nanos(), 0);
    /// assert_eq!(Nanoseconds::try_from_micros(-1234).unwrap().as_nanos(), -1_234_000);
    /// assert!(matches!(Nanoseconds::try_from_micros(i128::MAX), Err(Error::OutOfRange)));
    /// ```
    pub fn try_from_micros(micros: i128) -> Result<Self, Error> {
        Ok(Self(micros.checked_mul(1_000i128).ok_or(Error::OutOfRange)?))
    }

    /// Create [`Nanoseconds`] from the amount of nanoseconds since [`UNIX_EPOCH`].
    pub fn from_nanos(nanos: i128) -> Self {
        Self(nanos)
    }

    /// Convert to a RFC3339 date time string in the UTC timezone.
    ///
    /// The function uses [`DateTime::to_rfc3339_opts`] to perform the conversion, with [`SecondsFormat::Nanos`] and the `use_z` flag set.
    ///
    /// # Example
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// let ns_timestamp = Nanoseconds::from_nanos(1_234_567_890_123_456_789);
    /// assert_eq!(&ns_timestamp.to_date_time_string_utc(), "2009-02-13T23:31:30.123456789Z");
    /// ```
    pub fn to_date_time_string_utc(self) -> String {
        let date_time: DateTime<Utc> = self.try_into().unwrap();
        date_time.to_rfc3339_opts(SecondsFormat::Nanos, true)
    }

    /// Convert to a RFC3339 date time string in the local timezone.
    ///
    /// The function uses [`DateTime::to_rfc3339_opts`] to perform the conversion, with [`SecondsFormat::Nanos`] and the `use_z` flag not set.
    ///
    /// # Example
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    /// use chrono::{DateTime, Local, SecondsFormat};
    ///
    /// let ns_timestamp = Nanoseconds::from_nanos(1_234_567_890_123_456_789);
    /// let date_time_local: DateTime<Local> = ns_timestamp.try_into().unwrap();
    /// assert_eq!(ns_timestamp.to_date_time_string_local(), date_time_local.to_rfc3339_opts(SecondsFormat::Nanos, false));
    ///
    /// let ns_timestamp = Nanoseconds::from_nanos(-1_234_567_890_123_456_789);
    /// let date_time_local: DateTime<Local> = ns_timestamp.try_into().unwrap();
    /// assert_eq!(ns_timestamp.to_date_time_string_local(), date_time_local.to_rfc3339_opts(SecondsFormat::Nanos, false));
    /// ```
    pub fn to_date_time_string_local(self) -> String {
        let date_time: DateTime<Local> = self.try_into().unwrap();
        date_time.to_rfc3339_opts(SecondsFormat::Nanos, false)
    }

    /// Returns the timestamp opposite to the provided origin.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// let origin = Nanoseconds::from_secs(5);
    /// let two_seconds_later = origin + 2_000_000_000;
    /// let two_seconds_earlier = two_seconds_later.invert_with_origin(origin);
    /// assert_eq!(two_seconds_earlier, Nanoseconds::from_secs(3));
    ///
    /// let twelfth_second_since_epoch = Nanoseconds::from_secs(12);
    /// let twelfth_second_before_epoch = twelfth_second_since_epoch.invert_with_origin(Nanoseconds::ZERO);
    /// assert_eq!(twelfth_second_before_epoch, Nanoseconds::from_secs(-12));
    /// ```
    pub fn invert_with_origin(self, origin: Self) -> Self {
        let duration_since_origin = self.0 - origin.0;
        Self(origin.0 - duration_since_origin)
    }
}

impl fmt::Display for Nanoseconds {
    /// Display [`Nanoseconds`] as a UTC datetime string, and the amount of nanoseconds since [`UNIX_EPOCH`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.to_date_time_string_local(), self.0)
    }
}

impl Add<i128> for Nanoseconds {
    type Output = Self;

    /// Move the timestamp into the future by `rhs` nanoseconds.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    /// use std::ops::Add;
    ///
    /// let initial_timestamp = Nanoseconds::from_secs(3);
    /// assert_eq!(initial_timestamp.add(1_500_100_900), Nanoseconds::from_nanos(4_500_100_900));
    /// assert_eq!(initial_timestamp.add(1), Nanoseconds::from_nanos(3_000_000_001));
    /// assert_eq!(initial_timestamp.add(-10_000_000_000), Nanoseconds::from_secs(-7));
    /// ```
    ///
    /// Attempting to add with overflow will cause a panic:
    /// ```should_panic
    /// # use scoretracker::util::timestamp::Nanoseconds;
    /// # use std::ops::Add;
    /// Nanoseconds::MAX.add(1);
    /// ```
    fn add(self, rhs: i128) -> Self::Output {
        Self(self.0.add(rhs))
    }
}

impl Sub<i128> for Nanoseconds {
    type Output = Self;

    /// Move the timestamp into the past by `rhs` nanoseconds.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    /// use std::ops::Sub;
    ///
    /// let initial_timestamp = Nanoseconds::from_secs(3);
    /// assert_eq!(initial_timestamp.sub(1_500_100_900), Nanoseconds::from_nanos(1_499_899_100));
    /// assert_eq!(initial_timestamp.sub(1), Nanoseconds::from_nanos(2_999_999_999));
    /// assert_eq!(initial_timestamp.sub(-10_000_000_000), Nanoseconds::from_secs(13));
    /// ```
    ///
    /// Attempting to subtract with overflow will cause a panic:
    /// ```should_panic
    /// # use scoretracker::util::timestamp::Nanoseconds;
    /// # use std::ops::Sub;
    /// Nanoseconds::MIN.sub(1);
    /// ```
    fn sub(self, rhs: i128) -> Self::Output {
        Self(self.0.sub(rhs))
    }
}

impl Sub for Nanoseconds {
    type Output = i128;

    /// Calculate the amount of microseconds that has passed since `rhs`.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    /// use std::ops::Sub;
    ///
    /// assert_eq!(Nanoseconds::from_secs(3).sub(Nanoseconds::ZERO), 3_000_000_000);
    /// assert_eq!(Nanoseconds::from_secs(10).sub(Nanoseconds::from_secs(3)), 7_000_000_000);
    /// assert_eq!(Nanoseconds::from_secs(-123).sub(Nanoseconds::from_secs(-113)), -10_000_000_000);
    /// ```
    ///
    /// Attempting to subtract with overflow will cause a panic:
    /// ```should_panic
    /// # use scoretracker::util::timestamp::Nanoseconds;
    /// # use std::ops::Sub;
    /// Nanoseconds::from_secs(100).sub(Nanoseconds::MIN);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        self.0.sub(rhs.0)
    }
}

impl From<i128> for Nanoseconds {
    /// Create [`Nanoseconds`] from the amount of nanoseconds since [`UNIX_EPOCH`].
    ///
    /// # Example
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    ///
    /// let nanoseconds = 1_234_567_890;
    /// let timestamp = Nanoseconds::from(nanoseconds);
    /// assert_eq!(timestamp.as_nanos(), 1_234_567_890);
    /// ```
    fn from(value: i128) -> Self {
        Nanoseconds(value)
    }
}

impl TryFrom<u128> for Nanoseconds {
    type Error = Error;

    /// Try to convert a [`u128`] into [`Nanoseconds`].
    ///
    /// # Errors
    /// This function will return [`Error::OutOfRange`] if the the duration of time since the [`UNIX_EPOCH`] in nanoseconds is larger than [`i128::MAX`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    ///
    /// let nanoseconds: u128 = 1_234_567_890;
    /// let timestamp = Nanoseconds::try_from(nanoseconds).unwrap();
    /// assert_eq!(timestamp.as_nanos(), 1_234_567_890);
    ///
    /// let nanoseconds_out_of_range: u128 = i128::MAX as u128 + 300;
    /// let timestamp = Nanoseconds::try_from(nanoseconds_out_of_range);
    /// assert!(matches!(timestamp, Err(Error::OutOfRange)))
    /// ```
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let signed: i128 = value.try_into().ok().ok_or(Error::OutOfRange)?;
        Ok(signed.into())
    }
}

impl From<Duration> for Nanoseconds {
    /// Convert a [`Duration`] into [`Nanoseconds`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    /// use std::time::Duration;
    ///
    /// let duration = Duration::ZERO;
    /// assert_eq!(Nanoseconds::from(duration), Nanoseconds::ZERO);
    ///
    /// let duration = Duration::from_millis(3);
    /// assert_eq!(Nanoseconds::from(duration), Nanoseconds::from_nanos(3_000_000));
    ///
    /// let duration = Duration::from_secs_f32(6.25);
    /// assert_eq!(Nanoseconds::from(duration), Nanoseconds::from_nanos(6_250_000_000));
    /// ```
    fn from(value: Duration) -> Self {
        value.as_nanos().try_into().unwrap() // this should never fail because as_nanos should never return a value bigger than i128::MAX
    }
}

impl From<SystemTime> for Nanoseconds {
    /// Convert a [`SystemTime`] into [`Nanoseconds`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    /// use std::time::{SystemTime, UNIX_EPOCH, Duration};
    ///
    /// let system_time = SystemTime::now();
    /// let ns_timestamp = Nanoseconds::from(system_time);
    /// let duration = system_time.duration_since(UNIX_EPOCH).unwrap();
    /// assert_eq!(duration.as_nanos() as i128, ns_timestamp.as_nanos());
    ///
    /// let system_time = UNIX_EPOCH;
    /// let ns_timestamp = Nanoseconds::from(system_time);
    /// let duration = system_time.duration_since(UNIX_EPOCH).unwrap();
    /// assert_eq!(duration.as_nanos() as i128, ns_timestamp.as_nanos());
    /// assert_eq!(0, ns_timestamp.as_nanos());
    ///
    /// let system_time = UNIX_EPOCH - Duration::from_secs(5);
    /// let ns_timestamp = Nanoseconds::from(system_time);
    /// let negative_duration = UNIX_EPOCH.duration_since(system_time).unwrap();
    /// assert_eq!(-(negative_duration.as_nanos() as i128), ns_timestamp.as_nanos());
    /// ```
    fn from(value: SystemTime) -> Self {
        match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                // Provided SystemTime is later than or equal to UNIX_EPOCH
                duration.into()
            }
            Err(e) => {
                // Provided SystemTime is earlier than UNIX_EPOCH
                let negative_duration = e.duration();
                Nanoseconds::from(negative_duration).invert_with_origin(Nanoseconds::ZERO)
            }
        }
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for Nanoseconds {
    /// Convert a [`DateTime<Tz>`] into [`Nanoseconds`].
    ///
    /// # Example
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    /// use chrono::{Utc, DateTime, NaiveDate};
    ///
    /// let date_time = NaiveDate::from_ymd_opt(1970, 1, 1)
    ///     .unwrap()
    ///     .and_hms_milli_opt(0, 0, 1, 444)
    ///     .unwrap()
    ///     .and_local_timezone(Utc)
    ///     .unwrap();
    /// let timestamp = Nanoseconds::from(date_time);
    /// assert_eq!(timestamp, Nanoseconds::from_millis(1_444))
    /// ```
    fn from(value: DateTime<Tz>) -> Self {
        let system_time = SystemTime::from(value);
        system_time.into()
    }
}

impl TryFrom<Nanoseconds> for Duration {
    type Error = Error;

    /// Try to convert [`Nanoseconds`] into a [`Duration`].
    ///
    /// # Errors
    /// This function will return [`Error::OutOfDurationRange`] if the value is out of range of [`Duration`].
    /// This happens in two cases:
    /// - When the amount of nanoseconds in [`self`] is negative.
    /// - When the amount of seconds in [`self`] is greater than [`u64::MAX`].
    ///     - The amount of nanoseconds in [`self`] is greater than `u64::MAX * 1_000_000_000 + 999_999_999`.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    /// use std::time::Duration;
    ///
    /// let timestamp = Nanoseconds::now();
    /// let duration = Duration::try_from(timestamp).unwrap();
    /// assert_eq!(duration, Duration::from_nanos(timestamp.as_nanos() as u64));
    ///
    /// let timestamp = Nanoseconds::from_nanos(-1);
    /// let duration = Duration::try_from(timestamp);
    /// assert!(matches!(duration, Err(Error::OutOfDurationRange)));
    ///
    /// let u64_max_seconds = Nanoseconds::try_from_secs(u64::MAX as i128).unwrap();
    /// let duration = Duration::try_from(u64_max_seconds).unwrap();
    /// assert_eq!(duration, Duration::new(u64::MAX, 0));
    ///
    /// let timestamp = u64_max_seconds + 1;
    /// let duration = Duration::try_from(timestamp).unwrap();
    /// assert_eq!(duration, Duration::new(u64::MAX, 1));
    ///
    /// let timestamp = u64_max_seconds + 999_999_999;
    /// let duration = Duration::try_from(timestamp).unwrap();
    /// assert_eq!(duration, Duration::MAX);
    ///
    /// let timestamp = u64_max_seconds + 1_000_000_000;
    /// let duration = Duration::try_from(timestamp);
    /// assert!(matches!(duration, Err(Error::OutOfDurationRange)));
    /// ```
    fn try_from(value: Nanoseconds) -> Result<Self, Self::Error> {
        let nanos = value.0.rem_euclid(1_000_000_000i128) as u32; // this never fails
        let secs = value
            .0
            .div_euclid(1_000_000_000i128)
            .try_into()
            .ok()
            .ok_or(Error::OutOfDurationRange)?;
        let duration = Duration::new(secs, nanos);
        Ok(duration)
    }
}

impl TryFrom<Nanoseconds> for (bool, Duration) {
    type Error = Error;

    /// Try to convert [`Nanoseconds`] into a ([`bool`], [`Duration`]) tuple - a duration of time that has passed since [`UNIX_EPOCH`].
    ///
    /// The boolean in the returned tuple indicates whether the duration is negative.
    /// A negative duration indicates that the initial timestamp points to a time before [`UNIX_EPOCH`].
    ///
    /// # Errors
    /// This function will return [`Error::OutOfDurationRange`] if the value is out of range of [`Duration`].
    /// This happens in two cases:
    /// - When the amount of seconds in [`self`] is greater than [`u64::MAX`].
    ///     - The amount of nanoseconds in [`self`] is greater than `u64::MAX * 1_000_000_000 + 999_999_999`.
    /// - When the amount of seconds in [`self`] is less than `-u64::MAX`.
    ///     - The amount of nanoseconds in [`self`] is less than `-u64::MAX * 1_000_000_000 - 999_999_999`.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    /// use std::time::{SystemTime, Duration, UNIX_EPOCH};
    ///
    /// let timestamp = Nanoseconds::from_nanos(1);
    /// let (negative, duration) = timestamp.try_into().unwrap();
    /// assert_eq!(negative, false);
    /// assert_eq!(duration, Duration::from_nanos(1));
    ///
    /// let timestamp = Nanoseconds::from_nanos(-3);
    /// let (negative, duration) = timestamp.try_into().unwrap();
    /// assert_eq!(negative, true);
    /// assert_eq!(duration, Duration::from_nanos(3));
    ///
    /// let timestamp = Nanoseconds::from_nanos(-6_123_456_789);
    /// let (negative, duration) = timestamp.try_into().unwrap();
    /// assert_eq!(negative, true);
    /// assert_eq!(duration, Duration::from_nanos(6_123_456_789));
    ///
    /// let timestamp = Nanoseconds::ZERO;
    /// let (negative, duration) = timestamp.try_into().unwrap();
    /// assert_eq!(negative, false);
    /// assert_eq!(duration, Duration::ZERO);
    ///
    /// let u64_max_seconds = Nanoseconds::try_from_secs(u64::MAX as i128).unwrap();
    /// let (negative, duration) = u64_max_seconds.try_into().unwrap();
    /// assert_eq!(negative, false);
    /// assert_eq!(duration, Duration::new(u64::MAX, 0));
    ///
    /// let timestamp = u64_max_seconds + 999_999_999;
    /// let (negative, duration) = timestamp.try_into().unwrap();
    /// assert_eq!(negative, false);
    /// assert_eq!(duration, Duration::MAX);
    ///
    /// let timestamp = u64_max_seconds + 1_000_000_000;
    /// let result: Result<(bool, Duration), _> = timestamp.try_into();
    /// assert!(matches!(result, Err(Error::OutOfDurationRange)));
    ///
    /// let negative_u64_max_seconds = Nanoseconds::try_from_secs(-(u64::MAX as i128)).unwrap();
    /// let (negative, duration) = negative_u64_max_seconds.try_into().unwrap();
    /// assert_eq!(negative, true);
    /// assert_eq!(duration, Duration::new(u64::MAX, 0));
    ///
    /// let timestamp = negative_u64_max_seconds - 999_999_999;
    /// let (negative, duration) = timestamp.try_into().unwrap();
    /// assert_eq!(negative, true);
    /// assert_eq!(duration, Duration::MAX);
    ///
    /// let timestamp = negative_u64_max_seconds - 1_000_000_000;
    /// let result: Result<(bool, Duration), _> = timestamp.try_into();
    /// assert!(matches!(result, Err(Error::OutOfDurationRange)));
    /// ```
    fn try_from(value: Nanoseconds) -> Result<Self, Self::Error> {
        let negative = value.0.is_negative();
        let nanos = value.0.rem(1_000_000_000i128).unsigned_abs() as u32; // this never fails
        let secs = value.0.div(1_000_000_000i128).unsigned_abs();

        let secs = secs.try_into().ok().ok_or(Error::OutOfDurationRange)?;
        let duration = Duration::new(secs, nanos);
        Ok((negative, duration))
    }
}

impl TryFrom<Nanoseconds> for SystemTime {
    type Error = Error;

    /// Try to convert [`Nanoseconds`] into a [`SystemTime`].
    ///
    /// # Errors
    /// This function will return [`Error::OutOfSystemTimeRange`] if the value is out of range of [`SystemTime`].
    /// This happens in two cases:
    /// - When the amount of seconds in the intermediate [`Duration`] is greater than [`i64::MAX`].
    ///     - The amount of nanoseconds in [`self`] is greater than `i64::MAX * 1_000_000_000 + 999_999_999`.
    /// - When the amount of seconds in the intermediate [`Duration`] is less than [`i64::MIN`].
    ///     - The amount of nanoseconds in [`self`] is less than `i64::MIN * 1_000_000_000`.
    ///
    /// This function will return [`Error::OutOfDurationRange`] if the intermediate [`Duration`] value is out of range.
    /// This happens in two cases:
    /// - When the amount of seconds in [`self`] is greater than [`u64::MAX`].
    ///     - The amount of nanoseconds in [`self`] is greater than `u64::MAX * 1_000_000_000 + 999_999_999`.
    /// - When the amount of seconds in [`self`] is less than `-u64::MAX`.
    ///     - The amount of nanoseconds in [`self`] is less than `-u64::MAX * 1_000_000_000 - 999_999_999`.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::timestamp::{Nanoseconds, Error};
    /// use std::time::{SystemTime, UNIX_EPOCH, Duration};
    ///
    /// let timestamp = Nanoseconds::from_nanos(1);
    /// let system_time: SystemTime = timestamp.try_into().unwrap();
    /// assert_eq!(system_time.duration_since(UNIX_EPOCH).unwrap().as_nanos(), 1);
    ///
    /// let timestamp = Nanoseconds::from_nanos(-3);
    /// let system_time: SystemTime = timestamp.try_into().unwrap();
    /// assert_eq!(UNIX_EPOCH.duration_since(system_time).unwrap().as_nanos(), 3);
    ///
    /// let timestamp = Nanoseconds::ZERO;
    /// let system_time: SystemTime = timestamp.try_into().unwrap();
    /// assert_eq!(system_time, UNIX_EPOCH);
    ///
    /// let timestamp = Nanoseconds::from_secs(i64::MAX) + 999_999_999;
    /// let system_time: SystemTime = timestamp.try_into().unwrap();
    /// assert_eq!(system_time, SystemTime::UNIX_EPOCH + Duration::new(i64::MAX as u64, 999_999_999));
    ///
    /// let timestamp = Nanoseconds::from_secs(i64::MAX) + 1_000_000_000;
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Err(Error::OutOfSystemTimeRange)));
    ///
    /// let u64_max_seconds = Nanoseconds::try_from_secs(u64::MAX as i128).unwrap();
    ///
    /// let timestamp = u64_max_seconds + 999_999_999;
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Err(Error::OutOfSystemTimeRange)));
    ///
    /// let timestamp = u64_max_seconds + 1_000_000_000;
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Err(Error::OutOfDurationRange)));
    ///
    /// let i64_min_seconds = Nanoseconds::from_secs(i64::MIN);
    /// let system_time: SystemTime = i64_min_seconds.try_into().unwrap();
    /// assert_eq!(system_time, SystemTime::UNIX_EPOCH - Duration::new(i64::MIN.unsigned_abs(), 0));
    ///
    /// let timestamp = i64_min_seconds - 1;
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Err(Error::OutOfSystemTimeRange)));
    ///
    /// let negative_u64_max_seconds = Nanoseconds::try_from_secs(-(u64::MAX as i128)).unwrap();
    ///
    /// let timestamp = negative_u64_max_seconds - 999_999_999;
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Err(Error::OutOfSystemTimeRange)));
    ///
    /// let timestamp = negative_u64_max_seconds - 1_000_000_000;
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Err(Error::OutOfDurationRange)));
    /// ```
    fn try_from(value: Nanoseconds) -> Result<Self, Self::Error> {
        let (negative, duration) = value.try_into()?;
        let system_time = if negative {
            SystemTime::UNIX_EPOCH.checked_sub(duration)
        } else {
            SystemTime::UNIX_EPOCH.checked_add(duration)
        }
        .ok_or(Error::OutOfSystemTimeRange)?;
        Ok(system_time)
    }
}

impl<Tz: TimeZone> TryFrom<Nanoseconds> for DateTime<Tz>
where
    DateTime<Tz>: From<SystemTime>,
{
    /// Try to convert [`Nanoseconds`] into a [`DateTime<Tz>`].
    ///
    /// # Errors
    /// This function uses [`SystemTime`] under the hood,
    /// check out the documentation for [converting SystemTime into Nanoseconds](#impl-TryFrom<Nanoseconds>-for-SystemTime)
    /// for more information about the returned errors.
    ///
    /// # Example
    /// ```
    /// use scoretracker::util::timestamp::Nanoseconds;
    /// use chrono::{Utc, DateTime, NaiveDate};
    ///
    /// let date_time_utc: DateTime<Utc> = Nanoseconds::from_millis(1_444).try_into().unwrap();
    /// assert_eq!(date_time_utc, NaiveDate::from_ymd_opt(1970, 1, 1)
    ///     .unwrap()
    ///     .and_hms_milli_opt(0, 0, 1, 444)
    ///     .unwrap()
    ///     .and_local_timezone(Utc)
    ///     .unwrap())
    /// ```
    type Error = Error;
    fn try_from(value: Nanoseconds) -> Result<Self, Self::Error> {
        let system_time: SystemTime = value.try_into()?;
        Ok(DateTime::from(system_time))
    }
}

impl Serialize for Nanoseconds {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i128(self.0)
    }
}

struct NanosecondsVisitor;

impl<'de> Visitor<'de> for NanosecondsVisitor {
    type Value = Nanoseconds;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a number of nanoseconds or a struct containing a whole number of seconds and a fractional part")
    }

    fn visit_i8<E: serde::de::Error>(self, v: i8) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i16<E: serde::de::Error>(self, v: i16) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i32<E: serde::de::Error>(self, v: i32) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i128<E: serde::de::Error>(self, v: i128) -> Result<Self::Value, E> {
        Ok(Nanoseconds(v))
    }

    fn visit_u8<E: serde::de::Error>(self, v: u8) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u16<E: serde::de::Error>(self, v: u16) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u128<E: serde::de::Error>(self, v: u128) -> Result<Self::Value, E> {
        Nanoseconds::try_from(v).map_err(|e| E::custom(format!("timestamp out of range: {e}")))
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        SerializableStruct::deserialize(de::value::MapAccessDeserializer::new(map)).map(|x| x.nanos())
    }
}

impl<'de> Deserialize<'de> for Nanoseconds {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NanosecondsVisitor)
    }
}
