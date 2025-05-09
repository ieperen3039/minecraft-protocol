use crate::ids::block_states::*;

// this assumes normalized coordinates
#[inline]
pub fn to_facing(x: f32, y: f32, z: f32) -> Facing {
    // positive x is east, positive z is south, positive y is up
    if x > 0.5 {
        Facing::East
    } else if x < -0.5 {
        Facing::West
    } else if z > 0.5 {
        Facing::South
    } else if z < -0.5 {
        Facing::North
    } else if y > 0.5 {
        Facing::Up
    } else {
        Facing::Down
    }
}

// this assumes normalized coordinates
pub fn to_hopper_facing(x: f32, y: f32, z: f32) -> FacingHopper {
    match to_facing(x, y, z) {
        Facing::North => FacingHopper::North,
        Facing::South => FacingHopper::South,
        Facing::West => FacingHopper::West,
        Facing::East => FacingHopper::East,
        Facing::Down | Facing::Up => FacingHopper::Down,
    }
}

// this assumes normalized coordinates
pub fn to_facing_horizontal(x: f32, z: f32) -> FacingHorizontal {
    match to_facing(x, 0.0, z) {
        Facing::North => FacingHorizontal::North,
        Facing::South => FacingHorizontal::South,
        Facing::West => FacingHorizontal::West,
        Facing::East => FacingHorizontal::East,
        _ => unreachable!(),
    }
}

pub fn to_axis(x: f32, y: f32, z: f32) -> Axis {
    match to_facing(x, y, z) {
        Facing::North | Facing::South => Axis::X,
        Facing::Down | Facing::Up => Axis::Y,
        Facing::West | Facing::East => Axis::Z,
    }
}

pub fn to_axis_horizontal(x: f32, z: f32) -> AxisHorizontal {
    match to_facing_horizontal(x, z) {
        FacingHorizontal::North | FacingHorizontal::South => AxisHorizontal::X,
        FacingHorizontal::West | FacingHorizontal::East => AxisHorizontal::Z,
    }
}

// rotation:
// 0	The block is FacingHopper south.
// 4	The block is FacingHopper west.
// 8	The block is FacingHopper north.
// 12	The block is FacingHopper east.
// 15	The block is FacingHopper south-southeast.
pub fn to_sign_rotation(x: f32, z: f32) -> u8 {
    const SECTION_ANGLE: f32 = f32::consts::PI / 8;
    const HALF_SECTION_ANGLE: f32 = SECTION_ANGLE / 2;

    let angle_rad = f32::atan2(x, z);
    let normalized = (angle_rad + HALF_SECTION_ANGLE) / SECTION_ANGLE;
    normalized as u8 % 16
}

pub fn is_waterlogged(placed_on: BlockWithState) -> bool {
    match placed_on {
        BlockWithState::Water { level } => level > 14,
        _ => false,
    }
}

pub fn to_floor_wall_ceiling(cursor_position_vertical: f32) -> Face {
    if cursor_position_vertical < f32::EPSILON {
        Face::Floor
    } else if cursor_position_vertical > (1.0 - f32::EPSILON) {
        Face::Ceiling
    } else {
        Face::Wall
    }
}

#[allow(dead_code)]
pub fn to_rotation_from_normalized(x: f32, z: f32) -> u8 {
    const SECTION_ANGLE: f32 = f32::consts::PI / 8;
    const HALF_SECTION_ANGLE: f32 = SECTION_ANGLE / 2;
    const C8: f32 = HALF_SECTION_ANGLE.cos(); // 0.9808
    const C7: f32 = (HALF_SECTION_ANGLE + 1 * SECTION_ANGLE).cos(); // 0.8315
    const C6: f32 = (HALF_SECTION_ANGLE + 2 * SECTION_ANGLE).cos(); // 0.5556
    const C5: f32 = (HALF_SECTION_ANGLE + 3 * SECTION_ANGLE).cos(); // 0.1951
    const C4: f32 = -C5;
    const C3: f32 = -C6;
    const C2: f32 = -C7;
    const C1: f32 = -C8;

    // positive x is east, positive z is south
    match (x, z) {
        (C4..C5, C8..1.0) => 0,
        (C3..C4, C7..C8) => 1,
        (C2..C3, C6..C7) => 2,
        (C1..C2, C5..C6) => 3,
        (0.0..C1, C4..C5) => 4,
        (C1..C2, C3..C4) => 5,
        (C2..C3, C2..C3) => 6,
        (C3..C4, C1..C2) => 7,
        (C4..C5, 0.0..C1) => 8,
        (C5..C6, C1..C2) => 9,
        (C6..C7, C2..C3) => 10,
        (C7..C8, C3..C4) => 11,
        (C8..1.0, C4..C5) => 12,
        (C7..C8, C5..C6) => 13,
        (C6..C7, C6..C7) => 14,
        (C5..C6, C7..C8) => 15,
        _ => unreachable!("x and z must be normalized, was ({}, {})", x, z),
    }
}
