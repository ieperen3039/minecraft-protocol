
enum DropTable {
	Single(ItemDropLogic),
	Compound(Vec<ItemDropLogic>),
}

enum ItemDropLogic {
	Single(ItemId),
	FixedMultiple(ItemId, usize),
	RandomMultiple{
		item: ItemId,
		min: usize,
		max: usize,
	},
}

struct BlockDrops {
	block_tools: HashMap<BlockId, Vec<ItemId>>,
	tool_drops: HashMap<BlockId, DropTable>,
	hand_drops: HashMap<BlockId, DropTable>,
	silk_touch_drops: HashMap<BlockId, ItemDropLogic>,
	
	gravel_block: BlockId,
	gravel_item: ItemId,
	flint_item: ItemId,
}

impl BlockDrops {
	fn get_drops(&self, block: BlockId, held_item: Item, silk_touch: bool, fortune: u32, rng: &Rng, drops_out: &mut Vec<ItemId>) {
		if silk_touch {
			let drop = silk_touch_drops.get(block).unwrap_or(0);
			Self::parse_logic2(drop, rng, drops_out); 
			return;
		}	
		
		if let Some(tools) = self.block_tools.get(block) {
			if tools.contains(held_item) {
				if let Some(tool_drop) = self.tool_drops.get(block) {
					Self::parse_logic(tool_drop, rng, drops_out);
					return;
				}
			}
		}
		
		if let Some(hand_drop) = self.hand_drops.get(block) {
			Self::parse_logic(hand_drop, rng, drops_out);
			return;
		}
		
		if block == self.gravel_block {
			if rng
				drops_out.push(self.gravel_item);
			else
				drops_out.push(self.flint_item);
		}
		
		// else drop nothing
	}

	fn parse_logic(drop_logic: DropLogic, rng: &Rng, drops_out: &mut Vec<ItemId>) {
		match drop_logic {
			Single(item) => Self::parse_logic2(elt, rng, drops_out)
			Compound(multiple) => {
				for (elt : multiple) {
					Self::parse_logic2(elt, rng, drops_out);
				}
			},
		}
	}

	fn parse_logic2(drop_logic: DropLogic, rng: &Rng, drops_out: &mut Vec<ItemId>) {
		match drop_logic {
			Single(item) => drops_out.push(item),
			FixedMultiple(item, quantity) => TODO,
			RandomMultiple{item, min, max} => TODO,
		}
	}

}
