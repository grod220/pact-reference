//! The `pact_verifier_cli` crate provides the CLI for verification of service providers based on
//! pact files. It implements the V1 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-1).

#![warn(missing_docs)]

#[macro_use] extern crate clap;
#[macro_use] extern crate p_macro;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate pact_matching;
extern crate pact_verifier;
extern crate simplelog;
extern crate rand;
extern crate regex;

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

#[cfg(test)]
extern crate quickcheck;

use std::env;
use clap::{Arg, App, AppSettings, ErrorKind, ArgMatches};
use pact_matching::models::PactSpecification;
use pact_verifier::*;
use log::LogLevelFilter;
use simplelog::TermLogger;
use std::str::FromStr;
use std::error::Error;
use regex::Regex;

fn main() {
    match handle_command_args() {
        Ok(_) => (),
        Err(err) => std::process::exit(err)
    }
}

fn print_version() {
    println!("\npact verifier version     : v{}", crate_version!());
    println!("pact specification version: v{}", PactSpecification::V1.version_str());
}

fn integer_value(v: String) -> Result<(), String> {
    v.parse::<u16>().map(|_| ()).map_err(|e| format!("'{}' is not a valid port value: {}", v, e) )
}

fn pact_source(matches: &ArgMatches) -> Vec<PactSource> {
    let mut sources = vec![];
    match matches.values_of("file") {
        Some(values) => sources.extend(values.map(|v| PactSource::File(s!(v))).collect::<Vec<PactSource>>()),
        None => ()
    };
    match matches.values_of("dir") {
        Some(values) => sources.extend(values.map(|v| PactSource::Dir(s!(v))).collect::<Vec<PactSource>>()),
        None => ()
    };
    match matches.values_of("url") {
        Some(values) => sources.extend(values.map(|v| PactSource::URL(s!(v))).collect::<Vec<PactSource>>()),
        None => ()
    };
    match matches.values_of("broker-url") {
        Some(values) => sources.extend(values.map(|v| PactSource::BrokerUrl(s!(matches.value_of("provider-name").unwrap()),
            s!(v))).collect::<Vec<PactSource>>()),
        None => ()
    };
    sources
}

fn interaction_filter(matches: &ArgMatches) -> FilterInfo {
    if matches.is_present("filter-description") &&
        (matches.is_present("filter-state") || matches.is_present("filter-no-state")) {
        if matches.is_present("filter-state") {
            FilterInfo::DescriptionAndState(s!(matches.value_of("filter-description").unwrap()),
                s!(matches.value_of("filter-state").unwrap()))
        } else {
            FilterInfo::DescriptionAndState(s!(matches.value_of("filter-description").unwrap()),
                s!(""))
        }
    } else if matches.is_present("filter-description") {
        FilterInfo::Description(s!(matches.value_of("filter-description").unwrap()))
    } else if matches.is_present("filter-state") {
        FilterInfo::State(s!(matches.value_of("filter-state").unwrap()))
    } else if matches.is_present("filter-no-state") {
        FilterInfo::State(s!(""))
    } else {
        FilterInfo::None
    }
}

fn handle_command_args() -> Result<(), i32> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let version = format!("v{}", crate_version!());
    let app = App::new(program)
        .version(version.as_str())
        .about("Standalone Pact verifier")
        .version_short("v")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("loglevel")
            .short("l")
            .long("loglevel")
            .takes_value(true)
            .use_delimiter(false)
            .possible_values(&["error", "warn", "info", "debug", "trace", "none"])
            .help("Log level (defaults to warn)"))
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .required_unless_one(&["dir", "url", "broker-url"])
            .takes_value(true)
            .use_delimiter(false)
            .multiple(true)
            .number_of_values(1)
            .empty_values(false)
            .help("Pact file to verify (can be repeated)"))
        .arg(Arg::with_name("dir")
            .short("d")
            .long("dir")
            .required_unless_one(&["file", "url", "broker-url"])
            .takes_value(true)
            .use_delimiter(false)
            .multiple(true)
            .number_of_values(1)
            .empty_values(false)
            .help("Directory of pact files to verify (can be repeated)"))
        .arg(Arg::with_name("url")
            .short("u")
            .long("url")
            .required_unless_one(&["file", "dir", "broker-url"])
            .takes_value(true)
            .use_delimiter(false)
            .multiple(true)
            .number_of_values(1)
            .empty_values(false)
            .help("URL of pact file to verify (can be repeated)"))
        .arg(Arg::with_name("broker-url")
            .short("b")
            .long("broker-url")
            .required_unless_one(&["file", "dir", "url"])
            .requires("provider-name")
            .takes_value(true)
            .use_delimiter(false)
            .multiple(true)
            .number_of_values(1)
            .empty_values(false)
            .help("URL of the pact broker to fetch pacts from to verify (requires the provider name parameter)"))
        .arg(Arg::with_name("hostname")
            .short("h")
            .long("hostname")
            .takes_value(true)
            .use_delimiter(false)
            .help("Provider hostname (defaults to localhost)"))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .takes_value(true)
            .use_delimiter(false)
            .help("Provider port (defaults to 8080)")
            .validator(integer_value))
        .arg(Arg::with_name("provider-name")
            .short("n")
            .long("provider-name")
            .takes_value(true)
            .use_delimiter(false)
            .help("Provider name (defaults to provider)"))
        .arg(Arg::with_name("state-change-url")
            .short("s")
            .long("state-change-url")
            .takes_value(true)
            .use_delimiter(false)
            .help("URL to post state change requests to"))
        .arg(Arg::with_name("state-change-as-query")
            .long("state-change-as-query")
            .help("State change request data will be sent as query parameters instead of in the request body"))
        .arg(Arg::with_name("state-change-teardown")
            .long("state-change-teardown")
            .help("State change teardown requests are to be made after each interaction"))
        .arg(Arg::with_name("filter-description")
            .long("filter-description")
            .takes_value(true)
            .use_delimiter(false)
            .validator(|val| Regex::new(&val)
                .map(|_| ())
                .map_err(|err| format!("'{}' is an invalid filter value: {}", val, err.description())))
            .help("Only validate interactions whose descriptions match this filter"))
        .arg(Arg::with_name("filter-state")
            .long("filter-state")
            .takes_value(true)
            .use_delimiter(false)
            .conflicts_with("filter-no-state")
            .validator(|val| Regex::new(&val)
                .map(|_| ())
                .map_err(|err| format!("'{}' is an invalid filter value: {}", val, err.description())))
            .help("Only validate interactions whose provider states match this filter"))
        .arg(Arg::with_name("filter-no-state")
            .long("filter-no-state")
            .conflicts_with("filter-state")
            .help("Only validate interactions that have no defined provider state"))
        .arg(Arg::with_name("filter-consumer")
            .short("c")
            .long("filter-consumer")
            .takes_value(true)
            .multiple(true)
            .empty_values(false)
            .help("Consumer name to filter the pacts to be verified (can be repeated)"))
        ;

    let matches = app.get_matches_safe();
    match matches {
        Ok(ref matches) => {
            let level = matches.value_of("loglevel").unwrap_or("warn");
            let log_level = match level {
                "none" => LogLevelFilter::Off,
                _ => LogLevelFilter::from_str(level).unwrap()
            };
            TermLogger::init(log_level).unwrap();
            let provider = ProviderInfo {
                host: s!(matches.value_of("hostname").unwrap_or("localhost")),
                port: matches.value_of("port").unwrap_or("8080").parse::<u16>().unwrap(),
                state_change_url: matches.value_of("state-change-url").map(|s| s.to_string()),
                state_change_body: !matches.is_present("state-change-as-query"),
                state_change_teardown: matches.is_present("state-change-teardown"),
                .. ProviderInfo::default()
            };
            let source = pact_source(matches);
            let filter = interaction_filter(matches);
            if verify_provider(&provider, source, &filter, &matches.values_of_lossy("filter-consumer").unwrap_or(vec![])) {
                Ok(())
            } else {
                Err(2)
            }
        },
        Err(ref err) => {
            match err.kind {
                ErrorKind::HelpDisplayed => {
                    println!("{}", err.message);
                    Ok(())
                },
                ErrorKind::VersionDisplayed => {
                    print_version();
                    println!("");
                    Ok(())
                },
                _ => {
                    println!("{}", err.message);
                    err.exit()
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use quickcheck::{TestResult, quickcheck};
    use rand::Rng;
    use super::integer_value;
    use expectest::prelude::*;

    #[test]
    fn validates_integer_value() {
        fn prop(s: String) -> TestResult {
            let mut rng = ::rand::thread_rng();
            if rng.gen() && s.chars().any(|ch| !ch.is_numeric()) {
                TestResult::discard()
            } else {
                let validation = integer_value(s.clone());
                match validation {
                    Ok(_) => TestResult::from_bool(!s.is_empty() && s.chars().all(|ch| ch.is_numeric() )),
                    Err(_) => TestResult::from_bool(s.is_empty() || s.chars().find(|ch| !ch.is_numeric() ).is_some())
                }
            }
        }
        quickcheck(prop as fn(_) -> _);

        expect!(integer_value(s!("1234"))).to(be_ok());
        expect!(integer_value(s!("1234x"))).to(be_err());
    }
}
