use std::fmt::{Display, Formatter};
use crate::{Error, Result};

/// Represents a scaled decimal number.
#[derive(Debug, PartialEq, Clone)]
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
        let mut exponent: i32;
        let mut mantissa: i64;

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

        let parts: Vec<_> = value.split(".").collect();
        if parts.len() == 1 {
            mantissa = i64::from_str_radix(parts[0], 10)?;
            (exponent, mantissa) = scale_down(mantissa);
        } else if parts.len() == 2 {
            exponent = -(parts[1].len() as i32);
            mantissa = i64::from_str_radix(&format!("{}{}", parts[0], parts[1]), 10)?;
            if mantissa == 0 {
                return Ok(Decimal::new(0, 0));
            }
            let (e, m) = scale_down(mantissa);
            exponent += e;
            mantissa = m;
        } else {
            return Err(Error::Static(format!("Not a decimal '{}'", value)));
        }
        Ok(Decimal::new(exponent, mantissa))
    }

    pub fn from_float(value: f64) -> Result<Decimal> {
        Decimal::from_string(&format!("{value}"))
    }

    /// If the number is in fact an integer, it is converted as if was of integer type. Otherwise, the number
    /// is represented by an integer part and a decimal part separated by a decimal point (‘.’). Each part is
    /// a sequence of digits ‘0’ – ‘9’. There must be at least one digit on each side of the decimal point.
    /// If the number is negative it is preceded by a minus sign (‘-’).
    /// The integer part must not have any leading zeroes.
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

impl Into<f64> for Decimal {
    fn into(self) -> f64 {
        self.to_float()
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
}
