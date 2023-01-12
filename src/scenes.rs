//! Scene generation functionality

use std::{path::PathBuf, sync::Arc};

use glam::Vec3A;
use rand::Rng;

use crate::{
    camera::Camera, color::Color, hittables::*, material::Material, textures::*, tree::Tree,
};

/// Possible hard-coded scenes to choose from.
#[derive(Debug, Clone, Copy, clap::clap_derive::ValueEnum)]
pub enum SceneType {
    /// Test scene for materials development
    MaterialDev,
    /// Scene like the cover of "Ray Tracing in One Weekend".
    CoverPhoto,
    /// Two checkered spheres with the camera looking at their point of contact
    TwoSpheres,
    /// Two Perlin noise spheres
    TwoPerlinSpheres,
    /// A single sphere with an image of Earth mapped to it
    Earth,
    /// [SceneType::TwoPerlinSpheres] with a rectangular diffuse light
    SimpleLight,
    /// The famous [Cornell Box scene](https://en.wikipedia.org/wiki/Cornell_box)
    CornellBox,
    /// Cornell Box scene from the [definitive Cornell Box data](https://www.graphics.cornell.edu/online/box/data.html)
    CornellBox2,
    /// The [SceneType::CoverPhoto] in the dark with lights
    RandomLights,
    /// The Final Scene from Ray Tracing in One Weekend: The Next Week
    FinalScene,
    /// Debugging Cornell Box Scene
    DebugCornell,
    /// Debugging Final Scene from Book 2
    DebugFinal,
}

/// Returns a [Camera], a list of objects ([HittableList]), and the image dimensions as a tuple.
pub fn get_scene(
    image_width: u32,
    scene_type: SceneType,
    rng: &mut impl Rng,
) -> (Camera, HittableList, (u32, u32)) {
    // Setup default camera properties
    // uncomment the `mut` once its needed
    let mut aspect_ratio = 16.0 / 9.0;
    let mut look_from = Vec3A::new(13.0, 2.0, 3.0);
    let mut look_at = Vec3A::ZERO;
    let /* mut */ view_up = Vec3A::Y;
    let mut vert_fov = 20.0;
    let mut aperture = 0.0;
    let mut focus_dist = 10.0;
    let /* mut */ shutter_time = 0.0..1.0;
    let mut bg_color = Color::new(Vec3A::new(0.7, 0.8, 1.0));

    // Grabs the scene and changes any cam params
    let scene = match scene_type {
        SceneType::MaterialDev => {
            aspect_ratio = 16.0 / 9.0;
            look_from = Vec3A::ZERO;
            look_at = -Vec3A::Z;
            focus_dist = 1.0;
            vert_fov = 90.0;
            get_mat_dev_scene()
        }
        SceneType::CoverPhoto => {
            aperture = 0.1;
            aspect_ratio = 3.0 / 2.0;
            gen_random_scene(rng)
        }

        SceneType::TwoSpheres => gen_two_spheres(),
        SceneType::TwoPerlinSpheres => gen_two_perlin_spheres(),
        SceneType::Earth => gen_earth(),
        SceneType::SimpleLight => {
            bg_color = Color::new(Vec3A::ZERO);
            look_from = Vec3A::new(26.0, 3.0, 6.0);
            look_at = Vec3A::new(0.0, 2.0, 0.0);
            gen_simple_light()
        }
        SceneType::CornellBox => {
            aspect_ratio = 1.0;
            bg_color = Color::new(Vec3A::ZERO);
            look_from = Vec3A::new(278.0, 278.0, -800.0);
            look_at = Vec3A::new(278.0, 278.0, 0.0);
            vert_fov = 40.0;
            gen_cornell_box()
        }
        SceneType::CornellBox2 => {
            aspect_ratio = 1.0;
            bg_color = Color::new(Vec3A::ZERO);
            look_from = Vec3A::new(278.0, 278.0, -800.0);
            look_at = Vec3A::new(278.0, 278.0, 0.0);
            vert_fov = 40.0;
            gen_cornell_box2()
        }
        SceneType::RandomLights => {
            aperture = 0.1;
            aspect_ratio = 3.0 / 2.0;
            bg_color = Color::new(Vec3A::from(bg_color) / 10.0);
            gen_emissive_random(rng)
        }
        SceneType::FinalScene => {
            aspect_ratio = 1.0;
            bg_color = Color::new(Vec3A::ZERO);
            look_from = Vec3A::new(478.0, 278.0, -600.0);
            look_at = Vec3A::new(278.0, 278.0, 0.0);
            vert_fov = 40.0;
            focus_dist = look_from.distance(look_at);
            // TODO this is patch, what is the real bug?
            aperture = focus_dist.recip();
            gen_book2_scene(rng)
        }
        SceneType::DebugCornell => {
            aspect_ratio = 1.0;
            bg_color = Color::new(Vec3A::ZERO);
            look_from = Vec3A::new(278.0, 278.0, -800.0);
            look_at = Vec3A::new(278.0, 278.0, 0.0);
            vert_fov = 40.0;
            gen_debug_scene()
        }
        SceneType::DebugFinal => {
            aspect_ratio = 1.0;
            bg_color = Color::new(Vec3A::ZERO);
            look_from = Vec3A::new(478.0, 278.0, -600.0);
            // pointing at the group of random white spheres
            look_at = Vec3A::new(1.0, 353.0, 453.0);
            // narrower field of view to "zoom-in" to the spheres
            vert_fov = 20.0;
            focus_dist = look_from.distance(look_at);
            aperture = focus_dist.recip();
            gen_debug2_scene(rng)
        }
    };

    // set up camera with (possibly modified) properies
    let cam = Camera::new(
        look_from,
        look_at,
        view_up,
        vert_fov,
        aspect_ratio,
        aperture,
        focus_dist,
        shutter_time,
        bg_color,
    );

    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let dimensions = (image_width, image_height);

    (cam, scene, dimensions)
}

/// Returns a [HittableList] containing a few spheres with unique materials
fn get_mat_dev_scene() -> HittableList {
    //  Create ground sphere
    let ground_material = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.8, 0.2, 0.2))),
    });
    let ground_sph = Sphere::new(Vec3A::new(0.0, -1000.5, 0.0), 1000.0, &ground_material);

    let mat_left = Arc::new(Material::Dielectric { refract_index: 1.5 });
    let mat_right = Arc::new(Material::Metal {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.8, 0.6, 0.2))),
        roughness: 0.1,
    });
    let mat_center = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.1, 0.2, 0.5))),
    });

    let left_sph = Sphere::new(Vec3A::new(-1.0, 0.0, -1.0), 0.5, &mat_left);
    let right_sph = Sphere::new(Vec3A::new(1.0, 0.0, -1.0), 0.5, &mat_right);
    let center_sph = Sphere::new(Vec3A::new(0.0, 0.0, -1.0), 0.5, &mat_center);

    vec![
        ground_sph.wrap(),
        left_sph.wrap(),
        right_sph.wrap(),
        center_sph.wrap(),
    ]
}

/// Returns a [HittableList] containing randomly-generated spheres
fn gen_random_scene(rng: &mut impl Rng) -> HittableList {
    //  Create ground sphere
    let ground_material = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE / 2.0)),
    });
    let mut world: HittableList =
        vec![Sphere::new(Vec3A::new(0.0, -1000.0, 0.0), 1000.0, &ground_material).wrap()];

    // The random generation part
    const ORIGIN: Vec3A = Vec3A::from_array([4.0, 0.2, 0.0]);
    for a in -11..11 {
        for b in -11..11 {
            let center = Vec3A::new(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );

            if (center - ORIGIN).length() > 0.9 {
                let decide_mat = rng.gen::<f32>();
                // pick a material by "rarity"
                let mat = if (0.0..0.8).contains(&decide_mat) {
                    // diffuse
                    let rand_color_v = rng.gen::<Vec3A>() * rng.gen::<Vec3A>();
                    let albedo = Arc::new(SolidColor::new(rand_color_v));
                    Arc::new(Material::Lambertian { albedo })
                } else if (0.0..0.95).contains(&decide_mat) {
                    // metal
                    Arc::new(Material::Metal {
                        albedo: Arc::new(SolidColor::new(rng.gen())),
                        roughness: rng.gen(),
                    })
                } else {
                    // glass
                    Arc::new(Material::Dielectric { refract_index: 1.5 })
                };

                // make the diffuse spheres moveable
                if let Material::Lambertian { .. } = mat.as_ref() {
                    let center2 = center + Vec3A::Y * rng.gen_range(0.0..0.5);
                    let sph = MovingSphere::new(center, center2, 0.0, 1.0, 0.2, &mat);
                    world.push(sph.wrap())
                } else {
                    let sph = Sphere::new(center, 0.2, &mat);
                    world.push(sph.wrap())
                }
            }
        }
    }

    // The signature central spheres
    let mat_1 = Arc::new(Material::Dielectric { refract_index: 1.5 });
    let mat_2 = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.4, 0.2, 0.1))),
    });
    let mat_3 = Arc::new(Material::Metal {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.7, 0.6, 0.5))),
        roughness: 0.0,
    });

    world.push(Sphere::new(Vec3A::new(0.0, 1.0, 0.0), 1.0, &mat_1).wrap());
    world.push(Sphere::new(Vec3A::new(-4.0, 1.0, 0.0), 1.0, &mat_2).wrap());
    world.push(Sphere::new(Vec3A::new(4.0, 1.0, 0.0), 1.0, &mat_3).wrap());

    world
}

/// Returns a [HittableList] containing two checkered spheres.
fn gen_two_spheres() -> HittableList {
    let checkered = Arc::new(Material::Lambertian {
        albedo: Arc::new(Checkered::new(
            &Arc::new(SolidColor::new(Vec3A::new(0.9, 0.9, 0.9))),
            &Arc::new(SolidColor::new(Vec3A::new(0.2, 0.3, 0.1))),
        )),
    });

    vec![
        Sphere::new(Vec3A::new(0.0, -10.0, 0.0), 10.0, &checkered).wrap(),
        Sphere::new(Vec3A::new(0.0, 10.0, 0.0), 10.0, &checkered).wrap(),
    ]
}

/// Returns a [HittableList] containing two Perlin noise spheres.
fn gen_two_perlin_spheres() -> HittableList {
    let perlin_tex = Arc::new(Material::Lambertian {
        albedo: Arc::new(NoiseTexture::new(::noise::Perlin::default(), 4.0)),
    });

    vec![
        Sphere::new(Vec3A::new(0.0, -1000.0, 0.0), 1000.0, &perlin_tex).wrap(),
        Sphere::new(Vec3A::new(0.0, 2.0, 0.0), 2.0, &perlin_tex).wrap(),
    ]
}

/// Returns a [HittableList] containing a single image-backed sphere.
fn gen_earth() -> HittableList {
    let earth_tex = Arc::new(Material::Lambertian {
        albedo: Arc::new(ImageMap::new(PathBuf::from("resources/earthmap.jpg"))),
    });

    vec![Sphere::new(Vec3A::ZERO, 2.0, &earth_tex).wrap()]
}

/// Returns a [HittableList] resembling [gen_two_perlin_spheres], with a rectangular diffuse light
fn gen_simple_light() -> HittableList {
    let diff_light = Arc::new(Material::DiffuseLight {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE)),
        brightness: 4.0,
    });

    let mut world = gen_two_perlin_spheres();
    world.push(
        Quad::from_two_points_z(
            Vec3A::new(3.0, 1.0, 0.0),
            Vec3A::new(5.0, 3.0, 0.0),
            -2.0,
            &diff_light,
        )
        .wrap(),
    );

    world
}

/// The Cornell Box scene as defined by the Ray Tracing in One Weekend: The Next Week
fn gen_cornell_box() -> HittableList {
    let red_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.65, 0.05, 0.05))),
    });
    let white_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::splat(0.73))),
    });
    let green_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.12, 0.45, 0.15))),
    });
    let light = Arc::new(Material::DiffuseLight {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE)),
        brightness: 15.0,
    });

    // yz rect - zero x
    let left_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 555.0, 0, &red_diffuse);

    // yz rect - zero x
    let right_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 0.0, 0, &green_diffuse);

    // xz rect - zero y
    let light_rec = Quad::from_bounds_k(213.0, 343.0, 227.0, 332.0, 554.9, 1, &light);

    // xz rect - zero y
    let bottom_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 0.0, 1, &white_diffuse);

    // xz rect - zero y
    let top_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 555.0, 1, &white_diffuse);

    // xy rect - zero z
    let back_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 555.0, 2, &white_diffuse);

    let squarish_box: Arc<dyn Hittable> =
        QuadBox::new(Vec3A::ZERO, Vec3A::splat(165.0), &white_diffuse).wrap();
    let squarish_box = Transform::new(&squarish_box)
        .with_axis_angle_degrees(glam::Vec3::Y, -18.0)
        .with_translation(glam::Vec3::new(130.0, 0.0, 65.0))
        .finalize();

    let tall_box: Arc<dyn Hittable> =
        QuadBox::new(Vec3A::ZERO, Vec3A::new(165.0, 330.0, 165.0), &white_diffuse).wrap();
    let tall_box = Transform::new(&tall_box)
        .with_axis_angle_degrees(glam::Vec3::Y, 15.0)
        .with_translation(glam::Vec3::new(265.0, 0.0, 295.0))
        .finalize();

    vec![
        left_side.wrap(),
        right_side.wrap(),
        bottom_side.wrap(),
        top_side.wrap(),
        back_side.wrap(),
        light_rec.wrap(),
        squarish_box.wrap(),
        tall_box.wrap(),
    ]
}

/// The cornell box scene as defined from the original physical measurements
fn gen_cornell_box2() -> HittableList {
    // materials
    let red_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.65, 0.05, 0.05))),
    });
    let white_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::splat(0.73))),
    });
    let green_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.12, 0.45, 0.15))),
    });
    let light = Arc::new(Material::DiffuseLight {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE)),
        brightness: 12.0,
    });

    let _mirror_like = Arc::new(Material::Metal {
        albedo: Arc::new(SolidColor::new(Vec3A::splat(0.999))),
        roughness: 0.0,
    });
    let _glass_like = Arc::new(Material::Dielectric { refract_index: 1.5 });

    // walls
    let floor = Quad::new(
        Vec3A::new(552.8, 0.0, 0.0),
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(0.0, 0.0, 559.2),
        Vec3A::new(549.6, 0.0, 559.2),
        &white_diffuse,
    );

    let light_quad = Quad::new(
        Vec3A::new(343.0, 548.8, 227.0),
        Vec3A::new(343.0, 548.8, 332.0),
        Vec3A::new(213.0, 548.8, 332.0),
        Vec3A::new(213.0, 548.8, 227.0),
        &light,
    );

    let ceiling = Quad::new(
        Vec3A::new(556.0, 548.8, 0.0),
        Vec3A::new(556.0, 548.8, 559.2),
        Vec3A::new(0.0, 548.8, 559.2),
        Vec3A::new(0.0, 548.8, 0.0),
        &white_diffuse,
    );

    let back_wall = Quad::new(
        Vec3A::new(549.6, 0.0, 559.2),
        Vec3A::new(0.0, 0.0, 559.2),
        Vec3A::new(0.0, 548.8, 559.2),
        Vec3A::new(556.0, 548.8, 559.2),
        &white_diffuse,
    );

    let right_wall = Quad::new(
        Vec3A::new(0.0, 0.0, 559.2),
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(0.0, 548.8, 0.0),
        Vec3A::new(0.0, 548.8, 559.2),
        &green_diffuse,
    );

    let left_wall = Quad::new(
        Vec3A::new(552.8, 0.0, 0.0),
        Vec3A::new(549.6, 0.0, 559.2),
        Vec3A::new(556.0, 548.8, 559.2),
        Vec3A::new(556.0, 548.8, 0.0),
        &red_diffuse,
    );

    // boxes
    let short_box: HittableList = vec![
        Quad::new(
            Vec3A::new(130.0, 165.0, 65.0),
            Vec3A::new(82.0, 165.0, 225.0),
            Vec3A::new(240.0, 165.0, 272.0),
            Vec3A::new(290.0, 165.0, 114.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(290.0, 0.0, 114.0),
            Vec3A::new(290.0, 165.0, 114.0),
            Vec3A::new(240.0, 165.0, 272.0),
            Vec3A::new(240.0, 0.0, 272.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(130.0, 0.0, 65.0),
            Vec3A::new(130.0, 165.0, 65.0),
            Vec3A::new(290.0, 165.0, 114.0),
            Vec3A::new(290.0, 0.0, 114.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(82.0, 0.0, 225.0),
            Vec3A::new(82.0, 165.0, 225.0),
            Vec3A::new(130.0, 165.0, 65.0),
            Vec3A::new(130.0, 0.0, 65.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(240.0, 0.0, 272.0),
            Vec3A::new(240.0, 165.0, 272.0),
            Vec3A::new(82.0, 165.0, 225.0),
            Vec3A::new(82.0, 0.0, 225.0),
            &white_diffuse,
        )
        .wrap(),
    ];

    let tall_box: HittableList = vec![
        Quad::new(
            Vec3A::new(423.0, 330.0, 247.0),
            Vec3A::new(265.0, 330.0, 296.0),
            Vec3A::new(314.0, 330.0, 456.0),
            Vec3A::new(472.0, 330.0, 406.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(423.0, 0.0, 247.0),
            Vec3A::new(423.0, 330.0, 247.0),
            Vec3A::new(472.0, 330.0, 406.0),
            Vec3A::new(472.0, 0.0, 406.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(472.0, 0.0, 406.0),
            Vec3A::new(472.0, 330.0, 406.0),
            Vec3A::new(314.0, 330.0, 456.0),
            Vec3A::new(314.0, 0.0, 456.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(314.0, 0.0, 456.0),
            Vec3A::new(314.0, 330.0, 456.0),
            Vec3A::new(265.0, 330.0, 296.0),
            Vec3A::new(265.0, 0.0, 296.0),
            &white_diffuse,
        )
        .wrap(),
        Quad::new(
            Vec3A::new(265.0, 0.0, 296.0),
            Vec3A::new(265.0, 330.0, 296.0),
            Vec3A::new(423.0, 330.0, 247.0),
            Vec3A::new(423.0, 0.0, 247.0),
            &white_diffuse,
        )
        .wrap(),
    ];
    vec![
        floor.wrap(),
        light_quad.wrap(),
        ceiling.wrap(),
        back_wall.wrap(),
        right_wall.wrap(),
        left_wall.wrap(),
        short_box.wrap(),
        tall_box.wrap(),
    ]
}

fn box_helper() -> HittableList {
    let red_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.65, 0.05, 0.05))),
    });
    let white_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::splat(0.73))),
    });
    let green_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.12, 0.45, 0.15))),
    });
    let light = Arc::new(Material::DiffuseLight {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE)),
        brightness: 15.0,
    });

    // yz rect - zero x
    let left_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 555.0, 0, &red_diffuse);
    // yz rect - zero x
    let right_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 0.0, 0, &green_diffuse);
    // xz rect - zero y
    let light_rec = Quad::from_bounds_k(213.0, 343.0, 227.0, 332.0, 554.9, 1, &light);
    // xz rect - zero y
    let bottom_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 0.0, 1, &white_diffuse);
    // xz rect - zero y
    let top_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 555.0, 1, &white_diffuse);
    // xy rect - zero z
    let back_side = Quad::from_bounds_k(0.0, 555.0, 0.0, 555.0, 555.0, 2, &white_diffuse);

    vec![
        left_side.wrap(),
        right_side.wrap(),
        bottom_side.wrap(),
        top_side.wrap(),
        back_side.wrap(),
        light_rec.wrap(),
    ]
}

fn gen_debug_scene() -> HittableList {
    let mut world_box = box_helper();
    let white_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::splat(0.73))),
    });
    let close_sphere = Sphere::new(Vec3A::new(212.5, 82.5, 147.5), 82.5, &white_diffuse);
    let far_sphere = Sphere::new(Vec3A::new(347.5, 165.0, 377.5), 82.5, &white_diffuse);

    world_box.push(close_sphere.wrap());
    world_box.push(far_sphere.wrap());
    world_box
}

/// Returns a [HittableList] containing randomly-generated spheres, some emissive
fn gen_emissive_random(rng: &mut impl Rng) -> HittableList {
    // the set of objects with estimated capacity
    let mut world: HittableList = Vec::with_capacity(4 + (-11..11).len().pow(2));

    //  Create ground sphere
    let ground_material = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE / 2.0)),
    });

    let ground = Sphere::new(Vec3A::new(0.0, -1000.0, 0.0), 1000.0, &ground_material);
    world.push(ground.wrap());

    // The random generation part
    const ORIGIN: Vec3A = Vec3A::from_array([4.0, 0.2, 0.0]);
    for a in -11..11 {
        for b in -11..11 {
            let center = Vec3A::new(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );

            if (center - ORIGIN).length() > 0.9 {
                let decide_mat = rng.gen();
                // pick a material by "rarity"
                let mat = if (0.0..0.75).contains(&decide_mat) {
                    // diffuse
                    let rand_color_v = rng.gen::<Vec3A>() * rng.gen::<Vec3A>();
                    let albedo = Arc::new(SolidColor::new(rand_color_v));
                    Arc::new(Material::Lambertian { albedo })
                } else if (0.0..0.85).contains(&decide_mat) {
                    // metal
                    Arc::new(Material::Metal {
                        albedo: Arc::new(SolidColor::new(rng.gen())),
                        roughness: rng.gen(),
                    })
                } else if (0.0..0.90).contains(&decide_mat) {
                    // emissive
                    Arc::new(Material::DiffuseLight {
                        albedo: Arc::new(SolidColor::new(rng.gen())),
                        brightness: rng.gen_range(2.0..10.0),
                    })
                } else {
                    // glass
                    Arc::new(Material::Dielectric { refract_index: 1.5 })
                };

                let sph = Sphere::new(center, 0.2, &mat);
                world.push(sph.wrap())
            }
        }
    }

    // The signature central spheres
    let mat_1 = Arc::new(Material::Dielectric { refract_index: 1.5 });
    let sphere_1 = Sphere::new(Vec3A::new(0.0, 1.0, 0.0), 1.0, &mat_1);

    let mat_2 = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.4, 0.2, 0.1))),
    });
    let sphere_2 = Sphere::new(Vec3A::new(-4.0, 1.0, 0.0), 1.0, &mat_2);

    let mat_3 = Arc::new(Material::Metal {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.7, 0.6, 0.5))),
        roughness: 0.0,
    });
    let sphere_3 = Sphere::new(Vec3A::new(4.0, 1.0, 0.0), 1.0, &mat_3);

    world.push(sphere_1.wrap());
    world.push(sphere_2.wrap());
    world.push(sphere_3.wrap());

    world
}

/// The scene defined at the end of the second book for Ray Tracing in One Weekend
fn gen_book2_scene(rng: &mut impl Rng) -> HittableList {
    let mut ground_boxes: HittableList = vec![];
    let ground_mat = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.48, 0.83, 0.53))),
    });

    // step value
    let w = 100.0;

    // first y coord
    let y0 = 0.0;

    // 20 x 20 set of boxes
    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        // x coords
        let x0 = -1000.0 + i as f32 * w;
        let x1 = x0 + w;

        for j in 0..boxes_per_side {
            // 2nd y coord
            let y1 = rng.gen_range(1.0..101.0);

            // z coords
            let z0 = -1000.0 + j as f32 * w;
            let z1 = z0 + w;

            ground_boxes.push(
                QuadBox::new(Vec3A::new(x0, y0, z0), Vec3A::new(x1, y1, z1), &ground_mat).wrap(),
            )
        }
    }

    // BVH-ify the ground boxes
    let mut all_objects: HittableList = vec![Tree::new(ground_boxes, 0.0, 1.0).wrap()];

    // light
    let light_mat = Arc::new(Material::DiffuseLight {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE)),
        brightness: 7.0,
    });
    all_objects.push(Quad::from_bounds_k(123.0, 423.0, 147.0, 412.0, 554.0, 1, &light_mat).wrap());

    // horizontally moving sphere
    let center1 = Vec3A::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3A::X * 30.0;
    let moving_sphere_mat = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::new(0.7, 0.3, 0.1))),
    });
    all_objects
        .push(MovingSphere::new(center1, center2, 0.0, 1.0, 50.0, &moving_sphere_mat).wrap());

    // glassy sphere
    all_objects.push(
        Sphere::new(
            Vec3A::new(260.0, 150.0, 45.0),
            50.0,
            &Arc::new(Material::Dielectric { refract_index: 1.5 }),
        )
        .wrap(),
    );

    // metallic sphere
    all_objects.push(
        Sphere::new(
            Vec3A::new(0.0, 150.0, 145.0),
            50.0,
            &Arc::new(Material::Metal {
                albedo: Arc::new(SolidColor::new(Vec3A::new(0.8, 0.8, 0.9))),
                roughness: 1.0,
            }),
        )
        .wrap(),
    );

    // boundary for sub-surface object
    let boundary = Sphere::new(
        Vec3A::new(360.0, 150.0, 145.0),
        70.0,
        &Arc::new(Material::Dielectric { refract_index: 1.5 }),
    );

    let wrapped_boundary: Arc<dyn Hittable> = boundary.wrap();

    // sub-surface object interior
    let subsurface_tex: Arc<dyn Texture> = Arc::new(SolidColor::new(Vec3A::new(0.2, 0.4, 0.9)));
    all_objects.push(ConstantMedium::new(&wrapped_boundary, &subsurface_tex, 0.2).wrap());
    all_objects.push(wrapped_boundary);

    // boundary for world mist/fog
    let mist_boundary: Arc<dyn Hittable> = Sphere::new(
        Vec3A::ZERO,
        5000.0,
        &Arc::new(Material::Dielectric { refract_index: 1.5 }),
    )
    .wrap();

    // mist
    let mist_tex: Arc<dyn Texture> = Arc::new(SolidColor::new(Vec3A::ONE));
    all_objects.push(ConstantMedium::new(&mist_boundary, &mist_tex, 0.00001).wrap());

    // earth sphere
    let earth_mat = Arc::new(Material::Lambertian {
        albedo: Arc::new(ImageMap::new(PathBuf::from("resources/earthmap.jpg"))),
    });
    all_objects.push(Sphere::new(Vec3A::new(400.0, 200.0, 400.0), 100.0, &earth_mat).wrap());

    // perlin noise sphere
    let perlin_mat = Arc::new(Material::Lambertian {
        albedo: Arc::new(NoiseTexture::new(::noise::Perlin::default(), 0.5)),
    });
    all_objects.push(Sphere::new(Vec3A::new(220.0, 280.0, 300.0), 90.0, &perlin_mat).wrap());

    // group of white spheres
    let whiteish_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::splat(0.73))),
    });
    let rand_sphere_group: HittableList = std::iter::repeat_with(|| -> Arc<dyn Hittable> {
        Sphere::new(rng.gen::<Vec3A>() * 165.0, 10.0, &whiteish_diffuse).wrap()
    })
    .take(1000)
    .collect();

    let wrapped_spheres: Arc<dyn Hittable> = Tree::new(rand_sphere_group, 0.0, 1.0).wrap();
    all_objects.push(
        Transform::new(&wrapped_spheres)
            .with_axis_angle_degrees(glam::Vec3::Y, 15.0)
            .with_translation(glam::Vec3::new(-100.0, 270.0, 395.0))
            .finalize()
            .wrap(),
    );

    all_objects
}

// Like [gen_book2_scene], but only the light and random white sphere group
fn gen_debug2_scene(rng: &mut impl Rng) -> HittableList {
    let mut all_objects: Vec<Arc<dyn Hittable>> = vec![];
    // light
    let light_mat = Arc::new(Material::DiffuseLight {
        albedo: Arc::new(SolidColor::new(Vec3A::ONE)),
        brightness: 7.0,
    });
    all_objects.push(Quad::from_bounds_k(123.0, 423.0, 147.0, 412.0, 554.0, 1, &light_mat).wrap());

    // group of white spheres
    let whiteish_diffuse = Arc::new(Material::Lambertian {
        albedo: Arc::new(SolidColor::new(Vec3A::splat(0.73))),
    });
    let rand_sphere_group: HittableList = std::iter::repeat_with(|| -> Arc<dyn Hittable> {
        Sphere::new(rng.gen::<Vec3A>() * 165.0, 10.0, &whiteish_diffuse).wrap()
    })
    .take(1000)
    .collect();

    let wrapped_spheres: Arc<dyn Hittable> = Tree::new(rand_sphere_group, 0.0, 1.0).wrap();

    // eprintln!(
    //     "centroid of wrapped spheres: {}",
    //     wrapped_spheres.bounding_box(0.0, 1.0).unwrap().centroid()
    // );

    let transformed_spheres = Transform::new(&wrapped_spheres)
        .with_axis_angle_degrees(glam::Vec3::Y, 15.0)
        .with_translation(glam::Vec3::new(-100.0, 270.0, 395.0))
        .finalize();

    // eprintln!(
    //     "centroid of transformed spheres: {}",
    //     transformed_spheres
    //         .bounding_box(0.0, 1.0)
    //         .unwrap()
    //         .centroid()
    // );

    all_objects.push(transformed_spheres.wrap());

    all_objects
}
