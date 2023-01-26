use rand::SeedableRng;

use lustre::{cli, render::RenderContext};

fn main() {
    // Parsing cli args
    let cli_args = cli::parse_args();

    // set up enviroment
    let mut rng = if let Some(seed) = cli_args.seed {
        // use user-provided seed if available
        rand::rngs::SmallRng::seed_from_u64(seed)
    } else if cfg!(debug_assertions) {
        // if debugging, use deterministic seed
        rand::rngs::SmallRng::seed_from_u64(0)
    } else {
        // otherwise real psuedo-randomness
        rand::rngs::SmallRng::from_entropy()
    };

    let render_context = RenderContext::from_arguments(&cli_args, &mut rng);
    let img_buf = render_context.render();

    // write image to file
    match img_buf.save(&cli_args.output) {
        Ok(()) => println!("Image written to {:?}", &cli_args.output),
        Err(why) => {
            eprintln!("Failed to write: {why}");
        }
    }
}
