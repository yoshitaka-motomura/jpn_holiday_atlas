//! # Datebook
//!
//! Datebook is a library for Japanese holidays.
//!
//! ## Description
//!
//! Returns a list of dates that are set as holidays based on Japan's national holiday law for the year 2023 in vector format.
//! Note that temporary holiday transfers, etc. are not supported.
//!
//! See: [Japanese national holiday law](https://www8.cao.go.jp/chosei/shukujitsu/gaiyou.html)
//!
//! The vernal and autumnal equinoxes are not strictly calculated, as they are affected by the actual astronomical motion of the celestial bodies.
//! Currently, the projected dates from 2020 to 2050 are returned.
//! See: [Vernal Equinox Day](https://ja.wikipedia.org/wiki/%E6%98%A5%E5%88%86%E3%81%AE%E6%97%A5)
//!
//! ## Usage
//!    use datebook;
//!    use datebook::timebase::defaults;
//! ```
//!  fn main() {
//!      let d = defaults().unwrap();
//!     println!("{:?}", d);
//!  }
//! ```

use csv;
#[allow(unused_imports)]
use anyhow::{Result, Error};
const BASE_DATA: &[u8] = include_bytes!("base.csv");
const BASE_EQUINOX: &[u8] = include_bytes!("equinox_base_dates.csv");

#[derive(Debug)]
pub struct Condition {
    pub month: String,
    pub n: u32,
    pub weekday: String,
}

#[derive(Debug)]
    pub struct BaseHolyday {
    pub name: String,
    pub date: Option<String>,
    pub relative: bool,
    pub condition: Option<Condition>,
}
// List of Japanese Holidays throughout the Year
#[allow(dead_code)]
pub fn get_schedule()-> Result<Vec<BaseHolyday>> {
    //let path = format!("{}/src/utils/base.csv", env!("CARGO_MANIFEST_DIR"));
    let mut base_dates: Vec<BaseHolyday> = Vec::new();
    let mut reader = csv::Reader::from_reader(BASE_DATA);
    for result in reader.records() {
        match result {
            Ok(record) => {
                let m: Vec<String> = record.iter().map(|x| x.to_string()).collect();
                let value = BaseHolyday {
                    name: m[0].to_string(),
                    date: if m[1].is_empty() { None } else { Some(m[1].to_string())},
                    relative: match m[2].parse() {
                        Ok(v) => v,
                        Err(_) => false,
                    },
                    condition: if m[3].is_empty() { None } else {
                        let c: Vec<String> = m[3].split(":").map(|x| x.to_string()).collect();
                        Some(Condition {
                            month: c[0].to_string(),
                            n: match c[1].parse() {
                                Ok(v) => v,
                                Err(_) => 0,
                            },
                            weekday: c[2].to_string(),
                        })
                    },
                };
                base_dates.push(value);
            },
            Err(err) => return Err(err.into()),
        }
    }

    Ok(base_dates)
}

//　Basic data on Japanese national holidays, the vernal equinox and autumnal equinox, will be returned.
pub fn get_equinox_from_year(year: u32) {
    let mut equinox_dates: Vec<String> = Vec::new();
    let mut reader = csv::Reader::from_reader(BASE_EQUINOX);

    for result in reader.records() {
        match result {
            Ok(record) => {
                let m: Vec<String> = record.iter().map(|x| x.to_string()).collect();
                if m[0].parse::<u32>().unwrap() == year {
                    equinox_dates = m;
                }
            },
            Err(err) => println!("{:?}", err),
        }
    }

    equinox_dates.remove(0);
    for day in equinox_dates {
        println!("{}", day);
    }
}

