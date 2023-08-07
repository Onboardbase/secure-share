use std::fs::{self};

use crate::Cli;
use anyhow::{Context, Result};
use tracing::Level;
use tracing_subscriber::{
    self,
    fmt::{format, writer::MakeWriterExt},
};

pub fn log(config: &Cli) -> Result<()> {
    let (level, source_location) = match config.debug {
        0 => (Level::INFO, false),
        _ => (Level::DEBUG, true),
    };

    let dirs = directories_next::ProjectDirs::from("com", "onboardbase", "secureshare").unwrap();
    let path = dirs.data_local_dir();
    let path = path.join("logs");
    fs::create_dir_all(&path).context("Failed to create logs directory")?;

    let file_appender =
        tracing_appender::rolling::hourly(path, "service.log").with_max_level(level);
    let stdout = std::io::stdout.with_max_level(level);

    // let mk_writer = std::io::stderr
    // .with_max_level(Level::ERROR)
    // .or_else(std::io::stdout
    //     .with_max_level(Level::INFO)
    //     .and(file_appender.with_max_level(Level::DEBUG))
    // );

    tracing_subscriber::fmt()
        .event_format(
            format()
                .pretty()
                .with_source_location(source_location)
                .with_target(source_location)
                .without_time(),
        )
        .with_writer(stdout.and(file_appender))
        .with_max_level(level)
        .init();

    Ok(())
}
