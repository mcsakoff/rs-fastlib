use crate::{Error, Result};

pub(crate) fn make_decimal(exponent: i32, mantissa: i64) -> f64 {
    // Ok(Some(Value::Decimal(mantissa as f64 * 10_f64.powf(exponent as f64))))

    // This is pretty ugly but gives MUCH better results than the previous implementation!
    let value;
    if exponent > 0 {
        let multiplier = 10i64.pow(exponent as u32);
        value = (mantissa * multiplier) as f64;
    } else if exponent < 0 {
        let divisor = 10u64.pow(-exponent as u32);
        value = mantissa as f64 / divisor as f64;
    } else {
        value = mantissa as f64;
    }
    value
}

// If the field is of type decimal, the value resulting from the conversion is normalized. The reason for this is that
// the exponent and mantissa must be predictable when operators are applied to them individually. A decimal value
// is normalized by adjusting the mantissa and exponent so that the integer remainder after dividing the mantissa
// by 10 is not zero: mant % 10 != 0. For example 100 would be normalized as 1 * 10^2. If the mantissa is zero,
// the normalized decimal has a zero mantissa and a zero exponent.
pub(crate) fn decimal_normalize(value: &str) -> Result<(i32, i64)> {
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
            return Ok((0, 0));
        }
        let (e, m) = scale_down(mantissa);
        exponent += e;
        mantissa = m;
    } else {
        return Err(Error::Static(format!("Not a decimal '{}'", value)));
    }
    Ok((exponent, mantissa))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn normalize() {
        struct TestCase {
            input: &'static str,
            result: (i32, i64),
        }

        fn do_test(tts: Vec<TestCase>) {
            for tt in tts {
                let (exponent, mantissa) = decimal_normalize(tt.input).unwrap();
                assert_eq!((exponent, mantissa), tt.result);
            }
        }

        do_test(vec![
            TestCase {
                input: "1200.45",
                result: (-2, 120045),
            },
            TestCase {
                input: "001200.4500",
                result: (-2, 120045),
            },
            TestCase {
                input: "0",
                result: (0, 0),
            },
            TestCase {
                input: "0.0",
                result: (0, 0),
            },
            TestCase {
                input: "1",
                result: (0, 1),
            },
            TestCase {
                input: "10",
                result: (1, 1),
            },
            TestCase {
                input: "-1",
                result: (0, -1),
            },
        ]);
    }
}
