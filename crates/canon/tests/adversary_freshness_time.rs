//! Adversary cases for story:evidence-freshness against `Instant::seconds` and `Age::seconds`.
//!
//! Driven from the doc comments in `crates/canon/src/model/time.rs` and `model/ids.rs`: an instant
//! is exactly `YYYY-MM-DDTHH:MM:SSZ` with a real Gregorian date (leap years included), from
//! `00:00:00` to `23:59:59`; an age is a whole number without a leading zero followed by one of
//! `s`, `m`, `h`, `d`, refused when too long to count in seconds. `days_from_civil` claims years 0
//! to 9999.

use b10x_canon::model::{Age, Instant};

fn instant(text: &str) -> Option<i64> {
    Instant::new(text).seconds()
}

fn age(text: &str) -> Option<i64> {
    Age::new(text).seconds()
}

fn leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn month_length(year: i64, month: i64) -> i64 {
    [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ][(month - 1) as usize]
}

/// Every calendar day from 0000-01-01 to 9999-12-31 is exactly one day after the one before it,
/// counted independently of the implementation from the fixed anchor 0000-01-01 = day -719528;
/// the day after each month's last day is refused.
#[test]
fn every_day_of_years_0_to_9999_is_one_day_after_the_last() {
    let mut day: i64 = -719_528;
    for year in 0..=9999_i64 {
        for month in 1..=12_i64 {
            let length = month_length(year, month);
            for d in 1..=length {
                let text = format!("{year:04}-{month:02}-{d:02}T00:00:00Z");
                assert_eq!(instant(&text), Some(day * 86_400), "{text}");
                day += 1;
            }
            let past = format!("{year:04}-{month:02}-{:02}T00:00:00Z", length + 1);
            assert_eq!(instant(&past), None, "{past}");
        }
    }
    assert_eq!(day, 2_932_897, "9999-12-31 is day 2932896");
}

#[test]
fn the_range_ends_read_as_their_known_epoch_seconds() {
    assert_eq!(instant("0000-01-01T00:00:00Z"), Some(-62_167_219_200));
    assert_eq!(instant("0000-02-29T00:00:00Z"), Some(-62_162_121_600));
    assert_eq!(instant("9999-12-31T23:59:59Z"), Some(253_402_300_799));
    assert_eq!(instant("1970-01-01T23:59:59Z"), Some(86_399));
    assert_eq!(instant("2000-02-29T00:00:00Z"), Some(951_782_400));
    for refused in [
        "2100-02-29T00:00:00Z",
        "2016-12-31T23:59:60Z",
        "2026-10-04T12:00:00z",
        "2026-10-04T12:00:00-00:00",
        "2026-10-04T12:00:00.000Z",
        "2026-10-04T12:00:00,5Z",
        "10000-01-01T00:00:00Z",
        "-001-01-01T00:00:00Z",
        "2026-10-04T12:00:00Z\n",
        "\u{ff12}026-10-04T12:00:00Z",
    ] {
        assert_eq!(instant(refused), None, "{refused:?}");
    }
}

#[test]
fn an_age_reads_to_the_last_second_that_fits_and_no_further() {
    assert_eq!(age("9223372036854775807s"), Some(i64::MAX));
    assert_eq!(age("9223372036854775808s"), None);
    assert_eq!(age("153722867280912930m"), Some(9_223_372_036_854_775_800));
    assert_eq!(age("153722867280912931m"), None);
    assert_eq!(age("2562047788015215h"), Some(9_223_372_036_854_774_000));
    assert_eq!(age("2562047788015216h"), None);
    assert_eq!(age("106751991167300d"), Some(9_223_372_036_854_720_000));
    for refused in [
        "00s",
        "0d0",
        "1S",
        "1M",
        "1D",
        "\u{0661}s",
        "1\u{0301}s",
        "s1",
        "1é",
    ] {
        assert_eq!(age(refused), None, "{refused:?}");
    }
}
