use super::*;

#[inheritable]
#[inherit(Entity)]
pub struct AbstractArrow {
    pub entity: Entity,
    pub is_critical: bool,
    pub is_no_clip: bool,
    pub piercing_level: i32,
}

impl Default for AbstractArrow {
    fn default() -> Self {
        Self {
            entity: Entity::default(),
            is_critical: false,
            is_no_clip: false,
            piercing_level: 0,
        }
    }
}

#[inherit(AbstractArrow, Entity)]
pub struct Arrow {
    pub abstract_arrow: AbstractArrow,
    pub color: isize,
}

impl Default for Arrow {
    fn default() -> Self {
        Self {
            abstract_arrow: AbstractArrow::default(),
            color: -1,
        }
    }
}

#[inherit(AbstractArrow, Entity)]
pub struct SpectralArrow {
    pub abstract_arrow: AbstractArrow,
    pub loyalty_level: i32,
    pub has_enchantment_glint: bool,
}

impl Default for SpectralArrow {
    fn default() -> Self {
        Self {
            abstract_arrow: AbstractArrow::default(),
            loyalty_level: 0,
            has_enchantment_glint: false,
        }
    }
}
