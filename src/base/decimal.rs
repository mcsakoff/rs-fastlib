use std::fmt::{Display, Formatter};

use crate::utils::bytes::trim_whitespace;
use crate::{Error, Result};

pub(crate) const MAX_EXPONENT: i32 = 63;
pub(crate) const MIN_EXPONENT: i32 = -63;

/// Represents a scaled decimal number.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Decimal {
    pub exponent: i32,
    pub mantissa: i64,
}

impl Decimal {
    #[must_use]
    pub fn new(exponent: i32, mantissa: i64) -> Decimal {
        Decimal { exponent, mantissa }
    }

    /// Converts a string into normalized decimal.
    ///
    /// The string has an integer part and a decimal part. It is allowed to specify either or both of them.
    /// A leading minus sign indicates a negative number, e.g.: `1`, `1.1`, `.1` and `-0.1`. The literal is
    /// whitespace trimmed.
    ///
    /// The value is normalized by adjusting the mantissa and exponent so that the integer remainder after
    /// dividing the mantissa by 10 is not zero: mant % 10 != 0. For example 100 would be normalized as 1 * 10^2.
    /// If the mantissa is zero, the normalized decimal has a zero mantissa and a zero exponent.
    /// # Errors
    /// Returns [`Error::Dynamic`] if the string has wrong format, or the exponent is out of
    /// range \[-63 … 63\], or the mantissa doesn't fit into `i64`.
    pub fn from_string(value: &str) -> Result<Decimal> {
        let invalid = || Error::Dynamic(format!("invalid decimal '{value}'")); // [ERR D11]
        let out_of_range = || Error::Dynamic(format!("decimal '{value}' is out of range")); // [ERR R1]

        let s = trim_whitespace(value);
        let (negative, s) = match s.as_bytes().first() {
            Some(b'-') => (true, &s[1..]),
            Some(b'+') => (false, &s[1..]),
            _ => (false, s),
        };
        let (integer, fraction) = s.split_once('.').unwrap_or((s, ""));
        if (integer.is_empty() && fraction.is_empty())
            || !integer
                .bytes()
                .chain(fraction.bytes())
                .all(|b| b.is_ascii_digit())
        {
            return Err(invalid());
        }

        // Significant digits without leading and trailing zeroes.
        let digits = format!("{integer}{fraction}");
        let digits = digits.trim_start_matches('0');
        let significant = digits.trim_end_matches('0');
        if significant.is_empty() {
            return Ok(Decimal::new(0, 0));
        }
        let trailing_zeroes =
            i64::try_from(digits.len() - significant.len()).map_err(|_| out_of_range())?;
        let fraction_len = i64::try_from(fraction.len()).map_err(|_| out_of_range())?;
        let exponent = i32::try_from(trailing_zeroes - fraction_len)
            .ok()
            .filter(|e| (MIN_EXPONENT..=MAX_EXPONENT).contains(e))
            .ok_or_else(out_of_range)?;

        let mut mantissa: i64 = 0;
        for d in significant.bytes().map(|b| i64::from(b - b'0')) {
            // Accumulate negative numbers as negative to support i64::MIN.
            mantissa = mantissa
                .checked_mul(10)
                .and_then(|m| {
                    if negative {
                        m.checked_sub(d)
                    } else {
                        m.checked_add(d)
                    }
                })
                .ok_or_else(out_of_range)?;
        }
        Ok(Decimal::new(exponent, mantissa))
    }

    /// Creates decimal value from float.
    /// # Errors
    /// Returns error if number is not finite.
    pub fn from_float(value: f64) -> Result<Decimal> {
        if !value.is_finite() {
            return Err(Error::Static(format!("Not a finite decimal '{value}'")));
        }
        Decimal::from_string(&format!("{value}"))
    }

    /// If the number is in fact an integer, it is converted as if was of integer type. Otherwise, the number
    /// is represented by an integer part and a decimal part separated by a decimal point (‘.’). Each part is
    /// a sequence of digits ‘0’ – ‘9’. There must be at least one digit on each side of the decimal point.
    /// If the number is negative it is preceded by a minus sign (‘-’).
    /// The integer part must not have any leading zeroes.
    #[allow(clippy::inherent_to_string_shadow_display)]
    #[must_use]
    pub fn to_string(&self) -> String {
        let (sign, integer, fraction) = self.parts();
        if fraction.bytes().all(|b| b == b'0') {
            if integer == "0" {
                return integer;
            }
            return format!("{sign}{integer}");
        }
        format!("{sign}{integer}.{fraction}")
    }

    // Splits the number into the sign, the integer part and the decimal part (`exponent` digits if negative).
    fn parts(&self) -> (&'static str, String, String) {
        let sign = if self.mantissa < 0 { "-" } else { "" };
        let digits = self.mantissa.unsigned_abs().to_string();
        if self.mantissa == 0 {
            return ("", "0".to_string(), "0".repeat(self.scale()));
        }
        if self.exponent >= 0 {
            let zeroes = "0".repeat(self.exponent.unsigned_abs() as usize);
            return (sign, format!("{digits}{zeroes}"), String::new());
        }
        let scale = self.scale();
        if digits.len() > scale {
            let (integer, fraction) = digits.split_at(digits.len() - scale);
            (sign, integer.to_string(), fraction.to_string())
        } else {
            let zeroes = "0".repeat(scale - digits.len());
            (sign, "0".to_string(), format!("{zeroes}{digits}"))
        }
    }

    // Number of digits after the decimal point.
    fn scale(&self) -> usize {
        if self.exponent < 0 {
            self.exponent.unsigned_abs() as usize
        } else {
            0
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn to_float(&self) -> f64 {
        let mantissa = self.mantissa as f64;
        match self.exponent {
            0 => mantissa,
            // Use exact integer arithmetic when possible: it gives MUCH better results.
            exponent if exponent > 0 => match 10i64
                .checked_pow(exponent.unsigned_abs())
                .and_then(|m| self.mantissa.checked_mul(m))
            {
                Some(v) => v as f64,
                None => mantissa * 10f64.powi(exponent),
            },
            // Powers of ten up to 10^22 are exact in f64.
            exponent => mantissa / 10f64.powi(-exponent),
        }
    }
}

/// Format the decimal as number with specific number of digits after the decimal point.
impl Display for Decimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (sign, integer, fraction) = self.parts();
        if fraction.is_empty() {
            // keep .0 at the end even if the number is integer
            write!(f, "{sign}{integer}.0")
        } else {
            write!(f, "{sign}{integer}.{fraction}")
        }
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Decimal::new(0, 0)
    }
}

impl From<Decimal> for f64 {
    fn from(value: Decimal) -> Self {
        value.to_float()
    }
}

impl TryFrom<f64> for Decimal {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self> {
        Self::from_float(value)
    }
}

#[cfg(feature = "rust_decimal")]
impl From<Decimal> for rust_decimal::Decimal {
    fn from(value: Decimal) -> Self {
        if value.exponent <= 0 {
            Self::new(value.mantissa, -value.exponent as u32)
        } else {
            let number = value.mantissa * 10i64.pow(value.exponent as u32);
            Self::new(number, 0)
        }
    }
}

#[cfg(feature = "rust_decimal")]
impl TryFrom<rust_decimal::Decimal> for Decimal {
    type Error = Error;

    fn try_from(value: rust_decimal::Decimal) -> Result<Self> {
        use rust_decimal::prelude::ToPrimitive;

        // Get mantissa and scale as mantissa and exponent and adjust as per
        // the FIX specification for decimals: the integer remainder after dividing
        // the mantissa by 10 is not zero.
        let mut exponent = -(value.scale() as i64);
        let mut mantissa = value.mantissa();
        if mantissa != 0 {
            while mantissa % 10 == 0 {
                mantissa /= 10;
                exponent += 1;
            }
        }
        // Check mantissa and exponent are within bounds.
        let mantissa = match mantissa.to_i64() {
            Some(m) => m,
            None => {
                return Err(Error::Static("Mantissa is too large".to_string()));
            }
        };
        let exponent = match exponent.to_i32() {
            Some(e) => e,
            None => {
                return Err(Error::Static("Exponent is too large".to_string()));
            }
        };
        Ok(Self::new(exponent, mantissa))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decimal_from_string() {
        struct TestCase {
            input: &'static str,
            components: (i32, i64),
            float: f64,
        }

        fn do_test(tts: Vec<TestCase>) {
            for tt in tts {
                let d = Decimal::from_string(tt.input).unwrap();
                assert_eq!((d.exponent, d.mantissa), tt.components);
                assert_eq!(d.to_float(), tt.float);
            }
        }

        do_test(vec![
            TestCase {
                input: "1200.45",
                components: (-2, 120045),
                float: 1200.45,
            },
            TestCase {
                input: "001200.4500",
                components: (-2, 120045),
                float: 1200.45,
            },
            TestCase {
                input: "0",
                components: (0, 0),
                float: 0.0,
            },
            TestCase {
                input: "0.0",
                components: (0, 0),
                float: 0.0,
            },
            TestCase {
                input: "1",
                components: (0, 1),
                float: 1.0,
            },
            TestCase {
                input: "10",
                components: (1, 1),
                float: 10.0,
            },
            TestCase {
                input: "-1",
                components: (0, -1),
                float: -1.0,
            },
            TestCase {
                input: "0.03",
                components: (-2, 3),
                float: 0.03,
            },
            TestCase {
                input: "-0.03",
                components: (-2, -3),
                float: -0.03,
            },
        ]);
    }

    #[test]
    fn decimal_to_string_exact() {
        for (components, string, display) in [
            (
                (-2, 1_234_567_890_123_456_789),
                "12345678901234567.89",
                "12345678901234567.89",
            ),
            (
                (-2, i64::MIN),
                "-92233720368547758.08",
                "-92233720368547758.08",
            ),
            (
                (63, 1),
                "1000000000000000000000000000000000000000000000000000000000000000",
                "1000000000000000000000000000000000000000000000000000000000000000.0",
            ),
            (
                (-63, 1),
                "0.000000000000000000000000000000000000000000000000000000000000001",
                "0.000000000000000000000000000000000000000000000000000000000000001",
            ),
        ] {
            let d = Decimal::new(components.0, components.1);
            assert_eq!(d.to_string(), string, "{components:?}");
            assert_eq!(format!("{d}"), display, "{components:?}");
        }
        assert_eq!(Decimal::new(63, 1).to_float(), 1e63);
        assert_eq!(Decimal::new(-30, 5).to_float(), 5e-30);
        assert_eq!(
            Decimal::new(20, i64::MAX).to_float(),
            i64::MAX as f64 * 1e20
        );
    }

    #[test]
    fn decimal_to_string() {
        struct TestCase {
            components: (i32, i64),
            string: &'static str,
            display: &'static str,
        }

        fn do_test(tts: Vec<TestCase>) {
            for tt in tts {
                let d = Decimal::new(tt.components.0, tt.components.1);
                assert_eq!(d.to_string(), tt.string, "(string)");
                assert_eq!(&format!("{d}"), tt.display, "(display)");
            }
        }

        do_test(vec![
            TestCase {
                components: (0, 0),
                string: "0",
                display: "0.0", // keep .0 at the end even in the number in integer
            },
            TestCase {
                components: (2, 0),
                string: "0",
                display: "0.0", // keep .0 at the end even in the number in integer
            },
            TestCase {
                components: (1, 2),
                string: "20",
                display: "20.0", // keep .0 at the end even in the number in integer
            },
            TestCase {
                components: (-3, 1000),
                string: "1",
                display: "1.000",
            },
            TestCase {
                components: (-5, 100),
                string: "0.00100",
                display: "0.00100",
            },
            TestCase {
                components: (-2, 0),
                string: "0",
                display: "0.00",
            },
            TestCase {
                components: (1, -2),
                string: "-20",
                display: "-20.0", // keep .0 at the end even in the number in integer
            },
            TestCase {
                components: (-3, -1000),
                string: "-1",
                display: "-1.000",
            },
            TestCase {
                components: (-5, -100),
                string: "-0.00100",
                display: "-0.00100",
            },
        ]);
    }

    #[test]
    fn decimal_from_float() {
        struct TestCase {
            input: f64,
            components: (i32, i64),
        }

        fn do_test(tts: Vec<TestCase>) {
            for tt in tts {
                let d = Decimal::from_float(tt.input).unwrap();
                assert_eq!((d.exponent, d.mantissa), tt.components);
            }
        }

        do_test(vec![
            TestCase {
                input: 1200.45,
                components: (-2, 120045),
            },
            TestCase {
                input: 0.0,
                components: (0, 0),
            },
            TestCase {
                input: 1.0,
                components: (0, 1),
            },
            TestCase {
                input: 10.0,
                components: (1, 1),
            },
            TestCase {
                input: -1.0,
                components: (0, -1),
            },
            TestCase {
                input: 0.1,
                components: (-1, 1),
            },
        ]);
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn decimal_from_rust_decimal() {
        use rust_decimal::prelude::FromPrimitive;

        struct TestCase {
            float: f64,
            components: (i32, i64),
        }

        fn do_test(tts: Vec<TestCase>) {
            for tt in tts {
                let rd = rust_decimal::Decimal::from_f64(tt.float).unwrap();
                let d = Decimal::try_from(rd).unwrap();
                assert_eq!((d.exponent, d.mantissa), tt.components);
            }
        }

        do_test(vec![
            TestCase {
                float: 1200.45,
                components: (-2, 120045),
            },
            TestCase {
                float: 0.0,
                components: (0, 0),
            },
            TestCase {
                float: 1.0,
                components: (0, 1),
            },
            TestCase {
                float: 100.0,
                components: (2, 1),
            },
            TestCase {
                float: -1.0,
                components: (0, -1),
            },
            TestCase {
                float: 0.03,
                components: (-2, 3),
            },
            TestCase {
                float: -0.03,
                components: (-2, -3),
            },
        ]);
    }

    #[test]
    fn decimal_to_any() {
        struct TestCase {
            components: (i32, i64),
            float: f64,
        }

        fn do_test(tts: Vec<TestCase>) {
            #[cfg(feature = "rust_decimal")]
            use rust_decimal::prelude::ToPrimitive;

            for tt in tts {
                let d = Decimal::new(tt.components.0, tt.components.1);
                assert_eq!(d.to_float(), tt.float);
                assert_eq!(f64::from(d.clone()), tt.float);

                #[cfg(feature = "rust_decimal")]
                assert_eq!(rust_decimal::Decimal::from(d).to_f64().unwrap(), tt.float);
            }
        }

        do_test(vec![
            TestCase {
                components: (-2, 120045),
                float: 1200.45,
            },
            TestCase {
                components: (0, 0),
                float: 0.0,
            },
            TestCase {
                components: (0, 1),
                float: 1.0,
            },
            TestCase {
                components: (2, 1),
                float: 100.0,
            },
            TestCase {
                components: (0, -1),
                float: -1.0,
            },
            TestCase {
                components: (-2, 3),
                float: 0.03,
            },
            TestCase {
                components: (-2, -3),
                float: -0.03,
            },
        ]);
    }
}
