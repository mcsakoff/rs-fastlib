use std::fmt::{Display, Formatter};

use crate::{Error, Result};

/// Represents a scaled decimal number.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Decimal {
    pub exponent: i32,
    pub mantissa: i64,
}

impl Decimal {
    pub fn new(exponent: i32, mantissa: i64) -> Decimal {
        Decimal { exponent, mantissa }
    }

    // If the field is of type decimal, the value resulting from the conversion is normalized. The reason for this is that
    // the exponent and mantissa must be predictable when operators are applied to them individually. A decimal value
    // is normalized by adjusting the mantissa and exponent so that the integer remainder after dividing the mantissa
    // by 10 is not zero: mant % 10 != 0. For example 100 would be normalized as 1 * 10^2. If the mantissa is zero,
    // the normalized decimal has a zero mantissa and a zero exponent.
    pub fn from_string(value: &str) -> Result<Decimal> {
        fn scale_down(mut value: i64) -> (i32, i64) {
            let mut scale = 0;
            if value != 0 {
                while value % 10 == 0 {
                    value /= 10;
                    scale += 1;
                }
            }
            (scale, value)
        }

        let mut parts = value.split('.');
        let part1 = parts.next();
        let part2 = parts.next();
        let part3 = parts.next();

        if part1.is_none() || part3.is_some() {
            return Err(Error::Static(format!("Not a decimal '{}'", value)));
        }
        let integer = part1.unwrap();

        let (exponent, mantissa) = if let Some(fractional) = part2 {
            let mantissa = format!("{}{}", integer, fractional).parse::<i64>()?;
            if mantissa == 0 {
                return Ok(Decimal::new(0, 0));
            }
            let (exponent_fix, mantissa) = scale_down(mantissa);
            let exponent = -(fractional.len() as i32) + exponent_fix;
            (exponent, mantissa)
        } else {
            scale_down(integer.parse::<i64>()?)
        };
        Ok(Decimal::new(exponent, mantissa))
    }

    pub fn from_float(value: f64) -> Result<Decimal> {
        if !value.is_finite() {
            return Err(Error::Static(format!("Not a finite decimal '{}'", value)));
        }
        Decimal::from_string(&format!("{value}"))
    }

    /// If the number is in fact an integer, it is converted as if was of integer type. Otherwise, the number
    /// is represented by an integer part and a decimal part separated by a decimal point (‘.’). Each part is
    /// a sequence of digits ‘0’ – ‘9’. There must be at least one digit on each side of the decimal point.
    /// If the number is negative it is preceded by a minus sign (‘-’).
    /// The integer part must not have any leading zeroes.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        if self.exponent >= 0 {
            (self.mantissa * 10i64.pow(self.exponent as u32)).to_string()
        } else {
            let divisor = 10i64.pow(-self.exponent as u32);
            if self.mantissa % divisor == 0 {
                (self.mantissa / divisor).to_string()
            } else {
                format!("{:.*}", -self.exponent as usize, self.to_float())
            }
        }
    }

    pub fn to_float(&self) -> f64 {
        // self.mantissa as f64 * 10_f64.powf(self.exponent as f64)

        // This is pretty ugly but gives MUCH better results than the implementation above!
        if self.exponent > 0 {
            let multiplier = 10i64.pow(self.exponent as u32);
            (self.mantissa * multiplier) as f64
        } else if self.exponent < 0 {
            let divisor = 10u64.pow(-self.exponent as u32);
            self.mantissa as f64 / divisor as f64
        } else {
            self.mantissa as f64
        }
    }
}

/// Format the decimal as number with specific number of digits after the decimal point.
impl Display for Decimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.exponent >= 0 {
            // keep .0 at the end even if the number is integer
            write!(f, "{}.0", self.mantissa * 10i64.pow(self.exponent as u32))
        } else {
            write!(f, "{:.*}", -self.exponent as usize, self.to_float())
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
