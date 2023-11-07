/*
 * DMNTK - Decision Model and Notation Toolkit
 *
 * MIT license
 *
 * Copyright (c) 2015-2023 Dariusz Depta
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * Apache license, Version 2.0
 *
 * Copyright (c) 2015-2023 Dariusz Depta
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Implementation of FEEL date.

use crate::defs::*;
use crate::errors::*;
use crate::feel_ym_duration::FeelYearsAndMonthsDuration;
use crate::FeelDaysAndTimeDuration;
use chrono::{
    DateTime, Datelike, Days, FixedOffset, Local, LocalResult, Months, NaiveDate, TimeZone, Weekday,
};
use dmntk_common_1::DmntkError;
use dmntk_feel_number::FeelNumber;
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;

/// FEEL date.
#[derive(Debug, Clone)]
pub struct FeelDate(Year, Month, Day);

impl fmt::Display for FeelDate {
    /// Converts [FeelDate] into [String].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.0, self.1, self.2)
    }
}

impl FromStr for FeelDate {
    type Err = DmntkError;
    /// Converts [String] into [FeelDate].
    fn from_str(date: &str) -> Result<Self, Self::Err> {
        let captures = RE_DATE
            .captures(date)
            .ok_or_else(|| err_invalid_date_literal(date))?;
        if let Some(year_match) = captures.name("year") {
            if let Ok(mut year) = year_match.as_str().parse::<Year>() {
                if captures.name("sign").is_some() {
                    year = -year;
                }
                if let Some(month_match) = captures.name("month") {
                    if let Ok(month) = month_match.as_str().parse::<Month>() {
                        if let Some(day_match) = captures.name("day") {
                            if let Ok(day) = day_match.as_str().parse::<Day>() {
                                if is_valid_date(year, month, day) {
                                    return Ok(FeelDate(year, month, day));
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err_invalid_date_literal(date))
    }
}

impl TryFrom<(FeelNumber, FeelNumber, FeelNumber)> for FeelDate {
    type Error = DmntkError;
    /// Converts a tuple of numbers into [FeelDate].
    fn try_from(value: (FeelNumber, FeelNumber, FeelNumber)) -> Result<Self, Self::Error> {
        let year = value.0.try_into()?;
        if value.1 > FeelNumber::zero() && value.2 > FeelNumber::zero() {
            let month = value.1.try_into()?;
            let day = value.2.try_into()?;
            if is_valid_date(year, month, day) {
                return Ok(Self(year, month, day));
            }
        }
        Err(err_invalid_date(value.0, value.1, value.2))
    }
}

impl PartialEq for FeelDate {
    ///
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl Eq for FeelDate {}

impl PartialOrd for FeelDate {
    ///
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FeelDate {
    ///
    fn cmp(&self, other: &Self) -> Ordering {
        let y = self.0.cmp(&other.0);
        let m = self.1.cmp(&other.1);
        let d = self.2.cmp(&other.2);
        match (y, m, d) {
            (Ordering::Equal, Ordering::Equal, Ordering::Equal) => Ordering::Equal,
            (Ordering::Equal, Ordering::Equal, Ordering::Less) => Ordering::Less,
            (Ordering::Equal, Ordering::Equal, Ordering::Greater) => Ordering::Greater,
            (Ordering::Equal, Ordering::Less, _) => Ordering::Less,
            (Ordering::Equal, Ordering::Greater, _) => Ordering::Greater,
            (Ordering::Less, _, _) => Ordering::Less,
            (Ordering::Greater, _, _) => Ordering::Greater,
        }
    }
}

impl Sub<&FeelDate> for &FeelDate {
    type Output = FeelYearsAndMonthsDuration;
    /// Subtracts two [FeelDates](FeelDate), the result is [FeelYearsAndMonthsDuration].
    fn sub(self, other: &FeelDate) -> Self::Output {
        let mut months;
        if self.0 < other.0 {
            months = 12 * (other.0 as i64 - self.0 as i64) + (other.1 as i64 - self.1 as i64);
            if self.2 > other.2 {
                months -= 1;
            }
            months *= -1;
        } else {
            months = 12 * (self.0 as i64 - other.0 as i64) + (self.1 as i64 - other.1 as i64);
            if other.2 > self.2 {
                months -= 1;
            }
        }
        FeelYearsAndMonthsDuration::from_m(months)
    }
}

impl Add<FeelYearsAndMonthsDuration> for FeelDate {
    type Output = Option<Self>;
    /// Adds [FeelYearsAndMonthsDuration] to [FeelDate].
    fn add(self, other: FeelYearsAndMonthsDuration) -> Self::Output {
        if other.is_negative() {
            return self.sub(other.abs());
        }
        if let Ok(months) = other.as_months().try_into() {
            return self.add_months(months);
        }
        None
    }
}

impl Sub<FeelYearsAndMonthsDuration> for FeelDate {
    type Output = Option<Self>;
    /// Subtracts [FeelYearsAndMonthsDuration] from [FeelDate].
    fn sub(self, other: FeelYearsAndMonthsDuration) -> Self::Output {
        if other.is_negative() {
            return self.add(other.abs());
        }
        if let Ok(months) = other.as_months().try_into() {
            return self.sub_months(months);
        }
        None
    }
}

impl Add<FeelDaysAndTimeDuration> for FeelDate {
    type Output = Option<FeelDate>;
    /// Adds [FeelDaysAndTimeDuration] to [FeelDate].
    fn add(self, rhs: FeelDaysAndTimeDuration) -> Self::Output {
        if rhs.is_negative() {
            return self.sub(rhs.abs());
        }
        let duration_seconds = rhs.get_seconds();
        let carry_minutes = duration_seconds / 60;
        let duration_minutes = rhs.get_minutes() + carry_minutes;
        let carry_hours = duration_minutes / 60;
        let duration_hours = rhs.get_hours() + carry_hours;
        let carry_days = duration_hours / 60;
        let days = rhs.get_days() + carry_days;
        if let Some(date) = NaiveDate::from_ymd_opt(self.0, self.1, self.2) {
            if let Some(new_date) = date.checked_add_days(Days::new(days as u64)) {
                return Some(FeelDate(new_date.year(), new_date.month(), new_date.day()));
            }
        }
        None
    }
}

impl Sub<FeelDaysAndTimeDuration> for FeelDate {
    type Output = Option<FeelDate>;
    /// Subtracts [FeelDaysAndTimeDuration] from [FeelDate].
    fn sub(self, rhs: FeelDaysAndTimeDuration) -> Self::Output {
        if rhs.is_negative() {
            return self.add(rhs.abs());
        }
        let duration_seconds = rhs.get_seconds();
        let carry_minutes = duration_seconds / 60;
        let remainder_seconds = duration_seconds % 60;
        let duration_minutes = rhs.get_minutes() + carry_minutes;
        let carry_hours = duration_minutes / 60;
        let remainder_minutes = duration_minutes % 60;
        let duration_hours = rhs.get_hours() + carry_hours;
        let carry_days = duration_hours / 60;
        let remainder_hours = duration_hours % 60;
        let mut days = rhs.get_days() + carry_days;
        if let Some(date) = NaiveDate::from_ymd_opt(self.0, self.1, self.2) {
            if remainder_hours > 0 || remainder_minutes > 0 || remainder_seconds > 0 {
                days += 1;
            }
            if let Some(new_date) = date.checked_sub_days(Days::new(days as u64)) {
                return Some(FeelDate(new_date.year(), new_date.month(), new_date.day()));
            }
        }
        None
    }
}

impl TryFrom<FeelDate> for DateTime<FixedOffset> {
    type Error = DmntkError;
    /// Converts [FeelDate] date into [DateTime] in UTC zone.
    fn try_from(date: FeelDate) -> Result<Self, Self::Error> {
        if let Some(fixed_offset) = FixedOffset::east_opt(0) {
            if let LocalResult::Single(date_time) =
                fixed_offset.with_ymd_and_hms(date.0, date.1, date.2, 0, 0, 0)
            {
                return Ok(date_time);
            }
        }
        Err(err_invalid_feel_date(date))
    }
}

impl TryFrom<FeelDate> for NaiveDate {
    type Error = DmntkError;
    /// Converts [FeelDate] date into [NaiveDate].
    fn try_from(date: FeelDate) -> Result<Self, Self::Error> {
        NaiveDate::from_ymd_opt(date.0, date.1, date.2).ok_or_else(|| err_invalid_feel_date(date))
    }
}

impl FeelDate {
    ///
    //TODO this constructor may create invalid date - remove and use new_opt instead.
    pub fn new(year: Year, month: Month, day: Day) -> Self {
        Self(year, month, day)
    }

    ///
    pub fn new_opt(year: Year, month: Month, day: Day) -> Option<Self> {
        if is_valid_date(year, month, day) {
            Some(Self(year, month, day))
        } else {
            None
        }
    }

    /// Returns [FeelDate] set to today's date in local time.
    pub fn today() -> Self {
        let today = Local::now();
        Self(today.year(), today.month(), today.day())
    }

    ///
    pub fn year(&self) -> Year {
        self.0
    }

    ///
    pub fn month(&self) -> Month {
        self.1
    }

    ///
    pub fn day(&self) -> Day {
        self.2
    }

    ///
    pub fn day_of_week(&self) -> Option<DayOfWeek> {
        NaiveDate::from_ymd_opt(self.0, self.1, self.2).map(|naive_date| {
            match naive_date.weekday() {
                Weekday::Mon => ("Monday".to_string(), 1_u8),
                Weekday::Tue => ("Tuesday".to_string(), 2_u8),
                Weekday::Wed => ("Wednesday".to_string(), 3_u8),
                Weekday::Thu => ("Thursday".to_string(), 4_u8),
                Weekday::Fri => ("Friday".to_string(), 5_u8),
                Weekday::Sat => ("Saturday".to_string(), 6_u8),
                Weekday::Sun => ("Sunday".to_string(), 7_u8),
            }
        })
    }

    ///
    pub fn day_of_year(&self) -> Option<DayOfYear> {
        NaiveDate::from_ymd_opt(self.0, self.1, self.2)
            .map(|naive_date| naive_date.ordinal() as u16)
    }

    ///
    pub fn week_of_year(&self) -> Option<WeekOfYear> {
        NaiveDate::from_ymd_opt(self.0, self.1, self.2)
            .map(|naive_date| naive_date.iso_week().week() as u8)
    }

    ///
    pub fn month_of_year(&self) -> Option<MonthOfYear> {
        if let Some(naive_date) = NaiveDate::from_ymd_opt(self.0, self.1, self.2) {
            month_of_year(naive_date.month())
        } else {
            None
        }
    }

    ///
    pub fn as_tuple(&self) -> (Year, Month, Day) {
        (self.0, self.1, self.2)
    }

    ///
    pub fn add_days(&self, days: u64) -> Option<Self> {
        if let Some(naive_date) = NaiveDate::from_ymd_opt(self.0, self.1, self.2) {
            if let Some(updated_date) = naive_date.checked_add_days(Days::new(days)) {
                return Some(Self(
                    updated_date.year(),
                    updated_date.month(),
                    updated_date.day(),
                ));
            }
        }
        None
    }

    ///
    pub fn add_months(&self, months: u32) -> Option<Self> {
        if let Some(naive_date) = NaiveDate::from_ymd_opt(self.0, self.1, self.2) {
            if let Some(updated_date) = naive_date.checked_add_months(Months::new(months)) {
                return Some(Self(
                    updated_date.year(),
                    updated_date.month(),
                    updated_date.day(),
                ));
            }
        }
        None
    }

    ///
    pub fn sub_months(&self, months: u32) -> Option<Self> {
        if let Some(naive_date) = NaiveDate::from_ymd_opt(self.0, self.1, self.2) {
            if let Some(updated_date) = naive_date.checked_sub_months(Months::new(months)) {
                return Some(Self(
                    updated_date.year(),
                    updated_date.month(),
                    updated_date.day(),
                ));
            }
        }
        None
    }
}

/// Returns `true` when specified year, month and day form a valid [FeelDate].
pub fn is_valid_date(year: Year, month: Month, day: Day) -> bool {
    if (-999_999_999..=999_999_999).contains(&year) {
        if let Some(last_day_of_month) = last_day_of_month(year, month) {
            return (1..=last_day_of_month).contains(&day);
        }
    }
    false
}

/// Returns `true` when the specified year is a leap year.
pub fn is_leap_year(year: Year) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Returns the last dau of month in specified (leap) year.
pub fn last_day_of_month(year: Year, month: Month) -> Option<Day> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if is_leap_year(year) { 29 } else { 28 }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dmntk_common_1::Result;

    #[test]
    fn test_display() {
        assert_eq!("9999-01-01", format!("{}", FeelDate(9999, 1, 1)));
        assert_eq!("-9999-01-01", format!("{}", FeelDate(-9999, 1, 1)));
        assert_eq!(
            "999999999-01-01",
            format!("{}", FeelDate(999_999_999, 1, 1))
        );
        assert_eq!(
            "-999999999-01-01",
            format!("{}", FeelDate(-999_999_999, 1, 1))
        );
    }

    #[test]
    fn test_add_months() {
        assert_eq!(
            "9999-02-01",
            format!("{}", FeelDate(9999, 1, 1).add_months(1).unwrap())
        );
        assert_eq!(
            "99999-02-01",
            format!("{}", FeelDate(99999, 1, 1).add_months(1).unwrap())
        );
        assert_eq!(
            "99999-12-01",
            format!("{}", FeelDate(99999, 1, 1).add_months(11).unwrap())
        );
        assert_eq!(
            "100000-01-01",
            format!("{}", FeelDate(99999, 1, 1).add_months(12).unwrap())
        );
        assert_eq!(
            "199999-02-01",
            format!("{}", FeelDate(199999, 1, 1).add_months(1).unwrap())
        );
        assert_eq!(
            "262143-02-01",
            format!("{}", FeelDate(262143, 1, 1).add_months(1).unwrap())
        );
        assert_eq!("+262143-12-31", NaiveDate::MAX.to_string())
    }

    #[test]
    fn test_sub_months() {
        assert_eq!(
            "-9999-11-01",
            format!("{}", FeelDate(-9999, 12, 1).sub_months(1).unwrap())
        );
        assert_eq!(
            "-99999-02-01",
            format!("{}", FeelDate(-99999, 3, 1).sub_months(1).unwrap())
        );
        assert_eq!(
            "-99999-12-01",
            format!("{}", FeelDate(-99998, 1, 1).sub_months(1).unwrap())
        );
        assert_eq!(
            "-100000-01-01",
            format!("{}", FeelDate(-100000, 12, 1).sub_months(11).unwrap())
        );
        assert_eq!(
            "-199999-11-01",
            format!("{}", FeelDate(-199999, 12, 1).sub_months(1).unwrap())
        );
        assert_eq!(
            "-262144-01-01",
            format!("{}", FeelDate(-262144, 2, 1).sub_months(1).unwrap())
        );
        assert_eq!("-262144-01-01", NaiveDate::MIN.to_string())
    }

    #[test]
    fn test_is_valid_date() {
        assert!(is_valid_date(999_999_999, 12, 13));
        assert!(!is_valid_date(1_000_000_000, 1, 1));
        assert!(is_valid_date(-999_999_999, 1, 1));
        assert!(!is_valid_date(-1_000_000_000, 12, 31));
        assert!(!is_valid_date(2021, 2, 29));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(!is_leap_year(2500));
        assert!(is_leap_year(2400));
        assert!(!is_leap_year(2300));
        assert!(!is_leap_year(2200));
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(1800));
    }

    #[test]
    fn test_last_day_of_month() {
        assert_eq!(31, last_day_of_month(2021, 1).unwrap());
        assert_eq!(28, last_day_of_month(2021, 2).unwrap());
        assert_eq!(31, last_day_of_month(2021, 3).unwrap());
        assert_eq!(30, last_day_of_month(2021, 4).unwrap());
        assert_eq!(31, last_day_of_month(2021, 5).unwrap());
        assert_eq!(30, last_day_of_month(2021, 6).unwrap());
        assert_eq!(31, last_day_of_month(2021, 7).unwrap());
        assert_eq!(31, last_day_of_month(2021, 8).unwrap());
        assert_eq!(30, last_day_of_month(2021, 9).unwrap());
        assert_eq!(31, last_day_of_month(2021, 10).unwrap());
        assert_eq!(30, last_day_of_month(2021, 11).unwrap());
        assert_eq!(31, last_day_of_month(2021, 12).unwrap());
        assert_eq!(None, last_day_of_month(2021, 13));
        assert_eq!(None, last_day_of_month(2021, 0));
    }

    #[test]
    fn test_last_day_of_month_leap_year() {
        assert_eq!(31, last_day_of_month(2020, 1).unwrap());
        assert_eq!(29, last_day_of_month(2020, 2).unwrap());
        assert_eq!(31, last_day_of_month(2020, 3).unwrap());
        assert_eq!(30, last_day_of_month(2020, 4).unwrap());
        assert_eq!(31, last_day_of_month(2020, 5).unwrap());
        assert_eq!(30, last_day_of_month(2020, 6).unwrap());
        assert_eq!(31, last_day_of_month(2020, 7).unwrap());
        assert_eq!(31, last_day_of_month(2020, 8).unwrap());
        assert_eq!(30, last_day_of_month(2020, 9).unwrap());
        assert_eq!(31, last_day_of_month(2020, 10).unwrap());
        assert_eq!(30, last_day_of_month(2020, 11).unwrap());
        assert_eq!(31, last_day_of_month(2020, 12).unwrap());
        assert_eq!(None, last_day_of_month(2020, 13));
        assert_eq!(None, last_day_of_month(2020, 0));
    }

    #[test]
    fn test_eq() {
        assert!((FeelDate(2021, 2, 1) == FeelDate(2021, 2, 1)));
        assert!((FeelDate(2021, 2, 1) != FeelDate(2021, 2, 2)));
        assert!((FeelDate(2021, 2, 1) != FeelDate(2021, 3, 1)));
        assert!((FeelDate(2021, 2, 1) != FeelDate(2022, 2, 1)));
        assert!((FeelDate(999_999_999, 12, 31) != FeelDate(-999_999_999, 1, 1)));
    }

    #[test]
    fn test_compare() {
        assert!((FeelDate(2021, 2, 1) == FeelDate(2021, 2, 1)));
        assert!((FeelDate(2021, 2, 1) < FeelDate(2021, 2, 2)));
        assert!((FeelDate(2021, 2, 1) < FeelDate(2021, 3, 1)));
        assert!((FeelDate(2021, 2, 5) < FeelDate(2022, 2, 5)));
        assert!((FeelDate(2021, 2, 2) > FeelDate(2021, 2, 1)));
        assert!((FeelDate(2021, 3, 1) > FeelDate(2021, 2, 1)));
        assert!((FeelDate(2022, 2, 1) > FeelDate(2021, 2, 1)));
        assert!((FeelDate(2021, 2, 1) >= FeelDate(2021, 2, 1)));
        assert!((FeelDate(2021, 2, 2) >= FeelDate(2021, 2, 1)));
        assert!((FeelDate(2021, 2, 1) <= FeelDate(2021, 2, 1)));
        assert!((FeelDate(2021, 2, 1) <= FeelDate(2021, 2, 2)));
    }

    #[test]
    fn test_conversion() {
        let date = FeelDate(2023, 2, 8);
        let date_time: DateTime<FixedOffset> = date.try_into().unwrap();
        assert_eq!("2023-02-08 00:00:00 +00:00", date_time.to_string());
        let date = FeelDate(262144, 2, 8);
        let date_time: Result<DateTime<FixedOffset>> = date.try_into();
        assert_eq!(
            "<TemporalError> invalid date 262144-2-8",
            date_time.err().unwrap().to_string()
        );
        let date = FeelDate(262144, 2, 8);
        let date_time: Result<NaiveDate> = date.try_into();
        assert_eq!(
            "<TemporalError> invalid date 262144-2-8",
            date_time.err().unwrap().to_string()
        );
    }
}
