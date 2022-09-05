use clap::Parser;

/// RSearch searx instances randomizer
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ArgsClap {
    /// Download frontend directory
    #[clap(short, long, value_parser)]
    pub download: bool,
}

pub fn parse() -> ArgsClap {
    ArgsClap::parse()
}
