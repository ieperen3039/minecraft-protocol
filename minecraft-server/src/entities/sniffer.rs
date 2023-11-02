use super::*;

#[derive(Default)]
#[inherit(Animal, AgeableMob, PathfinderMob, Mob, LivingEntity, Entity)]
pub struct Sniffer {
    pub sniffer_state: u8,
    pub drop_seed_at_tick: usize,
}
