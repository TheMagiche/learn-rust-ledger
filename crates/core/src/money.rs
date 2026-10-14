//! Money stored as whole minor units so totals never drift.

use std::fmt;
use std::iter::Sum;
use std::ops::Add;

use serde::{Deserialize, Serialize};

use crate::error::SpendError;

/// LEARN(newtype): `Money` is a tuple struct with one field. It is a different
/// type from `u64`, so a cent total cannot be passed where a plain count is expected.
/// LEARN(integer-money): the value is cents (or the smallest unit of the currency),
/// never a float. `0.1 + 0.2` is not `0.3` in binary floating point; integer cents add exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money(pub u64);

impl Money {
    // LEARN(impl-blocks): methods live in an `impl` block. `Money::parse` belongs to the type; `cents` is called on a value.
    /// Minor units, such as cents.
    pub fn cents(self) -> u64 {
        self.0
    }

    /// Parse a decimal string such as `"12.50"` into minor units.
    ///
    /// The input is major units with at most two decimal places (`"450"` means 450.00).
    /// Thousands separators are not accepted; [`Display`] adds those on the way out.
    /// Zero is a valid `Money`. Recording an expense rejects zero separately.
    ///
    /// # Examples
    ///
    /// ```
    /// use spend_core::Money;
    /// let money = Money::parse("12.50").expect("12.50 is a valid amount");
    /// assert_eq!(money.cents(), 1250);
    /// ```
    pub fn parse(input: &str) -> Result<Money, SpendError> {
        // LEARN(doc-tests): the example above is compiled and run by `cargo test`.
        // LEARN(result): `Result` is either `Ok(Money)` or `Err(SpendError)`. We do not panic on bad input.
        let text = input.trim();
        if text.is_empty() || text.starts_with('+') || text.starts_with('-') {
            return Err(SpendError::BadAmount(text.to_string()));
        }

        let (whole, frac) = match text.split_once('.') {
            Some((whole, frac)) if !frac.is_empty() => (whole, frac),
            Some(_) => return Err(SpendError::BadAmount(text.to_string())),
            None => (text, ""),
        };

        if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(SpendError::BadAmount(text.to_string()));
        }
        if frac.len() > 2 || !frac.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(SpendError::BadAmount(text.to_string()));
        }

        let major: u64 = match whole.parse() {
            Ok(value) => value,
            Err(_) => return Err(SpendError::BadAmount(text.to_string())),
        };
        let minor: u64 = match frac.len() {
            0 => 0,
            1 => match frac.parse::<u64>() {
                Ok(value) => value * 10,
                Err(_) => return Err(SpendError::BadAmount(text.to_string())),
            },
            2 => match frac.parse() {
                Ok(value) => value,
                Err(_) => return Err(SpendError::BadAmount(text.to_string())),
            },
            _ => return Err(SpendError::BadAmount(text.to_string())),
        };

        match major
            .checked_mul(100)
            .and_then(|value| value.checked_add(minor))
        {
            Some(cents) => Ok(Money(cents)),
            None => Err(SpendError::BadAmount(text.to_string())),
        }
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // Saturating keeps a huge total from panicking in a beginner program.
        Money(self.0.saturating_add(rhs.0))
    }
}

impl Sum for Money {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Money>,
    {
        // LEARN(generics): `<I: Iterator<Item = Money>>` means "any iterator of money",
        // not one concrete collection. `Vec`, slices, and maps can all be summed.
        iter.fold(Money(0), Add::add)
    }
}

/// LEARN(display): `Display` is how a value prints for humans. `{}` in `format!` calls this.
impl fmt::Display for Money {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let major = self.0 / 100;
        let minor = self.0 % 100;
        write!(formatter, "{}.{:02}", with_commas(major), minor)
    }
}

/// LEARN(strings): a `String` owns its UTF-8 bytes. We build one here so the caller can keep the text.
fn with_commas(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // LEARN(testing): `#[cfg(test)]` keeps this module out of the built program.
    // `cargo test` compiles it and runs every `#[test]` function.

    #[test]
    fn parse_table() {
        let cases = [
            ("0", Ok(0_u64)),
            ("0.0", Ok(0)),
            ("0.00", Ok(0)),
            ("12", Ok(1_200)),
            ("12.5", Ok(1_250)),
            ("12.50", Ok(1_250)),
            ("450", Ok(45_000)),
            ("1234.50", Ok(123_450)),
            ("  8.05 ", Ok(805)),
            ("-1", Err(())),
            ("1.234", Err(())),
            ("abc", Err(())),
            ("", Err(())),
            (".50", Err(())),
            ("12.", Err(())),
            ("+4", Err(())),
            ("1,234.50", Err(())),
            ("12.5.0", Err(())),
        ];

        for (input, expected) in cases {
            let parsed = Money::parse(input).map(|money| money.cents());
            match expected {
                Ok(cents) => assert_eq!(parsed, Ok(cents), "input {input}"),
                Err(()) => assert!(parsed.is_err(), "input {input} should fail"),
            }
        }
    }

    #[test]
    fn display_groups_thousands() {
        assert_eq!(Money(0).to_string(), "0.00");
        assert_eq!(Money(50).to_string(), "0.50");
        assert_eq!(Money(1_250).to_string(), "12.50");
        assert_eq!(Money(123_450).to_string(), "1,234.50");
        assert_eq!(Money(1_000_000).to_string(), "10,000.00");
    }

    #[test]
    fn add_and_sum() {
        let total: Money = [Money(100), Money(250), Money(50)].into_iter().sum();
        assert_eq!(total, Money(400));
        assert_eq!(Money(1) + Money(2), Money(3));
    }
}
