mod actions;

use actions::slowloris::main::do_loris;
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
        .action(do_loris)
        .flag(
            Flag::new("verbose", FlagType::Bool)
                .description("Verbose logging flag")
                .alias("v"),
        )
        .flag(
            Flag::new("socket", FlagType::Int)
                .description("socket count")
                .alias("s"),
        )
        .flag(
            Flag::new("port", FlagType::Int)
                .description("port")
                .alias("p"),
        )
        .flag(
            Flag::new("httponly", FlagType::Bool)
                .description("use http connection")
                .alias("ho"),
        )
        .flag(
            Flag::new("proxy_file_path", FlagType::String)
                .description("proxy file path")
                .alias("pf"),
        );
    app.run(args);
    Ok(())
}
