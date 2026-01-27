//! Money type for representing monetary values.
//!
//! Uses cents-based integer representation to avoid floating-point
//! precision issues that plague monetary calculations.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul, Sub};

/// Supported currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Currency {
    #[default]
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
    CHF,
    CNY,
    INR,
    MXN,
}

impl Currency {
    /// Get the currency code (e.g., "USD").
    pub fn code(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
            Currency::CHF => "CHF",
            Currency::CNY => "CNY",
            Currency::INR => "INR",
            Currency::MXN => "MXN",
        }
    }

    /// Get the currency symbol (e.g., "$").
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "\u{20ac}",
            Currency::GBP => "\u{00a3}",
            Currency::JPY => "\u{00a5}",
            Currency::CAD => "CA$",
            Currency::AUD => "A$",
            Currency::CHF => "CHF",
            Currency::CNY => "\u{00a5}",
            Currency::INR => "\u{20b9}",
            Currency::MXN => "MX$",
        }
    }

    /// Get the number of decimal places for this currency.
    pub fn decimal_places(&self) -> u32 {
        match self {
            Currency::JPY => 0,
            _ => 2,
        }
    }

    /// Parse a currency code string.
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "USD" => Some(Currency::USD),
            "EUR" => Some(Currency::EUR),
            "GBP" => Some(Currency::GBP),
            "JPY" => Some(Currency::JPY),
            "CAD" => Some(Currency::CAD),
            "AUD" => Some(Currency::AUD),
            "CHF" => Some(Currency::CHF),
            "CNY" => Some(Currency::CNY),
            "INR" => Some(Currency::INR),
            "MXN" => Some(Currency::MXN),
            _ => None,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// A monetary value with currency.
///
/// Amounts are stored in the smallest unit of the currency (e.g., cents for USD).
/// This avoids floating-point precision issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Money {
    /// Amount in smallest currency unit (e.g., cents).
    pub amount_cents: i64,
    /// The currency.
    pub currency: Currency,
}

impl Money {
    /// Create a new Money value from cents.
    pub fn new(amount_cents: i64, currency: Currency) -> Self {
        Self {
            amount_cents,
            currency,
        }
    }

    /// Create a Money value from a decimal amount.
    ///
    /// ```
    /// use turbo_commerce::money::{Money, Currency};
    /// let price = Money::from_decimal(49.99, Currency::USD);
    /// assert_eq!(price.amount_cents, 4999);
    /// ```
    pub fn from_decimal(amount: f64, currency: Currency) -> Self {
        let multiplier = 10_i64.pow(currency.decimal_places());
        let amount_cents = (amount * multiplier as f64).round() as i64;
        Self::new(amount_cents, currency)
    }

    /// Create a zero amount in the given currency.
    pub fn zero(currency: Currency) -> Self {
        Self::new(0, currency)
    }

    /// Check if this is zero.
    pub fn is_zero(&self) -> bool {
        self.amount_cents == 0
    }

    /// Check if this is positive.
    pub fn is_positive(&self) -> bool {
        self.amount_cents > 0
    }

    /// Check if this is negative.
    pub fn is_negative(&self) -> bool {
        self.amount_cents < 0
    }

    /// Get the absolute value.
    pub fn abs(&self) -> Self {
        Self::new(self.amount_cents.abs(), self.currency)
    }

    /// Negate the amount.
    pub fn negate(&self) -> Self {
        Self::new(-self.amount_cents, self.currency)
    }

    /// Convert to a decimal value.
    pub fn to_decimal(&self) -> f64 {
        let divisor = 10_i64.pow(self.currency.decimal_places());
        self.amount_cents as f64 / divisor as f64
    }

    /// Format as a display string (e.g., "$49.99").
    pub fn display(&self) -> String {
        let decimal = self.to_decimal();
        let places = self.currency.decimal_places() as usize;
        format!("{}{:.places$}", self.currency.symbol(), decimal)
    }

    /// Format as a display string without symbol (e.g., "49.99").
    pub fn display_amount(&self) -> String {
        let decimal = self.to_decimal();
        let places = self.currency.decimal_places() as usize;
        format!("{:.places$}", decimal)
    }

    /// Add another Money value.
    ///
    /// # Panics
    /// Panics if currencies don't match. Use `try_add` for fallible addition.
    pub fn add(&self, other: &Money) -> Money {
        self.try_add(other).expect("Currency mismatch in addition")
    }

    /// Try to add another Money value, returning None if currencies don't match.
    pub fn try_add(&self, other: &Money) -> Option<Money> {
        if self.currency != other.currency {
            return None;
        }
        Some(Money::new(
            self.amount_cents + other.amount_cents,
            self.currency,
        ))
    }

    /// Subtract another Money value.
    ///
    /// # Panics
    /// Panics if currencies don't match.
    pub fn subtract(&self, other: &Money) -> Money {
        self.try_subtract(other)
            .expect("Currency mismatch in subtraction")
    }

    /// Try to subtract another Money value.
    pub fn try_subtract(&self, other: &Money) -> Option<Money> {
        if self.currency != other.currency {
            return None;
        }
        Some(Money::new(
            self.amount_cents - other.amount_cents,
            self.currency,
        ))
    }

    /// Multiply by a scalar.
    pub fn multiply(&self, factor: i64) -> Money {
        Money::new(self.amount_cents * factor, self.currency)
    }

    /// Multiply by a decimal factor (e.g., for percentages).
    pub fn multiply_decimal(&self, factor: f64) -> Money {
        let new_amount = (self.amount_cents as f64 * factor).round() as i64;
        Money::new(new_amount, self.currency)
    }

    /// Calculate a percentage of this amount.
    pub fn percentage(&self, percent: f64) -> Money {
        self.multiply_decimal(percent / 100.0)
    }

    /// Sum an iterator of Money values.
    pub fn sum<'a>(iter: impl Iterator<Item = &'a Money>, currency: Currency) -> Money {
        iter.fold(Money::zero(currency), |acc, m| acc + m.clone())
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, other: Money) -> Money {
        Money::add(&self, &other)
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, other: Money) -> Money {
        Money::subtract(&self, &other)
    }
}

impl Mul<i64> for Money {
    type Output = Money;

    fn mul(self, factor: i64) -> Money {
        self.multiply(factor)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_from_cents() {
        let m = Money::new(4999, Currency::USD);
        assert_eq!(m.amount_cents, 4999);
        assert_eq!(m.currency, Currency::USD);
    }

    #[test]
    fn test_money_from_decimal() {
        let m = Money::from_decimal(49.99, Currency::USD);
        assert_eq!(m.amount_cents, 4999);

        let m = Money::from_decimal(100.0, Currency::JPY);
        assert_eq!(m.amount_cents, 100); // JPY has no decimals
    }

    #[test]
    fn test_money_to_decimal() {
        let m = Money::new(4999, Currency::USD);
        assert!((m.to_decimal() - 49.99).abs() < 0.001);
    }

    #[test]
    fn test_money_display() {
        let m = Money::new(4999, Currency::USD);
        assert_eq!(m.display(), "$49.99");

        let m = Money::new(100, Currency::JPY);
        assert_eq!(m.display(), "\u{00a5}100");
    }

    #[test]
    fn test_money_addition() {
        let a = Money::new(1000, Currency::USD);
        let b = Money::new(500, Currency::USD);
        let c = a + b;
        assert_eq!(c.amount_cents, 1500);
    }

    #[test]
    fn test_money_subtraction() {
        let a = Money::new(1000, Currency::USD);
        let b = Money::new(300, Currency::USD);
        let c = a.subtract(&b);
        assert_eq!(c.amount_cents, 700);
    }

    #[test]
    fn test_money_multiply() {
        let m = Money::new(1000, Currency::USD);
        let doubled = m.multiply(2);
        assert_eq!(doubled.amount_cents, 2000);
    }

    #[test]
    fn test_money_percentage() {
        let m = Money::new(10000, Currency::USD); // $100.00
        let discount = m.percentage(10.0); // 10%
        assert_eq!(discount.amount_cents, 1000); // $10.00
    }

    #[test]
    #[should_panic(expected = "Currency mismatch")]
    fn test_money_currency_mismatch() {
        let usd = Money::new(1000, Currency::USD);
        let eur = Money::new(1000, Currency::EUR);
        let _ = usd + eur;
    }

    #[test]
    fn test_currency_from_code() {
        assert_eq!(Currency::from_code("USD"), Some(Currency::USD));
        assert_eq!(Currency::from_code("eur"), Some(Currency::EUR));
        assert_eq!(Currency::from_code("INVALID"), None);
    }
}
