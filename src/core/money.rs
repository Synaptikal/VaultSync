//! Monetary value types with proper decimal handling
//!
//! This module provides type-safe monetary operations using `rust_decimal`
//! to avoid IEEE 754 floating-point representation errors.
//!
//! ## Why Not f64?
//!
//! `0.1 + 0.2 != 0.3` in IEEE 754 floats. After thousands of transactions,
//! customer store credits and inventory valuations WILL drift. This is
//! unacceptable for a POS system.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul, Sub};

/// A monetary value with proper decimal handling.
///
/// Internally represented as `rust_decimal::Decimal` which can exactly
/// represent values like $0.01 without floating-point errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(Decimal);

impl Money {
    /// Zero dollars
    pub const ZERO: Money = Money(Decimal::ZERO);

    /// Create from a decimal value
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    /// Create from cents (100 = $1.00)
    pub fn from_cents(cents: i64) -> Self {
        Self(Decimal::new(cents, 2))
    }

    /// Create from a float (ONLY for migration from legacy f64 data)
    ///
    /// # Warning
    /// This should only be used when reading legacy data.
    /// New code should use `from_cents` or `from_str`.
    pub fn from_f64_lossy(value: f64) -> Self {
        Self(Decimal::try_from(value).unwrap_or(Decimal::ZERO))
    }

    /// Convert to f64 (ONLY for legacy API compatibility)
    ///
    /// # Warning
    /// This loses precision. Only use for backward-compatible API responses.
    #[deprecated(note = "Use proper Money serialization instead")]
    pub fn to_f64_lossy(&self) -> f64 {
        use rust_decimal::prelude::ToPrimitive;
        self.0.to_f64().unwrap_or(0.0)
    }

    /// Get the inner Decimal value
    pub fn inner(&self) -> Decimal {
        self.0
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Check if negative
    pub fn is_negative(&self) -> bool {
        self.0.is_sign_negative()
    }

    /// Absolute value
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    /// Round to 2 decimal places (standard currency precision)
    pub fn round_cents(&self) -> Self {
        Self(self.0.round_dp(2))
    }

    /// Apply a percentage (e.g., tax rate of 8.25%)
    pub fn apply_percentage(&self, percent: Decimal) -> Self {
        Self((self.0 * percent / dec!(100)).round_dp(2))
    }
}

impl Default for Money {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.0)
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<i32> for Money {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * Decimal::from(rhs))
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl From<Decimal> for Money {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl From<Money> for Decimal {
    fn from(m: Money) -> Self {
        m.0
    }
}

/// Parse money from string like "12.34" or "$12.34"
impl std::str::FromStr for Money {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cleaned = s.trim().trim_start_matches('$');
        Ok(Self(cleaned.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_precision() {
        // This WOULD fail with f64: 0.1 + 0.2 != 0.3
        let a = Money::from_cents(10); // $0.10
        let b = Money::from_cents(20); // $0.20
        let expected = Money::from_cents(30); // $0.30

        assert_eq!(a + b, expected);
    }

    #[test]
    fn test_money_large_sums() {
        // Test that large transaction sums don't drift
        let unit = Money::from_cents(199); // $1.99
        let mut total = Money::ZERO;

        for _ in 0..10000 {
            total = total + unit;
        }

        assert_eq!(total, Money::from_cents(1990000)); // $19,900.00 exactly
    }

    #[test]
    fn test_money_percentage() {
        let subtotal = Money::from_cents(10000); // $100.00
        let tax = subtotal.apply_percentage(dec!(8.25)); // 8.25% tax

        assert_eq!(tax, Money::from_cents(825)); // $8.25
    }

    #[test]
    fn test_money_display() {
        let amount = Money::from_cents(1234);
        assert_eq!(format!("{}", amount), "$12.34");
    }
}
