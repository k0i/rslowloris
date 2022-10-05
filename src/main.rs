mod actions;

use actions::slowloris::slowloris;
use anyhow::Result;
use seahorse::{App, Flag, FlagType};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("rloris [OPTIONS] [ARGS]")
        .action(slowloris)
        .flag(
            Flag::new("verbose", FlagType::Bool)
                .description("Verbose logging flag")
                .alias("v"),
        )
        .flag(
            Flag::new("socket", FlagType::Int)
                .description("socket count")
                .alias("s"),
        );
    app.run(args);
    Ok(())
}
