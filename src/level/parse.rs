use std::{iter::repeat, sync::atomic::AtomicUsize};

use ndarray::Array2;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, one_of, u32},
    combinator::map,
    error::{ParseError, VerboseError},
    multi::many1,
    sequence::{terminated, tuple},
    Finish, IResult,
};

use super::{Cell, Dir, Level, Levels, Coord};

static LEVEL_NUMBER: AtomicUsize = AtomicUsize::new(1);

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LevelLoadError {
    BadFormat,
    BadChecksum,
    BadHeader,
    InvalidCharacter,
}

impl<I> ParseError<I> for LevelLoadError {
    fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
        Self::BadFormat
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

pub(crate) fn parse_levels(input: &[u8]) -> Result<Levels, LevelLoadError> {
    levels(input).finish().map(|(_, levels)| levels)
}

fn levels(input: &[u8]) -> IResult<&[u8], Levels, LevelLoadError> {
    let (input, _) = tuple((tag(b"Mushroom Man 3.0"), line_ending::<_, VerboseError<_>>))(input)
        .map_err(|_| nom::Err::Error(LevelLoadError::BadHeader))?;
    let (input, checksum) = terminated(u32, line_ending::<_, VerboseError<_>>)(input)
        .map_err(|_| nom::Err::Error(LevelLoadError::BadFormat))?;
    let (input, _) = line_ending::<_, VerboseError<_>>(input)
        .map_err(|_| nom::Err::Error(LevelLoadError::BadFormat))?;
    let (input, levels) = many1::<_, _, LevelLoadError, _>(level)(input)?;
    // TODO: Calculate checksum and throw error if bad
    assert!(levels.len() > 99);
    log::info!("Found {} levels", levels.len());

    Ok((input, Levels { checksum, levels }))
}

fn level(input: &[u8]) -> IResult<&[u8], Level, LevelLoadError> {
    let (input, name) = map(
        terminated(not_line_ending, line_ending::<_, VerboseError<_>>),
        String::from_utf8_lossy,
    )(input)
    .map_err(|_| nom::Err::Error(LevelLoadError::BadFormat))?;
    let (input, author) = map(
        terminated(not_line_ending, line_ending::<_, VerboseError<_>>),
        String::from_utf8_lossy,
    )(input)
    .map_err(|_| nom::Err::Error(LevelLoadError::BadFormat))?;
    let (input, mut rows): (_, Vec<Vec<Cell>>) = many1(row)(input)?;
    let (input, _) = line_ending::<_, VerboseError<&[u8]>>(input)
        .map_err(|_| nom::Err::Error(LevelLoadError::BadFormat))?;

    let width = rows[0].len();
    rows.iter_mut().for_each(|row| {
        if row.len() < width {
            row.extend(repeat(Cell::Empty).take(width - row.len()));
        }
    });
    let map = Array2::from_shape_vec(
        (rows.len(), rows[0].len()),
        rows.into_iter().flatten().collect(),
    )
    .unwrap();
    let player_pos = Coord::new(
        map.indexed_iter()
            .find(|(_p, c)| c == &&Cell::Start)
            .expect("need a start")
            .0,
    );

    Ok((
        input,
        Level {
            name: name.to_string(),
            author: author.to_string(),
            number: LEVEL_NUMBER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            map,
            start_pos: player_pos,
            player_pos,
        },
    ))
}

fn row(input: &[u8]) -> IResult<&[u8], Vec<Cell>, LevelLoadError> {
    terminated(many1(cell), line_ending)(input)
}

fn cell(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    alt((
        wall, start, exit, bomb, cement, barrel, money, guard, hole, metal_wall, jelly_bean, key,
        lock, gun, oxygen, teleport, water, empty,
    ))(input)
}

fn wall(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"w"), |_| Cell::Wall)(input)
}

fn start(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"s"), |_| Cell::Start)(input)
}

fn exit(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"e"), |_| Cell::Exit)(input)
}

fn bomb(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"b"), |_| Cell::Bomb)(input)
}

fn cement(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"c"), |_| Cell::Cement)(input)
}

fn barrel(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"d"), |_| Cell::Barrel)(input)
}

fn money(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"f"), |_| Cell::Money)(input)
}

fn guard(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"g"), |_| Cell::Guard)(input)
}

fn hole(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"h"), |_| Cell::Hole)(input)
}

fn metal_wall(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"i"), |_| Cell::MetalWall)(input)
}

fn jelly_bean(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"j"), |_| Cell::JellyBean)(input)
}

fn key(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"k"), |_| Cell::Key)(input)
}

fn empty(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b" "), |_| Cell::Empty)(input)
}

fn water(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"~"), |_| Cell::Water)(input)
}

fn teleport(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(
        tuple((
            tag(b"t"),
            one_of("12345"),
            alt((
                map(tag(b"1"), |_| Dir::Up),
                map(tag(b"2"), |_| Dir::Down),
                map(tag(b"3"), |_| Dir::Left),
                map(tag(b"4"), |_| Dir::Right),
            )),
        )),
        |(_, id, dir)| Cell::Teleport(id as u8 - b'0' as u8, dir),
    )(input)
}

fn oxygen(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"o"), |_| Cell::Oxygen)(input)
}

fn gun(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"n"), |_| Cell::Gun)(input)
}

fn lock(input: &[u8]) -> IResult<&[u8], Cell, LevelLoadError> {
    map(tag(b"l"), |_| Cell::Lock)(input)
}
