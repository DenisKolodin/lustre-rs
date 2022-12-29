//! Definition of command line arguments

// pub is neeeded for the program to called Arguments::parse()
pub use clap::Parser;

use crate::scenes::SceneType;

/// Argument definitions for [clap::Parser]
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Arguments {
    /// The path to the file to write the resulting image into
    #[clap(
        short,
        long,
        value_parser,
        default_value = "output.png",
        value_name = "FILE"
    )]
    pub output: std::path::PathBuf,

    /// The width of the generated image
    #[clap(
        short = 'w',
        long = "width",
        value_parser = valid_count::<u32>,
        default_value_t = 1200,
        value_name = "NUM"
    )]
    pub image_width: u32,

    /// samples per pixel
    ///
    /// A higher count of samples leads to higher visual fidelity due to more rays sent for a pixel
    #[clap(
        short = 'n',
        long = "samples",
        value_parser = valid_count::<u32>,
        default_value_t = 100,
        value_name = "NUM"
    )]
    pub samples_per_pixel: u32,

    /// number of light contribution bounces
    ///
    /// A higher number of bounces leads to higher visual fidelity due to more accurate gathered light
    #[clap(
        short,
        long = "bounces",
        value_parser = valid_count::<u16>,
        default_value_t = 50,
        value_name = "NUM"
    )]
    pub bounce_depth: u16,

    /// The hardcoded scene to use
    #[clap(short, long, value_enum, default_value_t = SceneType::CoverPhoto)]
    pub scene: SceneType,

    /// The seed used for psuedorandom number generation
    #[clap(long)]
    pub seed: Option<u64>,
}

fn valid_count<T>(s: &str) -> Result<T, String>
where
    T: num_traits::PrimInt + std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match s.parse::<T>() {
        Ok(count) => {
            if count > T::zero() {
                Ok(count)
            } else {
                Err("count must be greater than 0".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_cli() {
        use clap::CommandFactory;
        Arguments::command().debug_assert()
    }
}
