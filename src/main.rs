use rand::SeedableRng;

use crate::{
    cli::{Arguments, Parser},
    tree::Tree,
};

mod bounds;
mod bvh;
mod camera;
mod cli;
mod color;
mod hittables;
mod material;
mod ray;
mod render;
mod scenes;
mod textures;
mod tree;
mod utils;

fn main() {
    // Parsing cli args
    let cli_args = Arguments::parse();

    // Set up image properties
    let img_w = 1200;

    // set up enviroment
    let mut rng = if cfg!(debug_assertions) {
        // if debugging, use deterministic seed
        rand::rngs::SmallRng::seed_from_u64(0)
    } else {
        // otherwise real psuedo-randomness
        rand::rngs::SmallRng::from_entropy()
    };

    // Get scene
    let (cam, world, dimensions) = scenes::get_scene(img_w, cli_args.scene, &mut rng);
    let world = Tree::new(world, 0.0, 1.0);

    let renderer = render::Renderer::new(
        dimensions.0,
        dimensions.1,
        cli_args.samples_per_pixel,
        cli_args.bounce_depth,
    );

    let img_buf = renderer.render_scene((cam, world));

    // write image to file
    match img_buf.save(&cli_args.output) {
        Ok(()) => println!("Image written to {:?}", &cli_args.output),
        Err(why) => {
            eprintln!("Failed to write: {}", why);
        }
    }
}
