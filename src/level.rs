use std::ops::{ControlFlow, Deref, DerefMut};

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use bevy_asset_loader::prelude::*;
use ndarray::{Array2, Axis};

use crate::{
    events::{DeathEvent, LevelEvent, MovementEvent, SoundEvent},
    image::TextureData,
    GameState, LevelRoot, Player,
};

use self::parse::parse_levels;

mod parse;

const CELL_WIDTH: f32 = 32.0;

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

    // TODO: Pass player inventory here, to see if they can pass
    pub fn action(&self) -> CellAction {
        match self {
            Cell::Empty => CellAction::Nothing,
            Cell::Wall => CellAction::Block,
            Cell::Start => CellAction::Nothing,
            Cell::Exit => CellAction::NextLevel,
            Cell::Bomb => CellAction::Explode,
            Cell::Cement => CellAction::Add(Item::Cement, 1),
            Cell::Barrel => CellAction::Block,
            Cell::Money => CellAction::Add(Item::Money, 1),
            Cell::Guard => CellAction::Consume {
                item: Item::Money,
                fail: Box::new(CellAction::Block),
                success: Box::new(CellAction::Nothing),
            },
            Cell::Hole => CellAction::Consume {
                item: Item::Cement,
                fail: Box::new(CellAction::Die("You fell in a hole!")),
                success: Box::new(CellAction::Nothing),
            },
            Cell::MetalWall => CellAction::Block,
            Cell::JellyBean => CellAction::Push,
            Cell::Key => CellAction::Add(Item::Key, 1),
            Cell::Lock => CellAction::Consume {
                item: Item::Key,
                fail: Box::new(CellAction::Block),
                success: Box::new(CellAction::Nothing),
            },
            Cell::Gun => CellAction::Shoot,
            Cell::Oxygen => CellAction::Add(Item::Oxygen, 3),
            Cell::Teleport(n, d) => CellAction::Teleport(*n, *d),
            Cell::Water => CellAction::Consume {
                item: Item::Oxygen,
                fail: Box::new(CellAction::Die("You drowned!")),
                success: Box::new(CellAction::Nothing),
            },
        }
    }

    pub fn construct(
        &self,
        builder: &mut ChildBuilder,
        coord: Coord,
        atlas: Handle<TextureAtlas>,
    ) -> Entity {
        let e = builder.spawn(SpriteSheetBundle {
            transform: Transform::from_xyz(
                coord.0 .1 as f32 * CELL_WIDTH,
                coord.0 .0 as f32 * -CELL_WIDTH,
                0.0,
            ),
            texture_atlas: atlas.clone(),
            sprite: TextureAtlasSprite::new(self.indices()[0]),
            ..Default::default()
        });
        // TODO: Add arrows as children for teleporters
        // Also, add timers for animations
        // OR: we could just have arrows ON the teleporters...
        // But then they won't be animated
        e.id()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum CellAction {
    #[default]
    Nothing,
    Consume {
        item: Item,
        fail: Box<CellAction>,
        success: Box<CellAction>, // Should we pause in cases of bomb blowing up wall?
    },
    Add(Item, usize),
    Block,
    Push,
    Explode,
    Shoot,
    Teleport(u8, Dir),
    Die(&'static str),
    NextLevel,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Key,
    // Gun,
    Oxygen,
    Cement,
    Money,
    // Bomb,
}

#[derive(Debug, Clone, Resource, Default)]
pub struct Inventory {
    pub(crate) map: HashMap<Item, usize>,
}

#[derive(Debug, Default, Clone)]
pub struct Level {
    name: String,
    author: String,
    pub(crate) number: usize,
    pub(crate) map: Array2<Cell>,
    pub(crate) start_pos: Coord,
    pub(crate) player_pos: Coord,
}

impl Level {
    pub fn explode_cells(&self, c: Coord) -> Vec<Coord> {
        let (height, width) = self.map.dim();
        itertools::iproduct!([-1, 0, 1], [-1, 0, 1])
            .map(move |(dy, dx)| (dx + c.0 .0 as isize, dy + c.0 .1 as isize))
            .filter(|(i, j)| i >= &0 && j >= &0)
            .filter(move |(j, i)| j < &(height as isize) && i < &(width as isize))
            .map(|(i, j)| (i as usize, j as usize))
            .filter(move |c| match self.map[*c] {
                // Ignore Metal walls and water
                Cell::MetalWall | Cell::Water => false,
                _ => true,
            })
            .map(|c| Coord::new(c))
            .collect()
    }

    pub fn neighbor(&self, c: Coord, delta: (isize, isize)) -> Option<Coord> {
        let (height, width) = self.map.dim();
        let j = c.0 .0 as isize + delta.0;
        let i = c.0 .1 as isize + delta.1;
        if (j < 0) || (i < 0) || (j >= height as isize) || (i >= width as isize) {
            return None;
        }
        Some(Coord::new((j as usize, i as usize)))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord((usize, usize));

impl Coord {
    fn new(pos: (usize, usize)) -> Self {
        Self(pos)
    }
}

impl Deref for Coord {
    type Target = (usize, usize);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Coord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(dead_code)]
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

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut)]
pub struct LevelMap(Level);

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut)]
pub struct LevelEntities(HashMap<Coord, Entity>);

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Levels>()
            .init_asset_loader::<LevelsLoader>()
            .insert_resource(CurrentLevel(0))
            .insert_resource(LevelMap::default())
            .insert_resource(LevelEntities::default())
            .insert_resource(Inventory::default())
            .add_collection_to_loading_state::<_, LevelData>(GameState::Loading)
            .add_system(start_game.in_schedule(OnEnter(GameState::Playing)))
            .add_system(load_level.in_set(OnUpdate(GameState::Playing)))
            .add_system(move_player.in_set(OnUpdate(GameState::Playing)));
    }
}

fn start_game(mut commands: Commands) {
    commands.insert_resource(CurrentLevel(0));
}

pub(crate) fn load_level(
    mut commands: Commands,
    mut level_events: EventReader<LevelEvent>,
    level_data: Res<LevelData>,
    levels: Res<Assets<Levels>>,
    texture_data: Res<TextureData>,
    root: Query<(Entity, &LevelRoot)>,
    mut window: Query<&mut Window>,
) {
    // if !(current_level.is_added() || current_level.is_changed()) {
    //     return;
    // }

    for LevelEvent(current_level) in level_events.iter() {
        let (root, _) = root
            .get_single()
            .expect("Always have a root outside of this system");
        let levels = levels
            .get(&level_data.handle)
            .expect("Only loaded levels by this point");
        let mut level = levels.levels[*current_level].clone();
        level.player_pos = level.start_pos;
        let mut window = window.single_mut();
        window.title = format!("Level: {}, by {}", level.name, level.author);
        let (height, width) = level.map.dim();

        log::info!("Changing level: {}", *current_level);
        log::info!("Width x Height: {width} x {height}");
        log::info!("Player start: {:?}", level.start_pos);

        let mut entities = HashMap::default();
        let atlas = texture_data.atlas.clone();
        commands.insert_resource(LevelMap(level.clone()));
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
                            let es = cell.construct(parent, Coord::new((j, i)), atlas.clone());
                            entities.insert(Coord::new((j, i)), es);
                        });
                    });
                parent.spawn((
                    SpriteSheetBundle {
                        transform: Transform::from_xyz(
                            level.player_pos.1 as f32 * CELL_WIDTH,
                            (*level.player_pos).0 as f32 * -CELL_WIDTH,
                            1.0,
                        ),
                        texture_atlas: atlas.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        ..Default::default()
                    },
                    Player,
                ));
            });

        commands.insert_resource(LevelEntities(entities));
    }
}

fn move_player(
    mut commands: Commands,
    mut movements: EventReader<MovementEvent>,
    mut level_map: ResMut<LevelMap>,
    mut level_entities: ResMut<LevelEntities>,
    mut inventory: ResMut<Inventory>,
    mut transform: Query<&mut Transform, With<Player>>,
    root: Query<Entity, With<LevelRoot>>,
    texture_data: Res<TextureData>,
    mut death_events: EventWriter<DeathEvent>,
    mut _sound_events: EventWriter<SoundEvent>,
    mut level_events: EventWriter<LevelEvent>,
) {
    for dest in &mut movements {
        let dest = dest.pos;
        let root = root.single();
        let atlas = texture_data.atlas.clone();
        let cell = level_map.map[*dest];
        let delta = (
            dest.0 .0 as isize - level_map.player_pos.0 .0 as isize,
            dest.0 .1 as isize - level_map.player_pos.0 .1 as isize,
        );
        match cell.action() {
            CellAction::Nothing => {}
            CellAction::Consume {
                item,
                fail,
                success,
            } => {
                if let ControlFlow::Break(_) = handle_consume(
                    &mut inventory,
                    &mut death_events,
                    &mut level_map,
                    &mut level_entities,
                    &mut commands,
                    root,
                    atlas,
                    dest,
                    item,
                    fail,
                    success,
                ) {
                    return;
                }
            }
            CellAction::Add(item, amount) => {
                handle_add(
                    &mut level_map,
                    &mut level_entities,
                    &mut inventory,
                    &mut commands,
                    root,
                    atlas,
                    dest,
                    item,
                    amount,
                );
            }
            CellAction::Block => {
                // *dest = *level_map.player_pos;
                return;
            }
            CellAction::Explode => {
                handle_explode(
                    &mut level_map,
                    &mut level_entities,
                    &mut death_events,
                    &mut commands,
                    root,
                    atlas,
                    dest,
                );
                // TODO: Add animations
            }
            CellAction::Shoot => {
                handle_shoot(
                    &mut level_map,
                    &mut level_entities,
                    &mut death_events,
                    &mut commands,
                    root,
                    atlas,
                    dest,
                    delta,
                );
            }
            CellAction::Push => {
                if let ControlFlow::Break(_) = handle_push(
                    &mut level_map,
                    &mut level_entities,
                    &mut commands,
                    root,
                    atlas,
                    dest,
                    delta,
                ) {
                    return;
                }
            }
            CellAction::Teleport(_, _) => todo!(),
            CellAction::Die(msg) => {
                death_events.send(DeathEvent(msg.to_string()));
            }
            CellAction::NextLevel => {
                level_events.send(LevelEvent(level_map.number + 1));
            }
        }

        *level_map.player_pos = *dest;
        let mut transform = transform.single_mut();
        *transform = Transform::from_xyz(
            level_map.player_pos.0 .1 as f32 * CELL_WIDTH,
            level_map.player_pos.0 .0 as f32 * -CELL_WIDTH,
            1.0,
        );
    }
}

fn handle_consume(
    inventory: &mut Inventory,
    death_events: &mut EventWriter<DeathEvent>,
    level_map: &mut LevelMap,
    level_entities: &mut LevelEntities,
    commands: &mut Commands,
    root: Entity,
    atlas: Handle<TextureAtlas>,
    dest: Coord,
    item: Item,
    fail: Box<CellAction>,
    success: Box<CellAction>,
) -> ControlFlow<()> {
    let count = inventory.map.entry(item).or_insert(0);
    // TODO: Make the inventory consumtion work
    if *count == 0 {
        match *fail {
            CellAction::Block => {
                // *dest = *level_map.player_pos;
                return ControlFlow::Break(());
            }
            CellAction::Die(msg) => {
                death_events.send(DeathEvent(msg.to_string()));
                return ControlFlow::Break(());
            }
            _ => panic!("unhandled failure"),
        }
    } else {
        *count -= 1;
        // TODO: Delete object with effect, and perform action.
        level_map.map[*dest] = Cell::Empty;
        let entity = level_entities
            .get_mut(&dest)
            .expect("should have all positions");
        commands.entity(entity.clone()).despawn_recursive();
        commands.entity(root).with_children(|parent| {
            *entity = Cell::Empty.construct(parent, dest, atlas);
        });
        match *success {
            CellAction::Nothing => {}
            CellAction::Block => {
                // TODO: Make this explode
                // *dest = *level_map.player_pos;
                return ControlFlow::Break(());
            }
            _ => panic!("unhandled success"),
        }
    }
    ControlFlow::Continue(())
}

fn handle_add(
    level_map: &mut LevelMap,
    level_entities: &mut LevelEntities,
    inventory: &mut Inventory,
    commands: &mut Commands,
    root: Entity,
    atlas: Handle<TextureAtlas>,
    dest: Coord,
    item: Item,
    amount: usize,
) {
    level_map.map[*dest] = Cell::Empty;
    *inventory.map.entry(item).or_insert(0) += amount;
    let entity = level_entities
        .get_mut(&dest)
        .expect("should have all positions");
    commands.entity(entity.clone()).despawn_recursive();
    commands.entity(root).with_children(|parent| {
        *entity = Cell::Empty.construct(parent, dest, atlas);
    });
}

fn handle_explode(
    level_map: &mut LevelMap,
    level_entities: &mut LevelEntities,
    death_events: &mut EventWriter<DeathEvent>,
    commands: &mut Commands,
    root: Entity,
    atlas: Handle<TextureAtlas>,
    dest: Coord,
) {
    let explode_cells = level_map.explode_cells(dest);
    explode_cells.iter().for_each(|dest| {
        let old = &mut level_map.map[**dest];
        if old == &Cell::Barrel {
            death_events.send(DeathEvent("You died in an explosion".to_string()));
            return;
        }
        if old == &Cell::Exit {
            death_events.send(DeathEvent("You blew up the exit".to_string()));
            return;
        }
        *old = Cell::Empty;
        let entity = level_entities
            .get_mut(&dest)
            .expect("should have all positions");
        commands.entity(entity.clone()).despawn_recursive();
        commands.entity(root).with_children(|parent| {
            *entity = Cell::Empty.construct(parent, *dest, atlas.clone());
        });
    });
}

fn handle_shoot(
    level_map: &mut LevelMap,
    level_entities: &mut LevelEntities,
    death_events: &mut EventWriter<DeathEvent>,
    commands: &mut Commands,
    root: Entity,
    atlas: Handle<TextureAtlas>,
    dest: Coord,
    delta: (isize, isize),
) {
    let old = &mut level_map.map[*dest];
    if old == &Cell::Barrel {
        death_events.send(DeathEvent("You died in an explosion".to_string()));
        return;
    }
    if old == &Cell::Exit {
        death_events.send(DeathEvent("You blew up the exit".to_string()));
        return;
    }
    *old = Cell::Empty;
    let entity = level_entities
        .get_mut(&dest)
        .expect("should have all positions");
    commands.entity(entity.clone()).despawn_recursive();
    commands.entity(root).with_children(|parent| {
        *entity = Cell::Empty.construct(parent, dest, atlas.clone());
    });
    let neighbor = level_map.neighbor(dest, delta);
    if let Some(neighbor) = neighbor {
        level_map.map[*neighbor] = Cell::Empty;
        let entity = level_entities
            .get_mut(&neighbor)
            .expect("should have all positions");
        commands.entity(entity.clone()).despawn_recursive();
        commands.entity(root).with_children(|parent| {
            *entity = Cell::Empty.construct(parent, neighbor, atlas.clone());
        });
    }
}

fn handle_push(
    level_map: &mut LevelMap,
    level_entities: &mut LevelEntities,
    commands: &mut Commands,
    root: Entity,
    atlas: Handle<TextureAtlas>,
    dest: Coord,
    delta: (isize, isize),
) -> ControlFlow<()> {
    let neighbor = level_map.neighbor(dest, delta);
    if let Some(neighbor) = neighbor {
        let cell = level_map.map[*neighbor];
        if cell != Cell::Empty && cell != Cell::Start {
            // *dest.0 = *level_map.player_pos;
            return ControlFlow::Break(());
        }
        level_map.map[*dest] = Cell::Empty;
        let entity = level_entities
            .get_mut(&dest)
            .expect("should have all positions");
        commands.entity(entity.clone()).despawn_recursive();
        commands.entity(root).with_children(|parent| {
            *entity = Cell::Empty.construct(parent, dest, atlas.clone());
        });
        level_map.map[*neighbor] = Cell::JellyBean;
        let entity = level_entities
            .get_mut(&neighbor)
            .expect("should have all positions");
        commands.entity(entity.clone()).despawn_recursive();
        commands.entity(root).with_children(|parent| {
            *entity = Cell::JellyBean.construct(parent, neighbor, atlas.clone());
        });
    }
    ControlFlow::Continue(())
}
