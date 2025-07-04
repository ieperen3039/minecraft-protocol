use crate::json::*;
use std::io::{ErrorKind, Read, Write};
use std::{collections::HashMap, fs::File};

pub const VERSION: &str = "1.20.1";

pub fn get_data(url: &str, cache: &str) -> serde_json::Value {
    match File::open(cache) {
        // The cache file is ready
        Ok(mut file) => {
            let mut data = Vec::new();
            if let Err(e) = file.read_to_end(&mut data) {
                panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file cannot be read. Error: {}", cache, e)
            }

            let json_text = match String::from_utf8(data) {
                Ok(json_text) => json_text,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file appears to contain invalid text data. Error: {}\nNote: Deleting the file will allow the library to download it again.", cache, e),
            };

            match serde_json::from_str(&json_text) {
                Ok(json) => json,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file appears to contain invalid json data. Error: {}\nNote: Deleting the file will allow the library to download it again.", cache, e),
            }
        }
        // The cache file needs to be downloaded
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let response = match minreq::get(url).send() {
                Ok(response) => response,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded from `{}`. Unfortunately, we can't access this URL. Error: {}", url, e)
            };

            let json_text = match response.as_str() {
                Ok(json_text) => json_text,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded from `{}`. Unfortunately, this file appears to contain invalid data. Error: {}", url, e),
            };

            let mut file = match File::create(cache) {
                Ok(file) => file,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, we can't access this path. Error: {}", cache, e),
            };

            if let Err(e) = file.write_all(json_text.as_bytes()) {
                panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, we can't write to this path. Error: {}", cache, e)
            };

            match serde_json::from_str(json_text) {
                Ok(json) => json,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file appears to contain invalid json data. Error: {}\nNote: Deleting the file will allow the library to download it again.", cache, e),
            }
        }

        // The cache file cannot be accessed
        Err(e) => {
            panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, we can't access this path. Error: {}", cache, e);
        }
    }
}

pub fn get_entities(target: &String, file_locations: &HashMap<String, String>) -> Vec<Entity> {
    let entities_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/entities.json",
        file_locations.get("entities").unwrap()
    );
    let entities_data = get_data(
        &entities_url,
        &format!("{target}/cache-entities-{}.json", VERSION),
    );
    let mut entities: Vec<Entity> =
        serde_json::from_value(entities_data).expect("Invalid entity data");
    entities.sort_by_key(|entity| entity.id);
    entities
}

pub fn get_blocks(target: &String, file_locations: &HashMap<String, String>) -> Vec<Block> {
    let blocks_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/blocks.json",
        file_locations.get("blocks").unwrap()
    );
    let block_data = get_data(
        &blocks_url,
        &format!("{target}/cache-blocks-{}.json", VERSION),
    );
    let mut blocks: Vec<Block> =
        serde_json::from_value(block_data).expect("Invalid block data");
    blocks.sort_by_key(|block| block.id);

    // Look for missing blocks in the array
    let mut expected = 0;
    for block in &mut blocks {
        if block.id != expected {
            panic!("The block with id {} is missing.", expected)
        }
        expected += 1;

        // reduce conflicts in the state categories
        for state in &mut block.states {
            if let Some(vals) = &state.values {
                if state.name == "facing" && vals.len() == 4 {
                    if vals
                        .iter()
                        .all(|v| v != "up" && v != "down")
                    {
                        state.name.push_str("_horizontal");
                    }
                }

                // hoppers are the only "facing" user without "up"
                if state.name == "facing" && vals.len() == 5 {
                    if vals
                        .iter()
                        .all(|v| v != "up")
                    {
                        state.name.push_str("_hopper");
                    }
                }

                // nether portals are the only "axis" user without a y coordinate
                if state.name == "axis" {
                    if vals.iter().all(|v| v != "y") {
                        state.name.push_str("_horizontal");
                    }
                }

                // all plants have a "half" state
                if state.name == "half" && vals.len() == 2 && vals[0] == "upper" && vals[1] == "lower" {
                    state.name = String::from("vertical_half");
                }
            }
        }
    }

    blocks
}

pub fn get_items(target: &String, file_locations: &HashMap<String, String>) -> Vec<Item> {
    let items_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/items.json",
        file_locations.get("items").unwrap()
    );
    let items_data = get_data(
        &items_url,
        &format!("{target}/cache-items-{}.json", VERSION),
    );

    let mut items: Vec<Item> = serde_json::from_value(items_data).expect("Invalid item data");
    items.sort_by_key(|item| item.id);

    // Patch the missing Air
    // TODO check if this is necessary
    if items.first().map(|i| i.id) != Some(0) {
        items.insert(
            0,
            Item {
                id: 0,
                display_name: String::from("Air"),
                internal_name: String::from("air"),
                stack_size: 64,
                max_durability: None,
            },
        );
    }

    items
}
pub fn get_recipes(target: &String, file_locations: &HashMap<String, String>) -> HashMap<u32, Vec<Recipe>> {
    let recipes_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/recipes.json",
        file_locations.get("recipes").unwrap()
    );
    let recipes_data = get_data(
        &recipes_url,
        &format!("{target}/cache-recipes-{}.json", VERSION),
    );
    let mut item_recipes: HashMap<u32, Vec<Recipe>> =
        serde_json::from_value(recipes_data).expect("Invalid recipes");
    // Count recipes
    for recipes in item_recipes.values_mut() {
        let old_len = recipes.len();
        recipes.retain(|recipe| !matches!(recipe, Recipe::DoubleShaped { .. }));
        if recipes.len() != old_len {
            println!("Contains a double shaped recipe, which support has been removed as an optimization. It needs to be enabled again if required by future minecraft updates.");
            println!("cargo::error=Recipes contains a double shaped recipe")
        }
    }
    item_recipes
}

pub fn get_block_drops(target: &String, file_locations: &HashMap<String, String>) -> Vec<BlockDropMapping> {
    let block_drops_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/blockLoot.json",
        file_locations.get("blockLoot").unwrap()
    );
    let block_drops_data = get_data(
        &block_drops_url,
        &format!("{target}/cache-block-loot-{}.json", VERSION),
    );

    serde_json::from_value(block_drops_data).expect("Invalid block loot data")
}

pub fn get_file_locations(target: &String) -> HashMap<String, String> {
    let mut file_locations = get_data(
        "https://raw.githubusercontent.com/PrismarineJS/minecraft-data/master/data/dataPaths.json",
        &format!("{target}/cache-file-location-{}.json", VERSION),
    );
    let file_locations = file_locations.get_mut("pc").unwrap().take();
    let mut file_locations: HashMap<String, HashMap<String, String>> =
        serde_json::from_value(file_locations).unwrap();

    file_locations
        .remove(VERSION)
        .expect("There is no generated data for this minecraft version yet")
}
