//! # holidays
//! 指定した年の日本の祝日を取得する
//!
//! ## Example
//! ```
//! use list_holiday_of_jpn::holidays::holidays;
//! fn main() {
//!    let year = 2024;
//!    let output = holidays(year);
//!    println!("{:?}", output);
//! }
//! ```
//!
pub mod holidays {
    #[allow(unused_imports)]
    use std::fs;
    use chrono::{Datelike, Duration, Weekday, NaiveDate, Local, DateTime, Utc};
    use chrono::TimeZone;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, to_string_pretty};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct HolidayShapedItem {
        pub name: String,
        pub date: String,
        pub time: i64,
        pub substitute: bool,
    }
    #[derive(Debug, Deserialize, Serialize)]
    pub struct OutputFormat {
        pub year: i32,
        pub holidays: Vec<HolidayShapedItem>,
        pub message: String,
    }
    #[derive(Debug, Deserialize)]
    pub struct EquinoxDates {
        pub spring: String,
        pub fall: String,
    }
    #[derive(Debug)]
    pub struct Holiday {
        pub name: String,
        pub date: NaiveDate,
        pub substitute: bool,
    }
    pub fn holidays(year:i32) -> String {
        let base_dates = include_str!("base.json"); //祝日の基準日データ
        let json: Value = serde_json::from_str(&base_dates).unwrap();

        let mut prepara_holidays = prepare_holidays(json, year); //祝日のレコードを準備
        let equinox_dates = get_equinox_date(year); //2020~2050年までの春分の日と秋分の日を取得

        check_substitute_holidays(&mut prepara_holidays);
        prepara_holidays.extend(equinox_dates);


        let output: OutputFormat = OutputFormat {
            year: year,
            holidays: format_by_holidays(prepara_holidays),
            message: "The vernal and autumnal equinoxes of future dates are predictions.".to_string(),
        };

        json_output(output)
    }

    fn format_by_holidays(mut holidays: Vec<Holiday>) -> Vec<HolidayShapedItem> {
        // Sort the holidays by date
        holidays.sort_by(|a, b| a.date.cmp(&b.date));

        holidays.iter().filter_map(|holiday| {
            // NaiveDate から NaiveDateTime を作成し、次に DateTime<Utc> に変換します
            let naive_datetime_opt = holiday.date.and_hms_opt(0, 0, 0);

            naive_datetime_opt.map(|naive_datetime| {
                let datetime = Utc.from_utc_datetime(&naive_datetime);
                let time = datetime.timestamp();

                HolidayShapedItem {
                    name: holiday.name.clone(),
                    date: holiday.date.format("%Y-%m-%d").to_string(),
                    time: time,
                    substitute: holiday.substitute,
                }
            })
        }).collect()
    }
    fn json_output(output: OutputFormat) -> String {
        to_string_pretty(&output).unwrap()
    }
    fn get_equinox_date(year: i32) -> Vec<Holiday> {
        if year < 2020 || year > 2050 {
            return Vec::new();
        }
        let base: &str = include_str!("equinox_base_dates.json");
        let equinoxes: std::collections::HashMap<String, EquinoxDates> =
            serde_json::from_str(base).expect("Error parsing the json");

        let mut holidays = Vec::new();

        if let Some(dates) = equinoxes.get(&year.to_string()) {
            let spring_date = format!("{}-{}", year, dates.spring);
            let fall_date = format!("{}-{}", year, dates.fall);

            let spring = NaiveDate::parse_from_str(&spring_date, "%Y-%m-%d").unwrap();
            let fall = NaiveDate::parse_from_str(&fall_date, "%Y-%m-%d").unwrap();

            let spring_substitute = if spring.weekday() == Weekday::Sun {
                Some(Holiday {
                    name: "春分の日(振替休日)".to_string(),
                    date: spring.succ_opt().expect("Failed to get next day"),
                    substitute: true,
                })
            } else {
                None
            };

            let fall_substitute = if fall.weekday() == Weekday::Sun {
                Some(Holiday {
                    name: "秋分の日(振替休日)".to_string(),
                    date: fall.succ_opt().expect("Failed to get next day"),
                    substitute: true,
                })
            } else {
                None
            };

            holidays.push(Holiday {
                name: "春分の日".to_string(),
                date: spring,
                substitute: false,
            });

            holidays.push(Holiday {
                name: "秋分の日".to_string(),
                date: fall,
                substitute: false,
            });

            if let Some(sub) = spring_substitute {
                holidays.push(sub);
            }

            if let Some(sub) = fall_substitute {
                holidays.push(sub);
            }
        }

        holidays
    }
    fn check_substitute_holidays(holidays: &mut Vec<Holiday>) {
        let mut i = 0;
        while i < holidays.len() {
            if holidays[i].date.weekday() == Weekday::Sun {
                // 連続する祝日を確認
                let mut last_holiday_date = holidays[i].date;
                while let Some(next_holiday) = holidays.get(i + 1) {
                    if next_holiday.date == last_holiday_date + Duration::days(1) {
                        i += 1;
                        last_holiday_date = next_holiday.date;
                    } else {
                        break;
                    }
                }

                // 振替休日を設定
                let mut substitute_date = last_holiday_date + Duration::days(1);

                // すでに存在する祝日との衝突を確認
                while holidays.iter().any(|h| h.date == substitute_date) {
                    substitute_date = substitute_date + Duration::days(1);
                }

                holidays.push(Holiday {
                    name: format!("振替休日({})", holidays[i].name),
                    date: substitute_date,
                    substitute: true,
                });
            }
            i += 1;
        }
    }

    fn prepare_holidays(json: Value, year: i32)-> Vec<Holiday> {
        let mut holidays: Vec<Holiday> = Vec::new();
        for item in json.as_array().unwrap() {
            if item["relative"].as_bool().unwrap() {
                //変動日の処理
                let parts: Vec<&str> = item["condition"].as_str().unwrap().split(',').collect();
                let month = get_month_num_from_string(parts[0]).unwrap(); //月
                let n = parts[1].parse::<u32>().unwrap(); //第n週目
                let weekday = get_weekday_from_string(parts[2]).unwrap(); //曜日
                let day = nth_weekday_of_month(year, month, n, weekday).unwrap();

                holidays.push(Holiday {
                    name: item["name"].as_str().unwrap().to_string(),
                    date: day.format("%Y-%m-%d").to_string().parse::<NaiveDate>().unwrap(),
                    substitute: false,
                });

            }else {
                //固定日の処理
                let conditions: Vec<u32> = item["date"].as_str().unwrap().split('/').map(|s| s.parse().unwrap()).collect();
                let date_opt = NaiveDate::from_ymd_opt(year, conditions[0], conditions[1]);
                holidays.push(Holiday {
                    name: item["name"].as_str().unwrap().to_string(),
                    date: date_opt.unwrap(),
                    substitute: false,
                });
            }
        }

        holidays

    }

    fn nth_weekday_of_month(year: i32, month: u32, n: u32, target_weekday: Weekday) -> Option<DateTime<Local>> {
        let mut date = Local.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();
        let mut dates: Vec<DateTime<Local>> = Vec::new();

        while date.month() == month {
            if date.weekday() == target_weekday {
                dates.push(date);
            }
            date =  date + Duration::days(1);
        }

        let nth_day = dates[n as usize - 1];
        Some(nth_day)

    }

    fn get_weekday_from_string(char: &str)-> Option<Weekday> {
        //曜日の名前からchronoのWeekday構造体を取得
        match char.trim().to_lowercase().as_str() {
            "monday" | "mon" => Some(Weekday::Mon),
            "tuesday" | "tue" => Some(Weekday::Tue),
            "wednesday" | "wed" => Some(Weekday::Wed),
            "thursday" | "thu" => Some(Weekday::Thu),
            "friday" | "fri" => Some(Weekday::Fri),
            "saturday" | "sat" => Some(Weekday::Sat),
            "sunday" | "sun" => Some(Weekday::Sun),
            _=> return None,
        }
    }

    fn get_month_num_from_string(char: &str) -> Option<u32> {
        //月の名前から月の数字を取得
        match char.trim().to_lowercase().as_str() {
            "january" | "jan" => Some(1),
            "february" | "feb" => Some(2),
            "march" | "mar" => Some(3),
            "april" | "apr" => Some(4),
            "may" => Some(5),
            "june" | "jun" => Some(6),
            "july" | "jul" => Some(7),
            "august" | "aug" => Some(8),
            "september" | "sep" => Some(9),
            "october" | "oct" => Some(10),
            "november" | "nov" => Some(11),
            "december" | "dec" => Some(12),
            _ => return None,
        }
    }
    // holidays
    // private module function unittest
    #[test]
    fn test_get_month_num_from_string() {
        let month_long_name = get_month_num_from_string("January");
        let month_short_name = get_month_num_from_string("Jan");
        let none_month_name = get_month_num_from_string("Jann");
        assert_eq!(month_long_name, Some(1), "Failed to get month number from long name");
        assert_eq!(month_short_name, Some(1), "Failed to get month number from short name");
        assert_eq!(none_month_name, None, "Failed to get month number from none name");
    }
    #[test]
    fn test_get_weekday_from_string() {
        let weekday_long_name = get_weekday_from_string("Monday");
        let weekday_short_name = get_weekday_from_string("Mon");
        let none_weekday_name = get_weekday_from_string("Mond");
        assert_eq!(weekday_long_name, Some(Weekday::Mon), "Failed to get weekday from long name");
        assert_eq!(weekday_short_name, Some(Weekday::Mon), "Failed to get weekday from short name");
        assert_eq!(none_weekday_name, None, "Failed to get weekday from none name");
    }
}

#[cfg(test)]
mod test {
    use crate::holidays::OutputFormat;
    use super::holidays::holidays;

    #[test]
    fn test_holidays() {
        let year = 2022;
        let output = holidays(year);
        let result: Result<OutputFormat, _> = serde_json::from_str(output.as_str());
        assert!(result.is_ok(), "Failed to parse json");
    }
}
