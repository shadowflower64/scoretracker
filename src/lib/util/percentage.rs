//! Module for [`Percentage`] struct, for handling percentage values without floating point errors.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Write},
    ops::{Div, Mul},
};

/// Stores a percentage as a [`f64`] value that is usually from 0 - 100.
///
/// This struct stores a fraction as a value scaled by 100 to prevent floating point errors.
///
/// With normal `f64` floating point fractions, some values like `0.29` is actually stored as `0.28999999999999998001598555674718... f64`, which may be undesirable in some situations.
/// For example, multiplying this value by `100.0f64` will result in `28.99999999999999644728632119949907... f64`, not `29.0`, even though `29.0f64` is a perfectly valid `f64` number.
/// This struct stores the fraction `0.29` as `29.0f64` (which can be read intuitively as *29%*), which prevents floating point errors for integer percentage numbers.
///
/// To prevent confusion in ambiguity, you cannot add or subtract percentage values from each other or from other numbers directly.
/// You have to use one of the conversion methods ([`Self::as_multiplier`] or [`Self::as_percentage`]) to convert the struct to f64, which you can then do math with.
///
/// However you are allowed to multiply two percentages together, or multiply a fraction by a percentage, as this is not an ambiguous operation.
///
/// # Examples
/// ```
/// use scoretracker::util::percentage::Percentage;
///
/// for i in 0..=100 {
///     let float = i as f64 / 100.0;
///     let percentage_from_float = float * 100.0;
///     let status = if i as f64 == percentage_from_float { "good" } else if i == percentage_from_float as i32 { "bad" } else { "really bad" };
///
///     let percentage = Percentage::from_percentage(i);
///     let string = format!("i = {}, % = {:.20}, f = {:.20}, p = {:.20} | {}", i, percentage, float, percentage_from_float, status);
///     dbg!(string);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct Percentage(pub f64);

impl Percentage {
    /// Returns a multiplier from the percentage value.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::percentage::Percentage;
    ///
    /// assert_eq!(Percentage(100.0).as_multiplier(), 1.0);
    /// assert_eq!(Percentage(125.0).as_multiplier(), 1.25);
    /// assert_eq!(Percentage(-30.0).as_multiplier(), -0.3);
    /// ```
    pub fn as_multiplier(self) -> f64 {
        self.0 / 100.0
    }
    /// Returns a percentage value.
    ///
    /// # Examples
    /// ```
    /// use scoretracker::util::percentage::Percentage;
    ///
    /// assert_eq!(Percentage(100.0).as_percentage(), 100.0);
    /// assert_eq!(Percentage(125.0).as_percentage(), 125.0);
    /// assert_eq!(Percentage(-30.0).as_percentage(), -30.0);
    /// ```
    pub fn as_percentage(self) -> f64 {
        self.0
    }

    pub fn from_multiplier(value: impl Into<f64>) -> Self {
        Self(value.into() * 100.0)
    }

    pub fn from_percentage(value: impl Into<f64>) -> Self {
        Self(value.into())
    }
}

impl Mul for Percentage {
    type Output = f64;
    fn mul(self, rhs: Self) -> Self::Output {
        self.as_multiplier() * rhs.as_multiplier()
    }
}

impl Mul<f64> for Percentage {
    type Output = f64;
    fn mul(self, rhs: f64) -> Self::Output {
        self.as_multiplier() * rhs
    }
}

impl Div for Percentage {
    type Output = f64;
    fn div(self, rhs: Self) -> Self::Output {
        self.as_multiplier() / rhs.as_multiplier()
    }
}

impl Div<f64> for Percentage {
    type Output = f64;
    fn div(self, rhs: f64) -> Self::Output {
        self.as_multiplier() / rhs
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f).and_then(|_| f.write_char('%'))
    }
}
