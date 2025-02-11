use clap::ArgMatches;

use super::utils::parse_pubkey;
use crate::{command::CliCommand, error::CliError};

pub fn admin_command(matches: &ArgMatches) -> Result<CliCommand, CliError> {
    match matches.subcommand() {
        Some(("task", matches)) => admin_task_command(matches),
        Some(("health", matches)) => admin_health_command(matches),
        Some(("initialize", _matches)) => Ok(CliCommand::AdminInitialize),
        _ => Err(CliError::CommandNotRecognized(
            matches.subcommand().unwrap().0.into(),
        )),
    }
}

fn admin_health_command(matches: &ArgMatches) -> Result<CliCommand, CliError> {
    match matches.subcommand() {
        Some(("reset", _matches)) => Ok(CliCommand::AdminHealthReset),
        _ => Err(CliError::CommandNotRecognized(
            matches.subcommand().unwrap().0.into(),
        )),
    }
}

fn admin_task_command(matches: &ArgMatches) -> Result<CliCommand, CliError> {
    match matches.subcommand() {
        Some(("cancel", matches)) => Ok(CliCommand::AdminTaskCancel {
            address: parse_pubkey(&"address".into(), matches)?,
        }),
        _ => Err(CliError::CommandNotRecognized(
            matches.subcommand().unwrap().0.into(),
        )),
    }
}
