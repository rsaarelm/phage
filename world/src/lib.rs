#![crate_name="world"]

extern crate rand;
extern crate rustc_serialize;
extern crate num;
extern crate vec_map;
extern crate calx;

pub use entity::{Entity};
pub use flags::{camera, set_camera, get_tick};
pub use location::{Location, Chart, Unchart};
pub use msg::{pop_msg};
pub use terrain::{TerrainType};
pub use world::{init_world, load, save};

macro_rules! msg(
    ($($arg:tt)*) => ( ::msg::push(::Msg::Text(format!($($arg)*))))
);

macro_rules! msgln(
    ($($arg:tt)*) => ({
        ::msg::push(::Msg::Text(format!($($arg)*)));
        ::msg::push(::Msg::Text("\n".to_string()));
    })
);

macro_rules! caption(
    ($($arg:tt)*) => ( ::msg::push(::Msg::Caption(format!($($arg)*))))
);

pub mod action;
pub mod components;
pub mod item;

mod ability;
mod area;
mod component_ref;
mod ecs;
mod entity;
mod flags;
mod geomorph;
mod geomorph_data;
mod location;
mod location_set;
mod mapgen;
mod msg;
mod prototype;
mod rng;
mod spatial;
mod spawn;
mod stats;
mod terrain;
mod world;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

/// Landscape type. Also serves as bit field in order to produce habitat masks
/// for entity spawning etc.
#[derive(Copy, Eq, PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Biome {
    Overland = 0b1,
    Base     = 0b10,

    // For things showing up at a biome.
    Anywhere = -1,
}

impl Biome {
    pub fn default_terrain(self) -> terrain::TerrainType {
        match self {
            _ => TerrainType::Rock,
        }
    }
}

#[derive(Copy, Eq, PartialEq, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct AreaSpec {
    pub biome: Biome,
    pub depth: i32,
}

impl AreaSpec {
    pub fn new(biome: Biome, depth: i32) -> AreaSpec {
        AreaSpec { biome: biome, depth: depth }
    }

    /// Return whether a thing with this spec can be spawned in an environment
    /// with the given spec.
    pub fn can_hatch_in(&self, environment: &AreaSpec) -> bool {
        self.depth >= 0 && self.depth <= environment.depth &&
        (self.biome as i32 & environment.biome as i32) != 0
    }
}

/// Various one-off signals the game sends to the UI layer.
#[derive(Clone, Debug)]
pub enum Msg {
    /// Regular event message
    Text(String),
    /// Important event message to the center of the screen
    Caption(String),
    // TODO: Type of effect.
    Explosion(Location),
    Damage(Entity),
    Gib(Location),
    Beam(Location, Location),
    /// Beam hitting a wall.
    Sparks(Location),
}

/// Light level value.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Light {
    lum: f32,
}

impl Light {
    pub fn new(lum: f32) -> Light {
        assert!(lum >= 0.0 && lum <= 2.0);
        Light { lum: lum }
    }

    pub fn apply(&self, color: &calx::Rgba) -> calx::Rgba {
        let darkness_color = calx::Rgba::new(0.05, 0.10, 0.25, color.a);
        calx::lerp(self.lum, *color * darkness_color, *color)
    }
}
