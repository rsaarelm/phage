use std::default::Default;
use util::color::*;
use ecs::{Component, ComponentAccess};
use entity::{Entity};
use components::{Spawn, Category, IsPrototype};
use components::{Desc, MapMemory, Health};
use components::{Brain, BrainState, Alignment};
use stats::{Stats};
use stats::Intrinsic::*;
use Biome::*;
use world;

#[derive(Copy)]
pub struct Prototype {
    pub target: Entity
}

impl Prototype {
    pub fn new(parent: Option<Entity>) -> Prototype {
        Prototype {
            target: world::with_mut(|w| {
                let e = w.ecs.new_entity(parent);
                w.prototypes_mut().insert(e, IsPrototype);
                e
            }),
        }
    }
}

impl<C: Component> Fn<(C,)> for Prototype {
    type Output = Prototype;
    extern "rust-call" fn call(&self, (comp,): (C,)) -> Prototype {
        comp.add_to(self.target);
        *self
    }
}

/// Only call at world init!
pub fn init() {
    let base_mob = Prototype::new(None)
        (Brain { state: BrainState::Asleep, alignment: Alignment::Indigenous })
        ({let h: Health = Default::default(); h})
        .target;

    // Init the prototypes

    // Player
    Prototype::new(Some(base_mob))
        (Brain { state: BrainState::PlayerControl, alignment: Alignment::Phage })
        (Desc::new("player", 40, CYAN))
        (Stats::new(2, &[]).attack(2))
        (MapMemory::new())
        ;

    // Enemies
    Prototype::new(Some(base_mob))
        (Desc::new("hopper", 32, YELLOW))
        (Stats::new(2, &[]))
        (Spawn::new(Category::Mob))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("colonist", 34, DARKORANGE))
        (Stats::new(4, &[Hands]))
        (Spawn::new(Category::Mob).biome(Base))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("marine", 36, DARKOLIVEGREEN))
        (Stats::new(6, &[Hands]))
        (Spawn::new(Category::Mob).biome(Base))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("rumbler", 38, OLIVE))
        (Stats::new(8, &[]))
        (Spawn::new(Category::Mob))
        ;
}
