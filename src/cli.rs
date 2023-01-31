//! Definition of command line arguments

use clap::Parser;

pub use clap_verbosity_flag::Verbosity;

use crate::scenes::SceneType;

/// Parses the commandline arguments into an [Arguments] struct
pub fn parse_args() -> Arguments {
    Arguments::parse()
}

/// Argument definitions for [clap::Parser]
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Arguments {
    /// The path to the file to write the resulting image into
    #[clap(
        short,
        long,
        value_parser = valid_image_file,
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

    #[clap(flatten)]
    pub verbosity: self::Verbosity,
}

/// Checks whether the given integer value is greater than 0
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

/// Checks whether the given output file is valid
///
/// Checks the following properties:
/// * a valid path (always the case)
/// * a supported image format
fn valid_image_file(s: &str) -> Result<std::path::PathBuf, String> {
    // &str -> PathBuf conversion is Infallible
    let path = s.parse::<std::path::PathBuf>().unwrap();
    match image::ImageFormat::from_path(&path).and_then(valid_image_format) {
        Ok(_) => Ok(path),
        Err(e) => Err(e.to_string()),
    }
}

/// Helper func for [valid_image_file] to check against compiled image formats
///
/// Since [image::ImageOutputFormat] conditionally compiles the supported formats,
/// use that existing functionality instead of manually parsing which formats
/// this crate supports against the feature flags of this crate.
fn valid_image_format(format: image::ImageFormat) -> image::ImageResult<()> {
    use image::{error, ImageOutputFormat};
    match ImageOutputFormat::from(format) {
        ImageOutputFormat::Unsupported(_) => Err(error::ImageError::Unsupported(
            error::UnsupportedError::from(error::ImageFormatHint::from(format)),
        )),
        _ => Ok(()),
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

    #[test]
    fn valid_output_file() {
        use clap::CommandFactory;
        // text files are not valid image files
        let res = Arguments::command().try_get_matches_from(["lustre", "--output", "bad.txt"]);

        assert!(res.is_err(), "Expected an error during argument parsing");

        assert_eq!(
            res.as_ref().unwrap_err().kind(),
            clap::error::ErrorKind::ValueValidation,
            "Expected an unrecognized image format error"
        );
    }
}
