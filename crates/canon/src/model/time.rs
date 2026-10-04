//! Reading [`Instant`] and [`Age`] text as seconds. Pure: no clock is read; an instant is only ever
//! compared with another instant given as text.

use super::ids::{Age, Instant};

impl Instant {
    /// Seconds since `1970-01-01T00:00:00Z`, or `None` when the text is not exactly
    /// `YYYY-MM-DDTHH:MM:SSZ` with a real calendar date (Gregorian, leap years included) and a time
    /// from `00:00:00` to `23:59:59`.
    pub fn seconds(&self) -> Option<i64> {
        let text = self.as_str().as_bytes();
        if text.len() != 20
            || [
                (4, b'-'),
                (7, b'-'),
                (10, b'T'),
                (13, b':'),
                (16, b':'),
                (19, b'Z'),
            ]
            .iter()
            .any(|&(at, byte)| text[at] != byte)
        {
            return None;
        }
        let number = |from: usize, to: usize| -> Option<i64> {
            text[from..to].iter().try_fold(0i64, |value, &byte| {
                byte.is_ascii_digit()
                    .then(|| value * 10 + i64::from(byte - b'0'))
            })
        };
        let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
        let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
        if !(1..=12).contains(&month)
            || day < 1
            || day > days_in_month(year, month)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return None;
        }
        Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
    }
}

impl Age {
    /// The length in seconds, or `None` when the text is not a whole number (digits only, no
    /// leading zero) followed by one unit, `s`, `m`, `h` or `d`, or is too long to count in
    /// seconds.
    pub fn seconds(&self) -> Option<i64> {
        let (digits, unit) = self.parts()?;
        digits.parse::<i64>().ok()?.checked_mul(unit)
    }

    /// Whether the text is a whole number (digits only, no leading zero) followed by one unit,
    /// `s`, `m`, `h` or `d`, however long. A well-formed age [`Age::seconds`] does not read is too
    /// long to count in seconds.
    pub fn is_well_formed(&self) -> bool {
        self.parts().is_some()
    }

    /// The digits and the unit's length in seconds, when the text is well formed.
    fn parts(&self) -> Option<(&str, i64)> {
        let text = self.as_str();
        let unit = match text.as_bytes().last()? {
            b's' => 1,
            b'm' => 60,
            b'h' => 3_600,
            b'd' => 86_400,
            _ => return None,
        };
        let digits = &text[..text.len() - 1];
        if digits.is_empty()
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
            || (digits.len() > 1 && digits.starts_with('0'))
        {
            return None;
        }
        Some((digits, unit))
    }
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days from 1970-01-01 to the given date of the proleptic Gregorian calendar (Howard Hinnant's
/// `days_from_civil`), for years 0 to 9999.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instant(text: &str) -> Option<i64> {
        Instant::new(text).seconds()
    }

    fn age(text: &str) -> Option<i64> {
        Age::new(text).seconds()
    }

    #[test]
    fn an_instant_reads_as_seconds_since_the_epoch() {
        assert_eq!(instant("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(instant("1970-01-02T00:00:01Z"), Some(86_401));
        assert_eq!(instant("2000-03-01T00:00:00Z"), Some(951_868_800));
        assert_eq!(instant("2026-10-04T12:00:00Z"), Some(1_791_115_200));
        assert_eq!(instant("1969-12-31T23:59:59Z"), Some(-1));
        assert_eq!(instant("2024-02-29T00:00:00Z"), Some(1_709_164_800));
    }

    #[test]
    fn an_instant_in_any_other_form_is_not_read() {
        for text in [
            "",
            "2026-10-04",
            "2026-10-04T12:00:00",
            "2026-10-04T12:00:00+00:00",
            "2026-10-04T12:00:00.5Z",
            "2026-10-04 12:00:00Z",
            "2026-10-04t12:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-00-01T00:00:00Z",
            "2026-02-29T00:00:00Z",
            "1900-02-29T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "2026-10-00T00:00:00Z",
            "2026-10-04T24:00:00Z",
            "2026-10-04T12:60:00Z",
            "2026-10-04T12:00:60Z",
            "+026-10-04T12:00:00Z",
            "2026-1a-04T12:00:00Z",
        ] {
            assert_eq!(instant(text), None, "{text:?}");
        }
    }

    #[test]
    fn an_age_reads_as_seconds() {
        assert_eq!(age("0s"), Some(0));
        assert_eq!(age("45s"), Some(45));
        assert_eq!(age("5m"), Some(300));
        assert_eq!(age("1h"), Some(3_600));
        assert_eq!(age("2d"), Some(172_800));
    }

    #[test]
    fn an_age_in_any_other_form_is_not_read() {
        for text in [
            "",
            "h",
            "1",
            "1H",
            "1w",
            "05m",
            "-1h",
            "+1h",
            "1.5h",
            " 1h",
            "1h ",
            "1 h",
            "1hh",
            "99999999999999999999s",
            "106751991167301d",
        ] {
            assert_eq!(age(text), None, "{text:?}");
        }
    }

    /// A well-formed age is one in the right form whatever its length; the too-long ones are
    /// well formed but not read.
    #[test]
    fn a_too_long_age_is_well_formed_and_a_malformed_one_is_not() {
        for text in ["0s", "5m", "99999999999999999999s", "106751991167301d"] {
            assert!(Age::new(text).is_well_formed(), "{text:?}");
        }
        for text in ["", "05m", "1H", "1 h", "-1h"] {
            assert!(!Age::new(text).is_well_formed(), "{text:?}");
        }
    }
}
