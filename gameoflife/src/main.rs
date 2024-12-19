use bevy::color::{color_difference::EuclideanDistance, palettes::css};
use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const PIXEL_SIZE: u32 = 10;
const IMAGE_WIDTH: u32 = 100;
const IMAGE_HEIGHT: u32 = 100;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, draw)
        .run();
}

#[derive(Resource)]
struct MyProcGenImage(Handle<Image>);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    let mut image = Image::new_fill(
        Extent3d {
            width: IMAGE_WIDTH,
            height: IMAGE_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &(css::WHITE.to_u8_array()),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let handle = images.add(image);

    for x in 0..IMAGE_WIDTH {
        for y in 0..IMAGE_HEIGHT {
            if rand::random() {
                image.set_color_at(x, y, Color::WHITE);
            } else {
                image.set_color_at(x, y, Color::BLACK);
            }
        }
    }

    commands.spawn(Sprite::from_image(handle.clone()));
    commands.insert_resource(MyProcGenImage(handle));
}

fn draw(my_handle: Res<MyProcGenImage>, mut images: ResMut<Assets<Image>>) {
    let image = images.get_mut(&my_handle.0).expect("Image not found");

    // for i in 0..IMAGE_WIDTH * IMAGE_HEIGHT {
    //     update_image(image, i);
    // }
}

fn update_image(image: &mut Image, i: u32) {
    let x = i % IMAGE_WIDTH;
    let y = i / IMAGE_WIDTH;

    for i in 0..PIXEL_SIZE {
        for j in 0..PIXEL_SIZE {
            let x = x * PIXEL_SIZE + i;
            let y = y * PIXEL_SIZE + j;

            let mut neighbors = 0;

            for i in -1..=1 {
                for j in -1..=1 {
                    if i == 0 && j == 0 {
                        continue;
                    }

                    let x = x as i32 + i;
                    let y = y as i32 + j;

                    if x < 0 || x >= IMAGE_WIDTH as i32 || y < 0 || y >= IMAGE_HEIGHT as i32 {
                        continue;
                    }

                    let x = x as u32;
                    let y = y as u32;

                    if image.get_color_at(x, y).unwrap() == Color::BLACK {
                        neighbors += 1;
                    }
                }
            }

            if image.get_color_at(x, y).unwrap() == Color::BLACK {
                if neighbors < 2 || neighbors > 3 {
                    image.set_color_at(x, y, Color::WHITE).unwrap();
                }
            } else {
                if neighbors == 3 {
                    image.set_color_at(x, y, Color::BLACK).unwrap();
                }
            }
        }
    }
}
