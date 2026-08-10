use super::vtk_type::VtkTypeUInt64;

pub const MILLIS_PER_SECOND: i32 = 1000;
pub const MILLIS_PER_MINUTE: i32 = 60000;
pub const MILLIS_PER_HOUR: i32 = 3600000;
pub const MILLIS_PER_DAY: i32 = 86400000;
pub const SECONDS_PER_MINUTE: i32 = 60;
pub const SECONDS_PER_HOUR: i32 = 3600;
pub const SECONDS_PER_DAY: i32 = 86400;
pub const MINUTES_PER_HOUR: i32 = 60;
pub const MINUTES_PER_DAY: i32 = 1440;
pub const HOURS_PER_DAY: i32 = 24;

/// VTK: `vtkTimePointUtility`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimePointUtility;

impl TimePointUtility {
    pub const ISO8601_DATETIME_MILLIS: i32 = 0;
    pub const ISO8601_DATETIME: i32 = 1;
    pub const ISO8601_DATE: i32 = 2;
    pub const ISO8601_TIME_MILLIS: i32 = 3;
    pub const ISO8601_TIME: i32 = 4;

    /// VTK: `vtkTimePointUtility::New`.
    pub fn new() -> Self {
        Self
    }

    /// VTK: `vtkTimePointUtility::DateToTimePoint`.
    pub fn date_to_time_point(year: i32, month: i32, day: i32) -> VtkTypeUInt64 {
        let mut year = year;
        if year < 0 {
            year += 1;
        }

        let julian_day =
            if year > 1582 || (year == 1582 && (month > 10 || (month == 10 && day >= 15))) {
                ((1461 * (year + 4800 + (month - 14) / 12)) / 4
                    + (367 * (month - 2 - 12 * ((month - 14) / 12))) / 12
                    - (3 * ((year + 4900 + (month - 14) / 12) / 100)) / 4
                    + day
                    - 32075) as VtkTypeUInt64
            } else if year < 1582 || (year == 1582 && (month < 10 || (month == 10 && day <= 4))) {
                let a = (14 - month) / 12;
                ((153 * (month + (12 * a) - 3) + 2) / 5 + (1461 * (year + 4800 - a)) / 4 + day
                    - 32083) as VtkTypeUInt64
            } else {
                0
            };

        julian_day * MILLIS_PER_DAY as VtkTypeUInt64
    }

    /// VTK: `vtkTimePointUtility::TimeToTimePoint`.
    pub fn time_to_time_point(hour: i32, minute: i32, second: i32, millis: i32) -> VtkTypeUInt64 {
        (MILLIS_PER_HOUR * hour + MILLIS_PER_MINUTE * minute + MILLIS_PER_SECOND * second + millis)
            as VtkTypeUInt64
    }

    /// VTK: `vtkTimePointUtility::DateTimeToTimePoint`.
    pub fn date_time_to_time_point(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millis: i32,
    ) -> VtkTypeUInt64 {
        Self::date_to_time_point(year, month, day)
            + Self::time_to_time_point(hour, minute, second, millis)
    }

    /// VTK: `vtkTimePointUtility::GetDate`.
    pub fn get_date(time: VtkTypeUInt64) -> (i32, i32, i32) {
        let mut julian_day = (time / MILLIS_PER_DAY as VtkTypeUInt64) as i32;
        let (year, month, day);

        if julian_day >= 2299161 {
            let mut ell = julian_day + 68569;
            let n = (4 * ell) / 146097;
            ell -= (146097 * n + 3) / 4;
            let i = (4000 * (ell + 1)) / 1461001;
            ell = ell - (1461 * i) / 4 + 31;
            let j = (80 * ell) / 2447;
            day = ell - (2447 * j) / 80;
            ell = j / 11;
            month = j + 2 - (12 * ell);
            year = 100 * (n - 49) + i + ell;
        } else {
            julian_day += 32082;
            let dd = (4 * julian_day + 3) / 1461;
            let ee = julian_day - (1461 * dd) / 4;
            let mm = ((5 * ee) + 2) / 153;
            day = ee - (153 * mm + 2) / 5 + 1;
            month = mm + 3 - 12 * (mm / 10);
            year = dd - 4800 + (mm / 10);
            if year <= 0 {
                return (year - 1, month, day);
            }
        }

        (year, month, day)
    }

    /// VTK: `vtkTimePointUtility::GetTime`.
    pub fn get_time(time: VtkTypeUInt64) -> (i32, i32, i32, i32) {
        let hour = ((time % MILLIS_PER_DAY as VtkTypeUInt64) as i32) / MILLIS_PER_HOUR;
        let minute = ((time % MILLIS_PER_HOUR as VtkTypeUInt64) as i32) / MILLIS_PER_MINUTE;
        let second = ((time % MILLIS_PER_MINUTE as VtkTypeUInt64) as i32) / MILLIS_PER_SECOND;
        let millis = (time % MILLIS_PER_SECOND as VtkTypeUInt64) as i32;
        (hour, minute, second, millis)
    }

    /// VTK: `vtkTimePointUtility::GetDateTime`.
    pub fn get_date_time(time: VtkTypeUInt64) -> (i32, i32, i32, i32, i32, i32, i32) {
        let (year, month, day) = Self::get_date(time);
        let (hour, minute, second, millis) = Self::get_time(time);
        (year, month, day, hour, minute, second, millis)
    }

    /// VTK: `vtkTimePointUtility::GetYear`.
    pub fn get_year(time: VtkTypeUInt64) -> i32 {
        Self::get_date(time).0
    }

    /// VTK: `vtkTimePointUtility::GetMonth`.
    pub fn get_month(time: VtkTypeUInt64) -> i32 {
        Self::get_date(time).1
    }

    /// VTK: `vtkTimePointUtility::GetDay`.
    pub fn get_day(time: VtkTypeUInt64) -> i32 {
        Self::get_date(time).2
    }

    /// VTK: `vtkTimePointUtility::GetHour`.
    pub fn get_hour(time: VtkTypeUInt64) -> i32 {
        ((time % MILLIS_PER_DAY as VtkTypeUInt64) as i32) / MILLIS_PER_HOUR
    }

    /// VTK: `vtkTimePointUtility::GetMinute`.
    pub fn get_minute(time: VtkTypeUInt64) -> i32 {
        ((time % MILLIS_PER_HOUR as VtkTypeUInt64) as i32) / MILLIS_PER_MINUTE
    }

    /// VTK: `vtkTimePointUtility::GetSecond`.
    pub fn get_second(time: VtkTypeUInt64) -> i32 {
        ((time % MILLIS_PER_MINUTE as VtkTypeUInt64) as i32) / MILLIS_PER_SECOND
    }

    /// VTK: `vtkTimePointUtility::GetMillisecond`.
    pub fn get_millisecond(time: VtkTypeUInt64) -> i32 {
        (time % MILLIS_PER_SECOND as VtkTypeUInt64) as i32
    }

    /// VTK: `vtkTimePointUtility::ISO8601ToTimePoint`.
    pub fn iso8601_to_time_point(cstr: Option<&str>, ok: Option<&mut bool>) -> VtkTypeUInt64 {
        let str = cstr.unwrap_or("");
        let mut format_valid;
        let mut value = 0;

        if str.len() == 19 || str.len() == 23 {
            format_valid = check_iso8601_pattern(
                str,
                &[(4, b'-'), (7, b'-'), (13, b':'), (16, b':')],
                Some((10, &[b'T', b' '])),
                (str.len() == 23).then_some((19, b'.')),
            );
            if format_valid {
                if let (
                    Some(year),
                    Some(month),
                    Some(day),
                    Some(hour),
                    Some(minute),
                    Some(second),
                ) = (
                    parse_i32(&str[0..4]),
                    parse_i32(&str[5..7]),
                    parse_i32(&str[8..10]),
                    parse_i32(&str[11..13]),
                    parse_i32(&str[14..16]),
                    parse_i32(&str[17..19]),
                ) {
                    let millis = if str.len() == 23 {
                        parse_i32(&str[20..23]).unwrap_or_else(|| {
                            format_valid = false;
                            0
                        })
                    } else {
                        0
                    };
                    if format_valid {
                        value = Self::date_time_to_time_point(
                            year, month, day, hour, minute, second, millis,
                        );
                    }
                } else {
                    format_valid = false;
                }
            }
        } else if str.len() == 10 {
            format_valid = check_iso8601_pattern(str, &[(4, b'-'), (7, b'-')], None, None);
            if format_valid {
                if let (Some(year), Some(month), Some(day)) = (
                    parse_i32(&str[0..4]),
                    parse_i32(&str[5..7]),
                    parse_i32(&str[8..10]),
                ) {
                    value = Self::date_to_time_point(year, month, day);
                } else {
                    format_valid = false;
                }
            }
        } else if str.len() == 8 || str.len() == 12 {
            format_valid = check_iso8601_pattern(
                str,
                &[(2, b':'), (5, b':')],
                None,
                (str.len() == 12).then_some((8, b'.')),
            );
            if format_valid {
                if let (Some(hour), Some(minute), Some(second)) = (
                    parse_i32(&str[0..2]),
                    parse_i32(&str[3..5]),
                    parse_i32(&str[6..8]),
                ) {
                    let millis = if str.len() == 12 {
                        parse_i32(&str[9..12]).unwrap_or_else(|| {
                            format_valid = false;
                            0
                        })
                    } else {
                        0
                    };
                    if format_valid {
                        value = Self::time_to_time_point(hour, minute, second, millis);
                    }
                } else {
                    format_valid = false;
                }
            }
        } else {
            format_valid = false;
        }

        if let Some(ok) = ok {
            *ok = format_valid;
        }
        value
    }

    /// VTK: `vtkTimePointUtility::TimePointToISO8601`.
    pub fn time_point_to_iso8601(time: VtkTypeUInt64, format: i32) -> Option<String> {
        let (year, month, day, hour, minute, second, millis) = Self::get_date_time(time);

        Some(match format {
            Self::ISO8601_DATETIME => format!(
                "{}-{}-{}T{}:{}:{}",
                width(year, 4),
                width(month, 2),
                width(day, 2),
                width(hour, 2),
                width(minute, 2),
                width(second, 2)
            ),
            Self::ISO8601_DATETIME_MILLIS => format!(
                "{}-{}-{}T{}:{}:{}.{}",
                width(year, 4),
                width(month, 2),
                width(day, 2),
                width(hour, 2),
                width(minute, 2),
                width(second, 2),
                width(millis, 3)
            ),
            Self::ISO8601_DATE => {
                format!("{}-{}-{}", width(year, 4), width(month, 2), width(day, 2))
            }
            Self::ISO8601_TIME => format!(
                "{}:{}:{}",
                width(hour, 2),
                width(minute, 2),
                width(second, 2)
            ),
            Self::ISO8601_TIME_MILLIS => format!(
                "{}:{}:{}.{}",
                width(hour, 2),
                width(minute, 2),
                width(second, 2),
                width(millis, 3)
            ),
            _ => return None,
        })
    }
}

fn parse_i32(value: &str) -> Option<i32> {
    value.parse().ok()
}

fn width(value: i32, width: usize) -> String {
    format!("{:0>width$}", value.to_string(), width = width)
}

fn check_iso8601_pattern(
    value: &str,
    fixed: &[(usize, u8)],
    alternatives: Option<(usize, &[u8])>,
    optional_fixed: Option<(usize, u8)>,
) -> bool {
    let bytes = value.as_bytes();
    for (index, byte) in bytes.iter().copied().enumerate() {
        if fixed.iter().any(|(fixed_index, _)| *fixed_index == index) {
            if fixed
                .iter()
                .find(|(fixed_index, _)| *fixed_index == index)
                .unwrap()
                .1
                != byte
            {
                return false;
            }
        } else if let Some((alt_index, allowed)) = alternatives {
            if index == alt_index {
                if !allowed.contains(&byte) {
                    return false;
                }
            } else if let Some((opt_index, opt_byte)) = optional_fixed {
                if index == opt_index {
                    if byte != opt_byte {
                        return false;
                    }
                } else if !byte.is_ascii_digit() {
                    return false;
                }
            } else if !byte.is_ascii_digit() {
                return false;
            }
        } else if let Some((opt_index, opt_byte)) = optional_fixed {
            if index == opt_index {
                if byte != opt_byte {
                    return false;
                }
            } else if !byte.is_ascii_digit() {
                return false;
            }
        } else if !byte.is_ascii_digit() {
            return false;
        }
    }
    true
}
