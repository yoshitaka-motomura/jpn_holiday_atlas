// Title: main.rs
// Compare this snippet from src/datebook/mod.rs:

mod datebook;
#[allow(unused_imports)]
use datebook::timebase::{get_schedule, get_equinox_from_year};
fn main() {
    let d = get_equinox_from_year(2020);
    println!("{:?}", d);
}
