use minecraft_game_logic::block_state_registry::BlockRegistry;

pub fn get_block_registry() -> BlockRegistry {
    let (result, _) = bincode::serde::decode_from_slice(
        include_bytes!("../../data/blocks.bin"),
        bincode::config::standard(),
    )
        .expect("Failed to decode data/blocks.bin");

    result
}

#[cfg(test)]
mod tests {
    use minecraft_protocol::data::blocks::Block;
    use crate::ids::block_states::BlockWithStateId;
    use crate::ids::blocks::BlockId;
    use crate::registries::block_state_registry::get_block_registry;

    #[test]
    fn test_get_block() {
        let registry = get_block_registry();

        let states = registry.block_to_block_states(Block::from(BlockId::OakSlab));

        for state_id in states.clone() {
            let element = BlockWithStateId::from_id(state_id).expect("invalid state id");

            match element {
                BlockWithStateId::OakSlab { .. } => { /* good */ }
                _ => panic!("state {} maps to {:?}", state_id, element)
            }
        }

        // (top, bottom and double slab) * (waterlogged or not)
        assert_eq!(states.count(), 3 * 2);
    }
}
