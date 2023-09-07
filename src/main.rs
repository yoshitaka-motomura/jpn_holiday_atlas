// Title: main.rs
// Compare this snippet from src/datebook/mod.rs:

mod datebook;
use datebook::calendar::OutputFormat;
use datebook::calendar::holiday;
fn main() {
    let year = 2024;
    let format = OutputFormat::JSON;
    let d = holiday(format, year);
    println!("{:?}", d);
}
