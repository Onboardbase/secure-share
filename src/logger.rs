use std::fs::{self};

use crate::Cli;
use anyhow::{Context, Result};
use tracing::Level;
use tracing_subscriber::{
    self,
    fmt::{format, writer::MakeWriterExt},
};

pub fn log(opts: &Cli) -> Result<()> {
    let level = match opts.debug {
        0 => Level::INFO,
        _ => Level::DEBUG,
    };

    let dirs = directories_next::ProjectDirs::from("build", "woke", "wokeshare").unwrap();
    let path = dirs.data_local_dir();
    let path = path.join("logs");
    fs::create_dir_all(&path).context("Failed to create logs directory")?;

    let file_appender =
        tracing_appender::rolling::hourly(path, "service.log").with_max_level(Level::DEBUG);
    let stdout = std::io::stdout.with_max_level(level);

    // let mk_writer = std::io::stderr
    // .with_max_level(Level::ERROR)
    // .or_else(std::io::stdout
    //     .with_max_level(Level::INFO)
    //     .and(file_appender.with_max_level(Level::DEBUG))
    // );

    tracing_subscriber::fmt()
        .event_format(format().pretty().with_source_location(false))
        .with_writer(stdout.and(file_appender))
        .init();

    Ok(())
}
