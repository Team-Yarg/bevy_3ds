use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::sync::Arc;

use bevy::app::Update;
use bevy::asset::{AssetId, AssetServer, Handle, LoadState};
use bevy::audio::{AudioBundle, AudioPlugin, AudioSource};
use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res, ResMut, Resource};
use bevy::math::{Quat, Vec2, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::render::color::Color;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::{CompressedImageFormats, Image};
use bevy::scene::{Scene, SceneBundle};
use bevy::sprite::{Sprite, SpriteBundle};
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    hierarchy::BuildChildren,
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        Style, Val,
    },
};
use image::{DynamicImage, Pixel, Rgba};

mod setup_logger;
mod shims;

const FRAME_W: u32 = 256;
const FRAME_H: u32 = 256;

#[derive(Component, Default)]
struct VideoPlayer {
    next_frame: Option<Handle<Image>>,
}

impl VideoPlayer {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Component)]
struct CornellBox;

const AUDIO_BYTES: &[u8] = include_bytes!("./assets/bad-apple.mp3");
const VIDEO_BYTES: &[u8] = include_bytes!("./assets/out.packed");

#[derive(Resource)]
struct VideoSource<R: Send> {
    src: R,
    frame: usize,
}
impl<R: Read + Send> VideoSource<R> {
    fn read_frame(&mut self) -> Option<DynamicImage> {
        const PACKED_FRAME_SIZE: u32 = (FRAME_H * FRAME_W) / 8;
        let mut buf = [0u8; PACKED_FRAME_SIZE as usize];
        self.src.read_exact(&mut buf).ok()?;
        let img = image::RgbaImage::from_fn(FRAME_W, FRAME_H, |x, y| {
            let big_idx = y * FRAME_W + x;
            let smol_idx = big_idx / 8;
            let bit = big_idx % 8;
            let packed = buf[smol_idx as usize];
            if (packed & (1 << bit)) > 0 {
                Pixel::from_channels(255, 255, 255, 255)
            } else {
                Pixel::from_channels(0, 0, 0, 255)
            }
        });
        self.frame += 1;
        Some(img.into())
    }
}

fn setup_video_player<R: Read + Send + Sync + 'static>(app: &mut App, src: R) {
    app.insert_resource(VideoSource { src, frame: 0 })
        .add_systems(Update, pull_next_frame::<R>);
}

fn main() {
    {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            std::fs::write("panic.log", info.to_string());
            prev(info)
        }));
    }
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    //setup_logger::setup_logger().unwrap();

    let mut app = App::new();
    app.add_plugins(bevy_3ds::DefaultPlugins);
    app.add_plugins(AudioPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(Update, (update, play_next_frame));
    setup_video_player(&mut app, Cursor::new(VIDEO_BYTES));
    unsafe {
        ctru_sys::osSetSpeedupEnable(true);
    }

    app.run();
}
fn update(time: Res<Time>, mut s: Query<(&mut Transform), With<CornellBox>>) {
    let velocity = PI / 4.0;
    for mut t in &mut s {
        t.rotate_around(
            Vec3::ZERO,
            Quat::from_rotation_y(velocity * time.delta_seconds()),
        );
    }
}

fn play_next_frame(
    assets: Res<AssetServer>,
    mut players: Query<(&mut Handle<Image>, &mut VideoPlayer)>,
) {
    for (mut img, mut player) in &mut players {
        if player.next_frame.is_none() {
            continue;
        }
        let id = player.next_frame.as_ref().unwrap().id();
        if matches!(assets.load_state(id), LoadState::Loaded) {
            let next = player.next_frame.take().unwrap();
            *img = next;
        }
    }
}

fn pull_next_frame<R: Read + Send + Sync + 'static>(
    mut video: ResMut<VideoSource<R>>,
    mut players: Query<&mut VideoPlayer>,
    assets: Res<AssetServer>,
) {
    let mut next_frame: Option<Handle<Image>> = None;
    for mut player in &mut players {
        if player.next_frame.is_some() {
            continue;
        }
        let next_frame = next_frame.get_or_insert_with(|| {
            let Some(frame) = video.read_frame() else {
                return Handle::Weak(AssetId::invalid());
            };
            /*frame.write_to(
                &mut BufWriter::new(
                    File::create(&format!("frames/frame-{}.webp", video.frame)).unwrap(),
                ),
                image::ImageFormat::WebP,
            );*/
            assets.add(Image::from_dynamic(frame, true))
        });
        player.next_frame.replace(next_frame.clone());
    }
}

fn setup(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn(AudioBundle {
        source: assets.add(AudioSource {
            bytes: AUDIO_BYTES.into(),
        }),
        ..Default::default()
    });

    cmds.spawn(Camera2dBundle {
        ..Default::default()
    });

    cmds.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(FRAME_W as f32, FRAME_H as f32)),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(VideoPlayer::new());
}
