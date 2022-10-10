pub mod cat;
pub mod df;
pub mod ls;

use std::io::Error;
use std::str::FromStr;

#[derive(Debug)]
pub struct Options {
    pub filename: String,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Command {
    cat,
    df,
    ls,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "cat" => Ok(Command::cat),
            "df" => Ok(Command::df),
            "ls" => Ok(Command::ls),
            _ => Err(()),
        };
    }
}

impl Command {
    pub fn run_command(&self, options: &Options, args: Vec<String>) -> Result<(), Error> {
        match self {
            Command::cat => cat::cat(&options, args),
            Command::df => df::df(&options, args),
            Command::ls => ls::ls(&options, args),
        }
    }
}
