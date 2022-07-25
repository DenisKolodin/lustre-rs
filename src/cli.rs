//! Definition of command line arguments

// pub is neeeded for the program to called Arguments::parse()
pub use clap::Parser;

use crate::scenes::SceneType;

/// Toy RT Renderer
#[derive(Parser, Debug)]
pub struct Arguments {
    /// The path to the file to write the resulting image into
    #[clap(
        short,
        long,
        value_parser,
        value_name = "FILE",
        default_value = "output.png"
    )]
    pub output: std::path::PathBuf,

    /// The hardcoded scene to use
    #[clap(short, long, value_enum, default_value_t = SceneType::CoverPhoto)]
    pub scene: SceneType,

    /// samples per pixel
    ///
    /// A higher count of samples leads to higher visual fidelity
    #[clap(
        short = 'n',
        long = "samples",
        value_parser,
        default_value_t = 100,
        value_name = "NUM"
    )]
    pub samples_per_pixel: u32,
}
