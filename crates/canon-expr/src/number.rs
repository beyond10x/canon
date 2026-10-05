use crate::{Diagnostic, Exact, Value, error};
use std::cmp::Ordering;

pub(crate) fn canonical_integer(s: &str) -> Result<i128, Diagnostic> {
    let value = s
        .parse::<i128>()
        .map_err(|_| error("number-range", "integer is outside i128"))?;
    if value.to_string() != s {
        return Err(error("number-canonical", "integer is not canonical"));
    }
    Ok(value)
}
impl Exact {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        let n = canonical_integer(&self.coefficient)?;
        if self.scale > 0 && (n == 0 || n % 10 == 0) {
            return Err(error("number-canonical", "decimal has redundant scale"));
        }
        Ok(())
    }
    pub(crate) fn lexeme(&self) -> String {
        if self.scale == 0 {
            self.coefficient.clone()
        } else {
            format!("{}e-{}", self.coefficient, self.scale)
        }
    }
}
pub(crate) fn parse_number(text: &str) -> Result<Value, Diagnostic> {
    let decimal = text.contains(['.', 'e', 'E']);
    let (mantissa, exp) = if let Some((a, b)) = text.split_once(['e', 'E']) {
        (
            a,
            b.parse::<i32>()
                .map_err(|_| error("number-range", "exponent out of range"))?,
        )
    } else {
        (text, 0)
    };
    let negative = mantissa.starts_with('-');
    let mantissa = mantissa.strip_prefix('-').unwrap_or(mantissa);
    let (whole, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
        || whole.len() > 1 && whole.starts_with('0')
    {
        return Err(error("number", "invalid numeric literal"));
    }
    let scale = (frac.len() as i64) - i64::from(exp);
    if !(-39..=255).contains(&scale) {
        return Err(error(
            "number-range",
            "decimal scale is outside 0..255 or magnitude exceeds i128",
        ));
    }
    let mut digits = format!("{whole}{frac}");
    if scale < 0 {
        digits.push_str(&"0".repeat((-scale) as usize));
    }
    let mut scale = scale.max(0) as u8;
    while scale > 0 && digits.ends_with('0') {
        digits.pop();
        scale -= 1;
    }
    let digits = digits.trim_start_matches('0');
    let mut coefficient = if digits.is_empty() {
        "0".into()
    } else {
        format!("{}{digits}", if negative { "-" } else { "" })
    };
    if coefficient == "0" {
        scale = 0;
    }
    let n = coefficient
        .parse::<i128>()
        .map_err(|_| error("number-range", "numeric coefficient is outside i128"))?;
    coefficient = n.to_string();
    if decimal {
        Ok(Value::Decimal(Exact { coefficient, scale }))
    } else {
        Ok(Value::Integer(coefficient))
    }
}
fn parts(value: &Value) -> Option<(&str, u8)> {
    match value {
        Value::Integer(n) => Some((n, 0)),
        Value::Decimal(n) => Some((&n.coefficient, n.scale)),
        _ => None,
    }
}
pub(crate) fn compare(a: &Value, b: &Value) -> Option<Ordering> {
    let (a, sa) = parts(a)?;
    let (b, sb) = parts(b)?;
    let an = a.starts_with('-');
    let bn = b.starts_with('-');
    if an != bn {
        return Some(if an {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    let a = a.trim_start_matches('-');
    let b = b.trim_start_matches('-');
    let mut aa = a.to_owned();
    let mut bb = b.to_owned();
    let scale = sa.max(sb);
    aa.push_str(&"0".repeat((scale - sa) as usize));
    bb.push_str(&"0".repeat((scale - sb) as usize));
    let aa = aa.trim_start_matches('0');
    let bb = bb.trim_start_matches('0');
    let order = aa.len().cmp(&bb.len()).then_with(|| aa.cmp(bb));
    Some(if an { order.reverse() } else { order })
}
pub(crate) fn negate(v: Value) -> Result<Value, Diagnostic> {
    match v {
        Value::Integer(s) => Ok(Value::integer(
            canonical_integer(&s)?
                .checked_neg()
                .ok_or_else(|| error("number-range", "negation overflows i128"))?,
        )),
        Value::Decimal(mut n) => {
            n.coefficient = canonical_integer(&n.coefficient)?
                .checked_neg()
                .ok_or_else(|| error("number-range", "negation overflows i128"))?
                .to_string();
            Ok(Value::Decimal(n))
        }
        _ => Err(error("type", "negation requires number")),
    }
}
