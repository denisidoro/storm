use anyhow::Result;
use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::time::SystemTime;
use thiserror::Error;

use crate::crypto;

#[derive(Error, Debug)]
pub enum DateError {
    #[error("invalid seconds {0}")]
    _Seconds(i64),
    #[error("parse error {0}")]
    Parse(#[from] chrono::ParseError),
    #[error("invalid ymd {0}")]
    Ymd(u32, u32, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SmallDate(u16);

const DAYS_FROM_CE_TO_UNIX: i32 = 719_162;
const DAYS_FROM_UNIX_TO_START: u16 = 8_401; // 1993-01-01
const DAYS_FROM_CE_TO_START: i32 = DAYS_FROM_CE_TO_UNIX + DAYS_FROM_UNIX_TO_START as i32;

pub fn now_hours_base36() -> Result<String> {
    let now = Utc::now().timestamp() as u128;
    let base: u128 = 1641006000;
    let hours = (now - base) / (60 * 60);
    let az = format!("{:0>4}", crypto::format_radix(hours, 36)?.to_uppercase());
    Ok(az)
}

fn naive_to_date(naive: NaiveDateTime) -> Result<SmallDate> {
    let days = naive.num_days_from_ce() - DAYS_FROM_CE_TO_START;
    let days = u16::try_from(days)?;
    Ok(SmallDate(days))
}

impl SmallDate {
    pub fn from_str(txt: &str) -> Result<Self> {
        let s: String = txt.chars().filter(|c| c.is_ascii_digit()).collect();
        let mut n: u32 = s.parse()?;
        let day = n % 100;
        n /= 100;
        let month = n % 100;
        n /= 100;
        let year = if n > 92 { 1900 + n } else { 2000 + n };
        SmallDate::from_ymd(year, month, day)
    }

    pub fn from_system_time(t: SystemTime) -> Result<Self> {
        let duration = t.duration_since(SystemTime::UNIX_EPOCH)?;
        let secs = duration.as_secs();
        let datetime = NaiveDateTime::from_timestamp(secs as i64, 0);
        naive_to_date(datetime)
    }

    pub fn now() -> Result<Self> {
        SmallDate::from_system_time(SystemTime::now())
    }

    pub fn from_ymd(year: u32, month: u32, day: u32) -> Result<Self> {
        NaiveDate::from_ymd_opt(year as i32, month, day)
            .ok_or_else(|| DateError::Ymd(year, month, day).into())
            .map(|naive| naive.and_hms(12, 0, 0))
            .and_then(naive_to_date)
    }

    fn as_ymd(&self) -> (i32, u32, u32) {
        let days = self.0 as i32 + DAYS_FROM_CE_TO_START;
        let date = NaiveDate::from_num_days_from_ce(days);
        let year = date.year();
        let month = date.month();
        let day = date.day();
        (year, month, day)
    }
}

impl Display for SmallDate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (y, m, d) = self.as_ymd();
        write!(f, "{}{:02}{:02}", y % 100, m, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_to_ymd() -> Result<()> {
        let cases = vec![(1993, 1, 1), (2021, 1, 25), (2060, 12, 31)];

        for (y, m, d) in cases {
            let day = i32::try_from(d)?;
            let month = i32::try_from(m)? * 100;
            let year = (y % 100) * 10000;
            let digits = u32::try_from(year + month + day)?;

            let day = SmallDate::from_str(&digits.to_string())?;

            assert_eq!(day.as_ymd(), (y, m, d));
        }

        Ok(())
    }

    #[test]
    fn invalid_date() {
        let cases = vec![
            SmallDate::from_ymd(2020, 1, 32),
            SmallDate::from_str(&2.to_string()),
            SmallDate::from_str(&19700101.to_string()),
            SmallDate::from_str(&2000.to_string()),
        ];

        for day in cases {
            assert!(day.is_err());
        }
    }

    #[test]
    fn ord() -> Result<()> {
        let cases = vec![
            (2020, 12, 31, 2020, 12, 30),
            (2021, 1, 1, 2020, 12, 31),
            (2021, 2, 2, 2021, 2, 1),
            (2021, 2, 2, 1993, 2, 1),
        ];

        for (y1, m1, d1, y2, m2, d2) in cases {
            assert!(SmallDate::from_ymd(y1, m1, d1)? > SmallDate::from_ymd(y2, m2, d2)?);
        }

        Ok(())
    }

    #[test]
    fn eq() {
        assert_eq!(
            SmallDate::from_ymd(2021, 1, 31).unwrap(),
            SmallDate::from_ymd(2021, 1, 31).unwrap()
        );

        assert_ne!(
            SmallDate::from_ymd(2021, 1, 31).unwrap(),
            SmallDate::from_ymd(2021, 1, 30).unwrap()
        );
    }

    #[test]
    fn too_far_in_the_past() {
        assert!(SmallDate::from_ymd(1800, 1, 1).is_err())
    }

    #[test]
    fn too_far_in_the_future() {
        assert!(SmallDate::from_ymd(4000, 1, 1).is_err())
    }

    #[test]
    fn to_str() {
        let cases = vec![(2020, 1, 4, "200104"), (1993, 1, 4, "930104")];
        for (y, m, d, txt) in cases {
            assert_eq!(txt, &SmallDate::from_ymd(y, m, d).unwrap().to_string());
        }
    }
}
