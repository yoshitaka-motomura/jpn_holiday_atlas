//! # Calendar
//! This module is for calendar.
//! ## Example
//! ```
//! use datebook::calendar::{holiday, OutputFormat};
//! let year = 2024;
//! let format = OutputFormat::YAML;
//! let result = holiday(format, year).unwrap();
//! println!("{}", result);
//! ```
//!
//! ## Output Format
//! | Format | Description |
//! | --- | --- |
//! | JSON | JSON format |
//! | YAML | YAML format |
//! | CSV | CSV format |
//!
//! ## Output Example
//! ### JSON
//! ```json
//! [
//!  {
//!     "name": "元旦",
//!    "date": "2024-01-01",
//!   "substitute": false
//! },
//! ]
//! ```
//! ### YAML
//! ```yaml
//! - name: 元旦
//! date: 2024-01-01
//! substitute: false
//! ```
//! ### CSV
//! ```csv
//! name,date,substitute
//! 元旦,2024-01-01,false
//! ```
//! ## Note
//! This module outputs a list of Japanese holidays based on the National Holidays Law.
//! Variations due to special events cannot be handled.
//!
//! Note: The exact dates of future vernal equinoxes and autumnal equinoxes cannot be calculated.
//! This is due to the need for astronomical data. However,
//! we use the predictions of Japanese observatories up to the year 2050.
//! https://www8.cao.go.jp/chosei/shukujitsu/gaiyou.html
//!
#[allow(unused_imports)]
use std::fs;
use chrono::{Datelike, Duration, Weekday, NaiveDate, Local, DateTime};
use chrono::TimeZone;
use anyhow::{Result, Error};
use serde::Serialize;
use serde_json::to_string_pretty;
use super::timebase::{get_schedule, get_equinox_dates, Condition};

#[derive(Debug)]
#[allow(dead_code)]
pub enum OutputFormat {
    JSON,
    CSV,
    YAML,
}
#[derive(Serialize)]
pub struct Holiday {
    pub name: String,
    pub date: NaiveDate,
    pub substitute: bool,
}

pub fn holiday(format:OutputFormat, year: u32)-> Result<String, Error> {
    //List of holidays stipulated in the Holidays Act
    let mut m = prepara(year);
    let e= pick_exuinox_from_year(year);
    m.extend(e);
    substitute_adjustment(&mut m);

    //sort
    m.sort_by(|a, b| a.date.cmp(&b.date));

    match format {
        OutputFormat::CSV => {
            let mut csv = String::new();
            csv.push_str("name,date,substitute\n");
            for d in m {
                csv.push_str(&format!("{},{},{}\n", d.name, d.date, d.substitute));
            }
            Ok(csv)
        },
        OutputFormat::JSON => {
            let json = to_string_pretty(&m).unwrap();
            Ok(json)
        },
        OutputFormat::YAML => {
            let yaml = serde_yaml::to_string(&m).unwrap();
            Ok(yaml)
        }
    }

}

// private functions

fn substitute_adjustment(data: &mut Vec<Holiday>) {
   let mut i:usize = 0;
   while i < data.len() {
        // if it a Sunday
        if data[i].date.weekday() == Weekday::Sun {
            let mut last_holiday_date = data[i].date;
            while let Some(next_holiday) = data.get(i+1) {
                if next_holiday.date == last_holiday_date + Duration::days(1) {
                    i += 1;
                    last_holiday_date = next_holiday.date;
                } else {
                    break;
                }
            }
            let mut sub_date = last_holiday_date + Duration::days(1);
            while data.iter().any(|h:&Holiday| h.date == sub_date) {
                sub_date = sub_date + Duration::days(1);
            }

            data.push(Holiday {
                name: format!("振替休日({})", data[i].name),
                date: sub_date,
                substitute: true,
            });
        }
        i += 1;
   }
}


fn pick_exuinox_from_year(year:u32) -> Vec<Holiday> {
    if year < 2020 || year > 2050 {
        return Vec::new();
    }
    let equinoxes = get_equinox_dates().unwrap();
    let target = equinoxes.into_iter().find(|x| x.year == year);
    let mut return_value: Vec<Holiday> = Vec::new();
    match target {
        Some(v) => {
            v.equinox.into_iter().for_each(|x| {
                return_value.push(Holiday {
                    name: x.name,
                    date: NaiveDate::parse_from_str(&format!("{}/{}", year, x.date).to_string(), "%Y/%m/%d").unwrap(),
                    substitute: false,
                });
            })
        },
        None => {},
    }

    return_value


}

// for base dates
fn prepara(year: u32)->Vec<Holiday> {
    let dataset = get_schedule().unwrap();
    let mut days: Vec<Holiday> = Vec::new();
    for d in dataset {
        if d.relative {
            let relative_date = get_relative_date(year, d.condition.unwrap()).unwrap();
            days.push(Holiday {
                name: d.name,
                date: relative_date.format("%Y-%m-%d").to_string().parse::<NaiveDate>().unwrap(),
                substitute: false,
            })
        } else {
            days.push(Holiday {
                name: d.name,
                date: NaiveDate::parse_from_str(&format!("{}/{}", year, d.date.unwrap()).to_string()
                , "%Y/%m/%d").unwrap(),
                substitute: false,
            })
        }
    }
    days
}

// for relative date comvart Datetime
fn get_relative_date(year: u32, condition: Condition)-> Option<DateTime<Local>> {
    let month = get_month_num_from_string(&condition.month).unwrap();
    let weekday = get_weekday_from_string(&condition.weekday).unwrap();
    let n = condition.n;
    let mut dates: Vec<DateTime<Local>> = Vec::new();
    let mut day:DateTime<Local> = Local.with_ymd_and_hms(year as i32, month, 1, 0, 0, 0).unwrap();

    while day.month() == month {
        if day.weekday() == weekday {
            dates.push(day);
        }
        day = day + Duration::days(1);
    }

    Some(dates[n as usize -1])
}

fn get_weekday_from_string(char: &str)-> Option<Weekday> {
    match char.trim().to_lowercase().as_str() {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}
fn get_month_num_from_string(char: &str) -> Option<u32> {
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
        _ => None,
    }
}

#[cfg(test)]
pub mod test {
    use pretty_assertions::assert_eq;
    #[test]
    pub fn test_holiday_output_yml() {
        let year = 2024;
        let expected = "- name: 元旦\n  date: 2024-01-01\n  substitute: false\n- name: 成人の日\n  date: 2024-01-08\n  substitute: false\n- name: 建国記念の日\n  date: 2024-02-11\n  substitute: false\n- name: 振替休日(建国記念の日)\n  date: 2024-02-12\n  substitute: true\n- name: 天皇誕生日\n  date: 2024-02-23\n  substitute: false\n- name: 春分の日\n  date: 2024-03-20\n  substitute: false\n- name: 昭和の日\n  date: 2024-04-29\n  substitute: false\n- name: 憲法記念日\n  date: 2024-05-03\n  substitute: false\n- name: みどりの日\n  date: 2024-05-04\n  substitute: false\n- name: こどもの日\n  date: 2024-05-05\n  substitute: false\n- name: 振替休日(こどもの日)\n  date: 2024-05-06\n  substitute: true\n- name: 海の日\n  date: 2024-07-15\n  substitute: false\n- name: 山の日\n  date: 2024-08-11\n  substitute: false\n- name: 振替休日(山の日)\n  date: 2024-08-12\n  substitute: true\n- name: 敬老の日\n  date: 2024-09-16\n  substitute: false\n- name: 秋分の日\n  date: 2024-09-22\n  substitute: false\n- name: 振替休日(秋分の日)\n  date: 2024-09-23\n  substitute: true\n- name: スポーツの日\n  date: 2024-10-14\n  substitute: false\n- name: 文化の日\n  date: 2024-11-03\n  substitute: false\n- name: 振替休日(文化の日)\n  date: 2024-11-04\n  substitute: true\n- name: 勤労感謝の日\n  date: 2024-11-23\n  substitute: false\n";
        let format = super::OutputFormat::YAML;
        let result = super::holiday(format, year).unwrap();
        assert_eq!(result, expected);
    }
    #[test]
    pub fn test_holiday_output_json() {
        let year = 2024;
        let expected = "[\n  {\n    \"name\": \"元旦\",\n    \"date\": \"2024-01-01\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"成人の日\",\n    \"date\": \"2024-01-08\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"建国記念の日\",\n    \"date\": \"2024-02-11\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"振替休日(建国記念の日)\",\n    \"date\": \"2024-02-12\",\n    \"substitute\": true\n  },\n  {\n    \"name\": \"天皇誕生日\",\n    \"date\": \"2024-02-23\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"春分の日\",\n    \"date\": \"2024-03-20\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"昭和の日\",\n    \"date\": \"2024-04-29\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"憲法記念日\",\n    \"date\": \"2024-05-03\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"みどりの日\",\n    \"date\": \"2024-05-04\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"こどもの日\",\n    \"date\": \"2024-05-05\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"振替休日(こどもの日)\",\n    \"date\": \"2024-05-06\",\n    \"substitute\": true\n  },\n  {\n    \"name\": \"海の日\",\n    \"date\": \"2024-07-15\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"山の日\",\n    \"date\": \"2024-08-11\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"振替休日(山の日)\",\n    \"date\": \"2024-08-12\",\n    \"substitute\": true\n  },\n  {\n    \"name\": \"敬老の日\",\n    \"date\": \"2024-09-16\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"秋分の日\",\n    \"date\": \"2024-09-22\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"振替休日(秋分の日)\",\n    \"date\": \"2024-09-23\",\n    \"substitute\": true\n  },\n  {\n    \"name\": \"スポーツの日\",\n    \"date\": \"2024-10-14\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"文化の日\",\n    \"date\": \"2024-11-03\",\n    \"substitute\": false\n  },\n  {\n    \"name\": \"振替休日(文化の日)\",\n    \"date\": \"2024-11-04\",\n    \"substitute\": true\n  },\n  {\n    \"name\": \"勤労感謝の日\",\n    \"date\": \"2024-11-23\",\n    \"substitute\": false\n  }\n]";
        let format = super::OutputFormat::JSON;
        let result = super::holiday(format, year).unwrap();
        assert_eq!(result, expected)
    }

    #[test]
    pub fn test_holiday_output_csv() {
        let year = 2024;
        let expected = "name,date,substitute\n元旦,2024-01-01,false\n成人の日,2024-01-08,false\n建国記念の日,2024-02-11,false\n振替休日(建国記念の日),2024-02-12,true\n天皇誕生日,2024-02-23,false\n春分の日,2024-03-20,false\n昭和の日,2024-04-29,false\n憲法記念日,2024-05-03,false\nみどりの日,2024-05-04,false\nこどもの日,2024-05-05,false\n振替休日(こどもの日),2024-05-06,true\n海の日,2024-07-15,false\n山の日,2024-08-11,false\n振替休日(山の日),2024-08-12,true\n敬老の日,2024-09-16,false\n秋分の日,2024-09-22,false\n振替休日(秋分の日),2024-09-23,true\nスポーツの日,2024-10-14,false\n文化の日,2024-11-03,false\n振替休日(文化の日),2024-11-04,true\n勤労感謝の日,2024-11-23,false\n";
        let format = super::OutputFormat::CSV;
        let result = super::holiday(format, year).unwrap();
        assert_eq!(result, expected)
    }

}
