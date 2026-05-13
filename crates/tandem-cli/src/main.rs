#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod cli;

fn main() -> anyhow::Result<()> {
    cli::run()
}
