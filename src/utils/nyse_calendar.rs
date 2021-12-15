use bdays::{easter::easter_naive_date, HolidayCalendar};
use chrono::{Datelike, Duration, NaiveDate, Weekday};

fn end_of_month(mut yy: i32, mut mm: u32) -> NaiveDate {
    assert!(mm <= 12);

    if mm == 12 {
        yy += 1;
        mm = 1;
    } else {
        mm += 1;
    }

    NaiveDate::from_ymd(yy, mm, 1).pred()
}

fn find_weekday_ascending(weekday: Weekday, yy: i32, mm: u32, occurrence: u32) -> NaiveDate {
    let anchor = NaiveDate::from_ymd(yy, mm, 1);
    let mut offset = (weekday.number_from_monday() + 7 - anchor.weekday().number_from_monday()) % 7;

    if occurrence > 1 {
        offset += 7 * (occurrence - 1);
    }

    anchor + Duration::days(offset as i64)
}

fn find_weekday_descending(weekday: Weekday, yy: i32, mm: u32, occurrence: u32) -> NaiveDate {
    let anchor = end_of_month(yy, mm);
    let mut offset = (anchor.weekday().number_from_monday() + 7 - weekday.number_from_monday()) % 7;

    if occurrence > 1 {
        offset += 7 * (occurrence - 1);
    }

    anchor - Duration::days(offset as i64)
}

fn find_weekday(weekday: Weekday, yy: i32, mm: u32, occurrence: u32, ascending: bool) -> NaiveDate {
    if ascending {
        find_weekday_ascending(weekday, yy, mm, occurrence)
    } else {
        find_weekday_descending(weekday, yy, mm, occurrence)
    }
}

/// In the United States, if a holiday falls on Saturday, it's observed on the preceding Friday.
/// If it falls on Sunday, it's observed on the next Monday.
fn adjust_weekend_holidays_us(date: NaiveDate) -> NaiveDate {
    match date.weekday() {
        Weekday::Sat => date - Duration::days(1),
        Weekday::Sun => date + Duration::days(1),
        _ => date,
    }
}

pub struct NyseCalendar;

impl<T: Datelike + Copy + PartialOrd> HolidayCalendar<T> for NyseCalendar {
    fn is_holiday(&self, date: T) -> bool {
        let (yy, mm, dd) = (date.year(), date.month(), date.day());
        let dt_naive = NaiveDate::from_ymd(yy, mm, dd);

        // New Year's Day
        if adjust_weekend_holidays_us(NaiveDate::from_ymd(yy, 1, 1)) == dt_naive {
            return true;
        }

        // Birthday of Martin Luther King, Jr.
        if yy >= 1998
            && adjust_weekend_holidays_us(find_weekday(Weekday::Mon, yy, 1, 3, true)) == dt_naive
        {
            return true;
        }

        // Washington's Birthday
        if adjust_weekend_holidays_us(find_weekday(Weekday::Mon, yy, 2, 3, true)) == dt_naive {
            return true;
        }

        // Good Friday
        let easter = easter_naive_date(yy).unwrap();
        if (easter - Duration::days(2)) == dt_naive {
            return true;
        }

        // Memorial Day
        if adjust_weekend_holidays_us(find_weekday(Weekday::Mon, yy, 5, 1, false)) == dt_naive {
            return true;
        }

        // Juneteenth
        if yy >= 2022 && adjust_weekend_holidays_us(NaiveDate::from_ymd(yy, 6, 19)) == dt_naive {
            return true;
        }

        // Independence Day
        if adjust_weekend_holidays_us(NaiveDate::from_ymd(yy, 7, 4)) == dt_naive {
            return true;
        }

        // Labor Day
        if adjust_weekend_holidays_us(find_weekday(Weekday::Mon, yy, 9, 1, true)) == dt_naive {
            return true;
        }

        // Thanksgiving Day
        if adjust_weekend_holidays_us(find_weekday(Weekday::Thu, yy, 11, 4, true)) == dt_naive {
            return true;
        }

        // Christmas
        if adjust_weekend_holidays_us(NaiveDate::from_ymd(yy, 12, 25)) == dt_naive {
            return true;
        }

        // Presidential election days
        if (yy <= 1968 || (yy <= 1980 && yy % 4 == 0))
            && mm == 11
            && dd <= 7
            && dt_naive.weekday() == Weekday::Tue
        {
            return true;
        }

        // Special closures
        let special_closures = [
            // George H.W. Bush's funeral
            NaiveDate::from_ymd(2018, 12, 5),
            // Hurrican Sandy
            NaiveDate::from_ymd(2012, 10, 29),
            NaiveDate::from_ymd(2012, 10, 30),
            // President Ford's funeral
            NaiveDate::from_ymd(2007, 1, 2),
            // 9/11
            NaiveDate::from_ymd(2001, 9, 11),
            NaiveDate::from_ymd(2001, 9, 12),
            NaiveDate::from_ymd(2001, 9, 13),
            NaiveDate::from_ymd(2001, 9, 14),
            // President Nixon's funeral
            NaiveDate::from_ymd(1994, 4, 27),
            // Hurrican Gloria
            NaiveDate::from_ymd(1985, 9, 27),
            // 1977 Blackout
            NaiveDate::from_ymd(1977, 7, 14),
            // President Johnson's funeral
            NaiveDate::from_ymd(1973, 1, 25),
            // President Truman's funeral
            NaiveDate::from_ymd(1972, 12, 25),
            // Moon landing
            NaiveDate::from_ymd(1969, 7, 21),
            // President Eisenhower's funeral
            NaiveDate::from_ymd(1969, 3, 31),
            // Heavy snow
            NaiveDate::from_ymd(1969, 2, 10),
            // Day after Independence day
            NaiveDate::from_ymd(1968, 7, 5),
            // Paperwork crisis
            NaiveDate::from_ymd(1968, 6, 12),
            NaiveDate::from_ymd(1968, 6, 19),
            NaiveDate::from_ymd(1968, 6, 26),
            NaiveDate::from_ymd(1968, 7, 3),
            NaiveDate::from_ymd(1968, 7, 10),
            NaiveDate::from_ymd(1968, 7, 17),
            NaiveDate::from_ymd(1968, 7, 24),
            NaiveDate::from_ymd(1968, 7, 31),
            NaiveDate::from_ymd(1968, 8, 7),
            NaiveDate::from_ymd(1968, 8, 14),
            NaiveDate::from_ymd(1968, 8, 21),
            NaiveDate::from_ymd(1968, 8, 28),
            NaiveDate::from_ymd(1968, 9, 4),
            NaiveDate::from_ymd(1968, 9, 11),
            NaiveDate::from_ymd(1968, 9, 18),
            NaiveDate::from_ymd(1968, 9, 25),
            NaiveDate::from_ymd(1968, 10, 2),
            NaiveDate::from_ymd(1968, 10, 9),
            NaiveDate::from_ymd(1968, 10, 16),
            NaiveDate::from_ymd(1968, 10, 23),
            NaiveDate::from_ymd(1968, 10, 30),
            NaiveDate::from_ymd(1968, 11, 6),
            NaiveDate::from_ymd(1968, 11, 13),
            NaiveDate::from_ymd(1968, 11, 20),
            NaiveDate::from_ymd(1968, 11, 27),
            NaiveDate::from_ymd(1968, 12, 4),
            NaiveDate::from_ymd(1968, 12, 11),
            NaiveDate::from_ymd(1968, 12, 18),
            NaiveDate::from_ymd(1968, 12, 25),
            // MLK assassination
            NaiveDate::from_ymd(1968, 4, 9),
            // President Kennedy's funeral
            NaiveDate::from_ymd(1963, 11, 25),
            // Day before Decoration day
            NaiveDate::from_ymd(1961, 5, 29),
            // Day after Christmas
            NaiveDate::from_ymd(1958, 12, 26),
            // Christmas eve
            NaiveDate::from_ymd(1965, 12, 24),
            NaiveDate::from_ymd(1956, 12, 24),
            NaiveDate::from_ymd(1954, 12, 24),
        ];

        if special_closures.contains(&dt_naive) {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod test {
    // https://www.nyse.com/markets/hours-calendars
    use super::*;

    const CAL: NyseCalendar = NyseCalendar;
    lazy_static! {
     static ref HOLIDAYS: [NaiveDate; 28] = [
         // New Years
         NaiveDate::from_ymd(2021, 1, 1),
         NaiveDate::from_ymd(2023, 1, 2),
         // MLK
         NaiveDate::from_ymd(2021, 1, 18),
         NaiveDate::from_ymd(2022, 1, 17),
         NaiveDate::from_ymd(2023, 1, 16),
         // Presidents' day
         NaiveDate::from_ymd(2021, 2, 15),
         NaiveDate::from_ymd(2022, 2, 21),
         NaiveDate::from_ymd(2023, 2, 20),
         // Good Friday
         NaiveDate::from_ymd(2021, 4, 2),
         NaiveDate::from_ymd(2022, 4, 15),
         NaiveDate::from_ymd(2023, 4, 7),
         // Memorial Day
         NaiveDate::from_ymd(2021, 5, 31),
         NaiveDate::from_ymd(2022, 5, 30),
         NaiveDate::from_ymd(2023, 5, 29),
         // Juneteenth
         NaiveDate::from_ymd(2022, 6, 20),
         NaiveDate::from_ymd(2023, 6, 19),
         // Independence Day
         NaiveDate::from_ymd(2021, 7, 5),
         NaiveDate::from_ymd(2022, 7, 4),
         NaiveDate::from_ymd(2023, 7, 4),
         // Labor day
         NaiveDate::from_ymd(2021, 9, 6),
         NaiveDate::from_ymd(2022, 9, 5),
         NaiveDate::from_ymd(2023, 9, 4),
         // Thanksgiving
         NaiveDate::from_ymd(2021, 11, 25),
         NaiveDate::from_ymd(2022, 11, 24),
         NaiveDate::from_ymd(2023, 11, 23),
         // Christmas Day
         NaiveDate::from_ymd(2021, 12, 24),
         NaiveDate::from_ymd(2022, 12, 26),
         NaiveDate::from_ymd(2023, 12, 25),
     ];
    }

    #[test]
    fn holidays() {
        let mut date = NaiveDate::from_ymd(2021, 1, 1);
        while date < NaiveDate::from_ymd(2024, 1, 1) {
            if HOLIDAYS.contains(&date) {
                assert!(CAL.is_holiday(date), "{} is not a holiday", date)
            } else {
                assert!(!CAL.is_holiday(date), "{} is a holiday", date)
            }
            date += Duration::days(1)
        }
    }
}
