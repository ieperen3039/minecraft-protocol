use minecraft_game_logic::block_drop_registry::BlockDropRegistry;

pub fn get_block_drop_registry() -> BlockDropRegistry {
    let (result, _) = bincode::serde::decode_from_slice(
        include_bytes!("../../data/block_drops.bin"),
        bincode::config::standard(),
    )
    .expect("Failed to decode data/block_drops.bin");

    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_block_drop() {}
}
