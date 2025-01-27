use bevy::{color::palettes::css::GREEN, prelude::*};
use rand::Rng;
use rand_chacha::ChaCha12Rng;

#[derive(Component, Debug, Clone, Copy)]
pub struct Food {
    pub pos_x: f32,
    pub pos_y: f32,
}

impl Food {
    pub fn new(
        seeded_rng: &mut ChaCha12Rng,
        game_config: &crate::common::GameConfig,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> (
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
        crate::common::Collider,
        Food,
    ) {
        let x = seeded_rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0);
        let y = seeded_rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0);

        return (
            Mesh2d(meshes.add(Rectangle::new(15., 15.))),
            MeshMaterial2d(materials.add(Color::from(GREEN))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            crate::common::Collider,
            Food { pos_x: x, pos_y: y },
        );
    }
}
