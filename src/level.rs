use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
use bevy_asset_loader::prelude::*;
use ndarray::{Array2, Axis};

use crate::{image::TextureData, GameState, LevelRoot, Player};

use self::parse::parse_levels;

mod parse;

// Case "b"
//     SetSq Pos, "bomb"
// Case "c"
//     SetSq Pos, "cement"
// Case "d"
//     SetSq Pos, "barrel"
// Case "e"
//     SetSq Pos, "exit"
// Case "f"
//     SetSq Pos, "money"
// Case "g"
//     SetSq Pos, "guard"
// Case "h"
//     SetSq Pos, "hole"
// Case "i"
//     SetSq Pos, "metalwall"
// Case "j"
//     SetSq Pos, "jellybean"
// Case "k"
//     SetSq Pos, "key"
// Case "l"
//     SetSq Pos, "lock"
// Case "n"
//     SetSq Pos, "gun"
// Case "o"
//     SetSq Pos, "oxygen"
// Case "s"
//     SetSq Pos, "mushman"
//     DrawLevel = Pos
// Case "t"
//     'Get teleport number
//     Which = Val(Mid(Map(N), O + 1, 1))
//     Dir = Val(Mid(Map(N), O + 2, 1))
//     CreateTeleport Pos, Which, Dir
//     TelesInRow = TelesInRow + 1
// Case "w"
//     SetSq Pos, "wall"
// Case "~"
//     SetSq Pos, "water"
// Case "1" To "5"
//     'Teleport codes (ignored)
// Case Else
// End Select
//     'Unknown object (ignored)

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    #[default]
    Empty,
    Wall,
    Start,
    Exit,
    Bomb,
    Cement,
    Barrel,
    Money,
    Guard,
    Hole,
    MetalWall,
    JellyBean,
    Key,
    Lock,
    Gun,
    Oxygen,
    Teleport(u8, Dir),
    Water,
}

impl Cell {
    pub fn indices(&self) -> &[usize] {
        match self {
            Cell::Empty => &[26],
            Cell::Wall => &[6],
            Cell::Start => &[26],
            Cell::Exit => &[5],
            Cell::Bomb => &[1],
            Cell::Cement => &[20],
            Cell::Barrel => &[10],
            Cell::Money => &[13],
            Cell::Guard => &[14],
            Cell::Hole => &[4],
            Cell::MetalWall => &[7],
            Cell::JellyBean => &[11],
            Cell::Key => &[2],
            Cell::Lock => &[3],
            Cell::Gun => &[9],
            Cell::Oxygen => &[19],
            Cell::Teleport(num, _) => match num {
                1 => &[15, 16, 17],
                2 => &[21, 22, 23],
                3 => &[27, 28, 29],
                4 => &[33, 34, 35],
                5 => &[39, 40, 41],
                _ => panic!("Invalid teleport number"),
            },
            Cell::Water => &[8],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Default, Clone)]
pub struct Level {
    name: String,
    author: String,
    map: Array2<Cell>,
    start_pos: (usize, usize),
}

#[derive(Debug, Clone, Default, Resource, TypeUuid, AssetCollection)]
#[uuid = "509449b4-8862-4b9c-ad82-ff8e0a2cbd15"]
pub struct Levels {
    checksum: u32,
    levels: Vec<Level>,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct LevelsLoader;

impl AssetLoader for LevelsLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let levels = parse_levels(bytes)
                .map_err(|e| bevy::asset::Error::msg(format!("Error loading levels: {e:?}")))?;
            load_context.set_default_asset(LoadedAsset::new(levels));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["dat"]
    }
}

#[derive(Debug, Clone, Default, Resource, AssetCollection)]
pub struct LevelData {
    #[asset(path = "levels.dat")]
    handle: Handle<Levels>,
}

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut)]
pub struct CurrentLevel(usize);

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Levels>()
            .init_asset_loader::<LevelsLoader>()
            .insert_resource(CurrentLevel(0))
            .add_collection_to_loading_state::<_, LevelData>(GameState::Loading)
            .add_system(start_game.in_schedule(OnEnter(GameState::Playing)))
            .add_system(load_level.in_set(OnUpdate(GameState::Playing)));
    }
}

fn start_game(mut commands: Commands) {
    commands.insert_resource(CurrentLevel(0));
}

fn load_level(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    level_data: Res<LevelData>,
    levels: Res<Assets<Levels>>,
    texture_data: Res<TextureData>,
    root: Query<(Entity, &LevelRoot)>,
    mut window: Query<&mut Window>,
) {
    if !(current_level.is_added() || current_level.is_changed()) {
        return;
    }

    let (root, _) = root
        .get_single()
        .expect("Always have a root outside of this system");
    let levels = levels
        .get(&level_data.handle)
        .expect("Only loaded levels by this point");
    let level = &levels.levels[**current_level];
    let mut window = window.single_mut();
    window.title = format!("Level: {}, by {}", level.name, level.author);
    let (height, width) = level.map.dim();

    log::info!("Changing level: {}", **current_level);
    log::info!("Width x Height: {width} x {height}");

    let atlas = texture_data.atlas.clone();
    commands.entity(root).despawn_recursive();
    commands
        .spawn((LevelRoot, SpatialBundle::default()))
        .with_children(|parent| {
            level
                .map
                .axis_iter(Axis(0))
                .enumerate()
                .for_each(|(j, row)| {
                    row.iter().enumerate().for_each(|(i, cell)| {
                        parent.spawn((SpriteSheetBundle {
                            transform: Transform::from_xyz(i as f32 * 32.0, j as f32 * -32.0, 0.0),
                            texture_atlas: atlas.clone(),
                            sprite: TextureAtlasSprite::new(cell.indices()[0]),
                            ..Default::default()
                        },));
                    });
                });
            parent.spawn((
                SpriteSheetBundle {
                    transform: Transform::from_xyz(
                        level.start_pos.1 as f32 * 32.0,
                        level.start_pos.0 as f32 * -32.0,
                        0.0,
                    ),
                    texture_atlas: atlas.clone(),
                    sprite: TextureAtlasSprite::new(0),
                    ..Default::default()
                },
                Player,
            ));
        });
}
