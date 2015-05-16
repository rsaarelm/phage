use std::default::Default;
use calx::color::*;
use ecs::{Component, ComponentAccess};
use entity::{Entity};
use components::{Spawn, Category, IsPrototype};
use components::{Desc, MapMemory, Health};
use components::{Brain, BrainState, Alignment, Colonist};
use stats::{Stats};
use stats::Intrinsic::*;
use Biome::*;
use world;

#[derive(Copy, Clone)]
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

    /// Add a component to the prototype.
    pub fn c<C: Component>(self, component: C) -> Prototype {
        component.add_to(self.target);
        self
    }
}


/// Only call at world init!
pub fn init() {
    let base_mob = Prototype::new(None)
        .c(Brain { state: BrainState::Asleep, alignment: Alignment::Indigenous })
        .c({let h: Health = Default::default(); h})
        .target;

    let colonist = Prototype::new(None)
        .c(Brain { state: BrainState::Asleep, alignment: Alignment::Colonist })
        .c({let h: Health = Default::default(); h})
        .target;

    // Init the prototypes

    // Player
    Prototype::new(Some(base_mob))
        .c(Brain { state: BrainState::PlayerControl, alignment: Alignment::Phage })
        .c(Desc::new("phage", 40, CYAN))
        .c(Stats::new(2, &[Fast]).attack(3))
        .c(MapMemory::new())
        ;

    // Enemies

    // Indigenous
    Prototype::new(Some(base_mob))
        .c(Desc::new("hopper", 32, YELLOW))
        .c(Stats::new(4, &[]).protection(-2))
        .c(Spawn::new(Category::Mob).commonness(2000))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("stalker", 60, ORCHID))
        .c(Stats::new(4, &[]))
        .c(Spawn::new(Category::Mob))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("metawasp", 58, ORANGERED))
        // Glass cannon
        .c(Stats::new(4, &[Fast]).protection(-1).attack(2))
        .c(Spawn::new(Category::Mob).commonness(600))
        ;

    // Can open doors, good for base attack.
    Prototype::new(Some(base_mob))
        .c(Desc::new("space monkey", 46, LAWNGREEN))
        .c(Stats::new(6, &[Hands]))
        .c(Spawn::new(Category::Mob).commonness(600))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("rumbler", 38, OLIVE))
        .c(Stats::new(8, &[Slow]))
        .c(Spawn::new(Category::Mob).commonness(100))
        ;

    // Colonist enemies

    Prototype::new(Some(colonist))
        .c(Desc::new("colonist", 34, DARKORANGE))
        .c(Stats::new(6, &[Hands]))
        .c(Spawn::new(Category::Mob).biome(Base))
        .c(Colonist::new())
        ;

    // TODO: Ranged attack
    Prototype::new(Some(colonist))
        .c(Desc::new("marine", 36, DARKOLIVEGREEN))
        .c(Stats::new(8, &[Hands]))
        .c(Spawn::new(Category::Mob).biome(Base).commonness(400))
        .c(Colonist::new())
        ;

    // TODO: Ranged attack
    Prototype::new(Some(colonist))
        .c(Desc::new("cyber controller", 42, LIGHTSLATEGRAY))
        .c(Stats::new(12, &[Slow, Hands, Robotic]))
        .c(Colonist::new())
        .c(Spawn::new(Category::Mob).biome(Base).commonness(40))
        ;

    // Dogs count as colonists because of terran DNA
    Prototype::new(Some(colonist))
        .c(Desc::new("dog", 44, OLIVE))
        .c(Stats::new(4, &[]))
        .c(Spawn::new(Category::Mob).biome(Base))
        .c(Colonist::new())
        ;

    // Robots don't count as colonists, being completely inorganic
    Prototype::new(Some(colonist))
        .c(Desc::new("robot", 62, SILVER))
        .c(Stats::new(6, &[Hands, Robotic, Slow]))
        .c(Spawn::new(Category::Mob).biome(Base).commonness(200))
        ;
}
