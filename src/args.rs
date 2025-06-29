use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Resource, Debug, Clone)]
pub struct Args {
    /// runs the game in debug mode
    #[clap(long, alias = "dbg")]
    pub debug: bool,
}
