use std::time;

type Int = i64;

const SECS_IN_MIN: Int = 60;
const MINS_IN_HOUR: Int = 60;
const SECS_IN_HOUR: Int = SECS_IN_MIN * MINS_IN_HOUR;
const HOURS_IN_DAY: Int = 24;
const DAYS_IN_YEAR: Int = 365;
const UNIX_START_YEAR: Int = 1970;
const MONTH_TO_DAYS: [Int; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTHS_IN_YEAR: Int = MONTH_TO_DAYS.len() as Int;
const SECS_IN_DAY: Int = SECS_IN_MIN * MINS_IN_HOUR * HOURS_IN_DAY;
const SECS_IN_WEEK: Int = SECS_IN_DAY * 7;
const DAYS: [Int; 7] = [0, 1, 2, 3, 4, 5, 6]; // Mon -> Sun
const DAYS_IN_WEEK: Int = DAYS.len() as Int;
const MON_YDAY: [[Int; 13]; 2] = [
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365], // Normal years
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366], // Leap years
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
/// super basic Date (with Time, even if you can ignore it)
///
/// this is fine for working with a limited amount of dates,
/// as there is no optimization for now,
/// especially in the creation logic
pub struct DateTime {
    year: Int,
    month: Int,
    day: Int,
    hour: Int,
    minute: Int,
    second: Int,
}

macro_rules! impl_getter {
    ($field:ident) => {
        pub const fn $field(&self) -> Int {
            self.$field
        }
    };
}

impl DateTime {
    impl_getter!(year);
    impl_getter!(month);
    impl_getter!(day);
    impl_getter!(hour);
    impl_getter!(minute);
    impl_getter!(second);

    /// if you don't care about the time part, that will be zeroed
    pub const fn new_date(year: Int, month: Int, day: Int) -> Option<Self> {
        Self::new(year, month, day, 0, 0, 0)
    }

    pub const fn new(
        year: Int,
        month: Int,
        day: Int,
        hour: Int,
        minute: Int,
        second: Int,
    ) -> Option<Self> {
        let out = Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        };
        if out.is_valid() { Some(out) } else { None }
    }

    /// date as in "seconds since UNIX_EPOCH"
    pub fn now() -> Option<Self> {
        let secs = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("duration since UNIX_EPOCH is always defined")
            .as_secs() as Int;
        Self::from_anchor(secs)
    }

    fn as_anchor(&self) -> Int {
        let mut secs = self.second;
        secs += self.minute * SECS_IN_MIN;
        secs += self.hour * SECS_IN_MIN * MINS_IN_HOUR;
        secs += SECS_IN_DAY * self.as_days();
        secs - SECS_IN_DAY
    }

    /// num of full days since anchor, including current one
    fn as_days(&self) -> Int {
        self.day
            + (1..self.month)
                .map(|month| month_days(month, self.year))
                .chain((UNIX_START_YEAR..self.year).map(year_days))
                .sum::<Int>()
    }

    const fn from_anchor(secs: Int) -> Option<Self> {
        // copied from gnu c stdlib (2x faster than mine)
        let (mut days, mut rem) = divmod(secs, SECS_IN_DAY);
        while rem < 0 {
            rem += SECS_IN_DAY;
            days -= 1;
        }
        while rem >= SECS_IN_DAY {
            rem -= SECS_IN_DAY;
            days += 1;
        }
        let hour = rem / SECS_IN_HOUR;
        rem %= SECS_IN_HOUR;
        let (minute, second) = divmod(rem, SECS_IN_MIN);
        let mut y: Int = UNIX_START_YEAR;
        while days < 0 || days >= year_days(y) {
            // Guess a corrected year, assuming 365 days per year
            let yg = y + days / DAYS_IN_YEAR - ((days % DAYS_IN_YEAR < 0) as Int);
            // Adjust DAYS and Y to match the guessed year
            days -= (yg - y) * DAYS_IN_YEAR + leaps_thru_end_of(yg - 1) - leaps_thru_end_of(y - 1);
            y = yg;
        }
        let year = y;
        let ip = MON_YDAY[is_leap(year) as usize];
        y = MONTHS_IN_YEAR - 1;
        while days < ip[y as usize] {
            y -= 1;
        }
        days -= ip[y as usize];
        let month = y + 1;
        let day = days + 1;
        Self::new(year, month, day, hour, minute, second)
    }

    pub const fn asarray(&self) -> [Int; 6] {
        [
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        ]
    }

    pub fn str_date(&self) -> String {
        format!(
            "{}-{}-{}",
            self.year,
            fmt_int(self.month),
            fmt_int(self.day)
        )
    }

    pub fn str_full(&self) -> String {
        format!(
            "{}-{}-{} {}:{}:{}",
            self.year,
            fmt_int(self.month),
            fmt_int(self.day),
            fmt_int(self.hour),
            fmt_int(self.minute),
            fmt_int(self.second)
        )
    }

    /// how many days in the current month
    pub const fn month_days(&self) -> Int {
        month_days(self.month, self.year)
    }

    /// how many days in the current year
    pub const fn year_days(&self) -> Int {
        year_days(self.year)
    }

    pub const fn is_leap_year(&self) -> bool {
        is_leap(self.year)
    }

    pub fn weekday(&self) -> Int {
        const DAY_ON_UNIX_START: Int = 3;
        (((self.as_days() - 1) % DAYS_IN_WEEK) + DAY_ON_UNIX_START) % DAYS_IN_WEEK
    }

    pub fn prev_day(&self) -> Option<Self> {
        Self::from_anchor(self.as_anchor() - SECS_IN_DAY)
    }

    pub fn next_day(&self) -> Option<Self> {
        Self::from_anchor(self.as_anchor() + SECS_IN_DAY)
    }

    pub fn prev_week(&self) -> Option<Self> {
        Self::from_anchor(self.as_anchor() - SECS_IN_WEEK)
    }

    pub fn next_week(&self) -> Option<Self> {
        Self::from_anchor(self.as_anchor() + SECS_IN_WEEK)
    }

    pub fn prev_month(&self) -> Option<Self> {
        if self.month == 1 {
            return Self::new(
                self.year - 1,
                12,
                self.day,
                self.hour,
                self.minute,
                self.second,
            );
        }
        for offset in 0..4 {
            let out = Self::new(
                self.year,
                self.month - 1,
                self.day - offset,
                self.hour,
                self.minute,
                self.second,
            );
            if out.is_some() {
                return out;
            }
        }
        unreachable!("all months have at least 28 days")
    }

    pub fn next_month(&self) -> Option<Self> {
        if self.month == 12 {
            return Self::new(
                self.year + 1,
                1,
                self.day,
                self.hour,
                self.minute,
                self.second,
            );
        }
        for offset in 0..4 {
            let out = Self::new(
                self.year,
                self.month + 1,
                self.day - offset,
                self.hour,
                self.minute,
                self.second,
            );
            if out.is_some() {
                return out;
            }
        }
        unreachable!("all months have at least 28 days")
    }

    pub const fn prev_year(&self) -> Option<Self> {
        let out = Self::new(
            self.year - 1,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        );
        if out.is_some() {
            out
        } else {
            Self::new(
                self.year - 1,
                self.month,
                self.day - 1,
                self.hour,
                self.minute,
                self.second,
            )
        }
    }

    pub const fn next_year(&self) -> Option<Self> {
        let out = Self::new(
            self.year + 1,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        );
        if out.is_some() {
            out
        } else {
            Self::new(
                self.year + 1,
                self.month,
                self.day - 1,
                self.hour,
                self.minute,
                self.second,
            )
        }
    }

    pub const fn quarter(&self) -> Int {
        1 + (self.month - 1) / 3
    }

    pub const fn semester(&self) -> Int {
        1 + ((self.month > 6) as Int)
    }

    pub const fn start_of_month(&self) -> Option<Self> {
        Self::new(
            self.year,
            self.month,
            1,
            self.hour,
            self.minute,
            self.second,
        )
    }

    pub fn end_of_month(&self) -> Option<Self> {
        for day in (28..=31).rev() {
            let out = Self::new(
                self.year,
                self.month,
                day,
                self.hour,
                self.minute,
                self.second,
            );
            if out.is_some() {
                return out;
            }
        }
        unreachable!("all months have at least 28 days")
    }

    pub const fn start_of_year(&self) -> Option<Self> {
        Self::new(self.year, 1, 1, self.hour, self.minute, self.second)
    }

    pub const fn end_of_year(&self) -> Option<Self> {
        Self::new(self.year, 12, 31, self.hour, self.minute, self.second)
    }

    pub fn start_of_week(&self) -> Option<Self> {
        Self::from_anchor(self.as_anchor() - SECS_IN_DAY * self.weekday())
    }

    pub fn end_of_week(&self) -> Option<Self> {
        Self::from_anchor(self.as_anchor() + SECS_IN_DAY * (DAYS_IN_WEEK - 1 - self.weekday()))
    }

    // --------------------------------------------------------------------
    // private stuff

    const fn is_valid(&self) -> bool {
        self.year >= UNIX_START_YEAR
            && 0 < self.month
            && self.month <= MONTHS_IN_YEAR
            && 0 < self.day
            && self.day <= self.month_days()
            && 0 <= self.hour
            && self.hour < HOURS_IN_DAY
            && 0 <= self.minute
            && self.minute < MINS_IN_HOUR
            && 0 <= self.second
            && self.second < SECS_IN_MIN
    }
}

fn fmt_int(i: Int) -> String {
    if i > 9 {
        format!("{}", i)
    } else {
        format!("0{}", i)
    }
}

#[inline(always)]
/// days in the month of the given year
const fn month_days(month: Int, year: Int) -> Int {
    MONTH_TO_DAYS[(month - 1) as usize] + ((month == 2 && is_leap(year)) as Int)
}

#[inline(always)]
/// num of days in this year
const fn year_days(year: Int) -> Int {
    DAYS_IN_YEAR + (is_leap(year) as Int)
}

#[inline(always)]
/// (a / b, a % b)
const fn divmod(a: Int, b: Int) -> (Int, Int) {
    (a / b, a % b)
}

#[inline(always)]
const fn is_leap(year: Int) -> bool {
    // if year % 4 != 0 {
    //     return false;
    // }
    // if year % 100 != 0 {
    //     return true;
    // }
    // year % 400 == 0
    (year) % 4 == 0 && ((year) % 100 != 0 || (year) % 400 == 0)
}

#[inline(always)]
const fn div(a: Int, b: Int) -> Int {
    a / b - ((a % b < 0) as Int)
}

#[inline(always)]
const fn leaps_thru_end_of(year: Int) -> Int {
    div(year, 4) - div(year, 100) + div(year, 400)
}

// --------------------------------------------------------------------
// TESTS
// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        let res = DateTime::new_date(2000, 1, 1).unwrap().is_leap_year();
        let exp = true;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2004, 1, 1).unwrap().is_leap_year();
        let exp = true;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2100, 1, 1).unwrap().is_leap_year();
        let exp = false;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2400, 1, 1).unwrap().is_leap_year();
        let exp = true;
        assert_eq!(res, exp);
        let res = DateTime::new_date(1999, 1, 1).unwrap().is_leap_year();
        let exp = false;
        assert_eq!(res, exp);
    }

    #[test]
    fn test_weekday() {
        let res = DateTime::new_date(2025, 5, 29).unwrap().weekday();
        let exp = 3;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2025, 5, 30).unwrap().weekday();
        let exp = 4;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2025, 5, 31).unwrap().weekday();
        let exp = 5;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2025, 6, 1).unwrap().weekday();
        let exp = 6;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2025, 6, 2).unwrap().weekday();
        let exp = 0;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2025, 6, 3).unwrap().weekday();
        let exp = 1;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2025, 6, 4).unwrap().weekday();
        let exp = 2;
        assert_eq!(res, exp);
        let res = DateTime::new_date(2025, 6, 5).unwrap().weekday();
        let exp = 3;
        assert_eq!(res, exp);
    }

    #[test]
    fn test_prev_day() {
        let res = DateTime::new_date(2025, 5, 26).unwrap().prev_day().unwrap();
        let exp = DateTime::new_date(2025, 5, 25).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_next_day() {
        let res = DateTime::new_date(2025, 5, 26).unwrap().next_day().unwrap();
        let exp = DateTime::new_date(2025, 5, 27).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_prev_week() {
        let res = DateTime::new_date(2025, 5, 27)
            .unwrap()
            .prev_week()
            .unwrap();
        let exp = DateTime::new_date(2025, 5, 20).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_next_week() {
        let res = DateTime::new_date(2025, 5, 20)
            .unwrap()
            .next_week()
            .unwrap();
        let exp = DateTime::new_date(2025, 5, 27).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_prev_month() {
        let res = DateTime::new_date(2025, 6, 20)
            .unwrap()
            .prev_month()
            .unwrap();
        let exp = DateTime::new_date(2025, 5, 20).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_next_month() {
        let res = DateTime::new_date(2025, 4, 20)
            .unwrap()
            .next_month()
            .unwrap();
        let exp = DateTime::new_date(2025, 5, 20).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_prev_year() {
        let res = DateTime::new_date(2025, 5, 20)
            .unwrap()
            .prev_year()
            .unwrap();
        let exp = DateTime::new_date(2024, 5, 20).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_next_year() {
        let res = DateTime::new_date(2024, 4, 20)
            .unwrap()
            .next_year()
            .unwrap();
        let exp = DateTime::new_date(2025, 4, 20).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_quarter() {
        let exps = [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4];
        for month in 1..=12 {
            let res = DateTime::new_date(2024, month, 20).unwrap().quarter();
            let exp = exps[(month - 1) as usize];
            assert_eq!(res, exp);
        }
    }

    #[test]
    fn test_start_of_month() {
        let res = DateTime::new_date(2025, 4, 20)
            .unwrap()
            .start_of_month()
            .unwrap();
        let exp = DateTime::new_date(2025, 4, 1).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_end_of_month() {
        let res = DateTime::new_date(2025, 4, 20)
            .unwrap()
            .end_of_month()
            .unwrap();
        let exp = DateTime::new_date(2025, 4, 30).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_start_of_week() {
        let res = DateTime::new_date(2025, 5, 29)
            .unwrap()
            .start_of_week()
            .unwrap();
        let exp = DateTime::new_date(2025, 5, 26).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_end_of_week() {
        let res = DateTime::new_date(2025, 5, 29)
            .unwrap()
            .end_of_week()
            .unwrap();
        let exp = DateTime::new_date(2025, 6, 1).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_ord() {
        let d = DateTime::new_date(2025, 5, 29).unwrap();
        let res = d < d.next_week().unwrap();
        let exp = true;
        assert_eq!(res, exp);
    }
}
