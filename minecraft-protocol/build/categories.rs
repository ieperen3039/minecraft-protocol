
pub fn is_stairs(internal_name: &str) -> bool {
    internal_name.ends_with("_stairs")
}

pub fn is_trapdoor(internal_name: &str) -> bool {
    internal_name.ends_with("_trapdoor")
}

pub fn is_slab(internal_name: &str) -> bool {
    internal_name.ends_with("_slab")
}

pub fn is_wall(internal_name: &str) -> bool {
    internal_name.ends_with("_wall")
}

pub fn is_fence(internal_name: &str) -> bool {
    internal_name.ends_with("_fence") || internal_name.ends_with("_fence_gate")
}

pub fn is_portal(internal_name: &str) -> bool {
    internal_name.ends_with("_portal")
}

pub fn has_axis(internal_name: &str) -> bool {
    internal_name.ends_with("_log") || internal_name.ends_with("_basalt")
}