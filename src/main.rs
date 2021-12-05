extern crate anyhow;
extern crate aoc_2021;
extern crate clap;

use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use clap::{App, Arg, ArgMatches};

use aoc_2021::*;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let matches = App::new("Advent of Code 2021")
        .version(version().as_str())
        .about("Solutions to AoC 2021 !")
        .arg(
            Arg::with_name("day")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("Which day's solution you want to run"),
        )
        .get_matches();
    match get_number("day", Some(0), &matches) {
        1 => day_01::run()?,
        2 => day_02::run()?,
        3 => day_03::run()?,
        4 => day_04::run()?,
        5 => day_05::run()?,
        other => anyhow::bail!(format!("Invalid day: {}", other)),
    }
    Ok(())
}

fn version() -> String {
    let (maj, min, pat) = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH"),
    );
    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_owned(),
    }
}

fn get_number<'a, A>(name: &str, maybe_min: Option<A>, matches: &ArgMatches<'a>) -> A
where
    A: FromStr + PartialOrd + Display + Copy,
    <A as FromStr>::Err: std::fmt::Debug,
{
    matches
        .value_of(name)
        .and_then(|s| s.parse::<A>().ok())
        .and_then(|u| match maybe_min {
            Some(min) => {
                if u > min {
                    Some(u)
                } else {
                    None
                }
            }
            _ => Some(u),
        })
        .expect(
            &{
                if let Some(min) = maybe_min {
                    format!("{} should be a positive number greater than {}.", name, min)
                } else {
                    format!("{} should be a positive number.", name)
                }
            }[..],
        )
}
