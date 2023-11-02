use std::{net::SocketAddr, future::Future, collections::{HashMap, BTreeMap}};

use futures::FutureExt;
use minecraft_protocol::{MinecraftPacketPart, components::{gamemode::{Gamemode, PreviousGamemode}, difficulty::Difficulty, chunk::{Chunk, PalettedData, ChunkData}, entity::{EntityMetadata, EntityMetadataValue, EntityAttribute}, slots::Slot, chat::ChatMode, players::MainHand}, nbt::NbtTag};
use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};

use crate::prelude::*;

pub type Task = Pin<Box<dyn Future<Output = Result<(), ()>> + Send + Sync + 'static>>;

pub async fn receive_packet(stream: &mut TcpStream) -> Vec<u8> {
    let mut length: Vec<u8> = Vec::with_capacity(2);

    loop {
        if length.len() >= 5 {
            //return Err("length too long".into());
        }
        let mut byte = [0];
        stream.read_exact(&mut byte).await.unwrap();
        length.push(byte[0]);
        if byte[0] < 0b1000_0000 {
            break;
        }
    }

    let length = VarInt::deserialize_uncompressed_minecraft_packet(length.as_mut_slice()).unwrap();

    let mut data = Vec::with_capacity(length.0 as usize);
    unsafe { data.set_len(length.0 as usize); }
    stream.read_exact(&mut data).await.unwrap();

    data
}

pub async fn send_packet_raw(stream: &mut TcpStream, packet: &[u8]) {
    let length = VarInt::from(packet.len());
    stream.write_all(length.serialize_minecraft_packet().unwrap().as_slice()).await.unwrap();
    stream.write_all(packet).await.unwrap();
    stream.flush().await.unwrap();
}

pub async fn send_packet<'a, P: MinecraftPacketPart<'a>>(stream: &mut TcpStream, packet: P) {
    let packet = packet.serialize_minecraft_packet().unwrap();
    send_packet_raw(stream, packet.as_slice()).await;
}

pub async fn status(stream: &mut TcpStream) {
    loop {
        let packet = receive_packet(stream).await;
        match StatusServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap() {
            StatusServerbound::Request => {
                let response = StatusClientbound::Response {
                    json_response: include_str!("raw/status_response.json")
                };
                send_packet(stream, response).await;    
                debug!("StatusResponse sent");                
            },
            StatusServerbound::Ping { payload } => {
                warn!("Ping received");
                let pong = StatusClientbound::Pong {
                    payload
                };
                send_packet(stream, pong).await;
                debug!("Pong sent");
                return;
            },
            _ => {
                debug!("Unexpected packet: {packet:?}");
            }
        };    
    }
}

pub struct LoggedInPlayerInfo {
    addr: SocketAddr,
    username: String,
    uuid: u128,
}

pub async fn login(stream: &mut TcpStream, addr: SocketAddr) -> Result<LoggedInPlayerInfo, ()> {
    // Receive login start
    let packet = receive_packet(stream).await;
    let packet = LoginServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap();
    let LoginServerbound::LoginStart{ username, player_uuid } = packet else {
        error!("Expected LoginStart packet, got: {packet:?}");
        return Err(());
    };
    debug!("LoginStart: {username}");

    // TODO encryption

    // TODO compression

    // Send login success
    let login_success = LoginClientbound::LoginSuccess {
        uuid: player_uuid,
        username,
        properties: Array::default(),
    };
    send_packet(stream, login_success).await;
    debug!("LoginSuccess sent");

    // Receive login acknowledged
    let packet = receive_packet(stream).await;
    let packet = LoginServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap();
    let LoginServerbound::LoginAcknowledged = packet else {
        error!("Expected LoginAcknowledged packet, got: {packet:?}");
        return Err(());
    };
    debug!("LoginAcknowledged received");

    // Ignore encryption response if any
    let packet = receive_packet(stream).await;
    if let Ok(LoginServerbound::EncryptionResponse { .. }) = LoginServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()) {
        // Ignore for now (TODO)
        //packet = receive_packet(stream).await;
    }
    debug!("EncryptionResponse ignored");

    Ok(LoggedInPlayerInfo {
        addr,
        username: username.to_owned(),
        uuid: player_uuid,
    })
}

pub struct PlayerInfo {
    addr: SocketAddr,
    username: String,
    uuid: u128,
    locale: String,
    render_distance: usize,
    chat_mode: ChatMode,
    chat_colors: bool,
    displayed_skin_parts: u8,
    main_hand: MainHand,
    enable_text_filtering: bool,
    allow_server_listing: bool,
}

pub async fn handshake(mut stream: &mut TcpStream, logged_in_player_info: LoggedInPlayerInfo) -> Result<PlayerInfo, ()> {
        // Receive client informations
    let packet = receive_packet(&mut stream).await;
    debug!("Packet received");
    let packet = ConfigServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap();
    let ConfigServerbound::ClientInformations { locale, render_distance, chat_mode, chat_colors, displayed_skin_parts, main_hand, enable_text_filtering, allow_server_listing } = packet else {
        error!("Expected ClientInformation packet, got: {packet:?}");
        return Err(());
    };
    debug!("ClientInformation received");

    // Send server agent
    let server_agent = ConfigClientbound::PluginMessage {
        channel: "minecraft:brand",
        data: RawBytes {
            data: &[6, 83, 112, 105, 103, 111, 116]
        },
    };
    send_packet(&mut stream, server_agent).await;
    debug!("PluginMessage sent");

    // Send feature flags
    let feature_flags = ConfigClientbound::FeatureFlags {
        features: Array::from(vec!["minecraft:vanilla"]),
    };
    send_packet(&mut stream, feature_flags).await;
    debug!("FeatureFlags sent");

    // Send registry data
    send_packet_raw(&mut stream, include_bytes!("raw/registry_codec.mc_packet")).await;
    debug!("RegistryData sent");

    // Update tags
    let update_tags = ConfigClientbound::UpdateTags {
        tags: Map::default(),
    };
    send_packet(&mut stream, update_tags).await;
    debug!("UpdateTags sent");

    // Send finish configuration
    let finish_configuration = ConfigClientbound::FinishConfiguration;
    send_packet(&mut stream, finish_configuration).await;
    debug!("FinishConfiguration sent");

    // Receive finish configuration
    let packet = receive_packet(&mut stream).await;
    let packet = ConfigServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap();
    let ConfigServerbound::FinishConfiguration = packet else {
        error!("Expected FinishConfiguration packet, got: {packet:?}");
        return Err(());
    };
    debug!("FinishConfiguration received");

    // Send join game
    let player_id: usize = 3429; // TODO dynamic attribution
    let join_game = PlayClientbound::JoinGame {
        player_id: player_id as i32,
        is_hardcore: false,
        dimensions_names: Array::from(vec!["minecraft:overworld"]),
        max_players: VarInt::from(1000),
        render_distance: VarInt::from(12),
        simulation_distance: VarInt::from(8),
        reduced_debug_info: false,
        enable_respawn_screen: true,
        do_limited_crafting: false,
        dimension_type: "minecraft:overworld",
        dimension_name: "minecraft:overworld",
        hashed_seed: 42,
        gamemode: Gamemode::Creative,
        previous_gamemode: PreviousGamemode::Creative,
        is_debug: false,
        is_flat: true,
        death_location: None,
        portal_cooldown: VarInt::from(0),
    };
    send_packet(&mut stream, join_game).await;
    debug!("JoinGame sent");

    // Set difficulty
    let change_difficulty = PlayClientbound::ChangeDifficulty {
        difficulty: Difficulty::Normal,
        difficulty_locked: false
    };
    send_packet(&mut stream, change_difficulty).await;
    debug!("ChangeDifficulty sent");

    // Set player abilities
    let change_player_abilities = PlayClientbound::PlayerAbilities {
        flags: 0,
        flying_speed: 0.05,
        field_of_view_modifier: 0.1
    };
    send_packet(&mut stream, change_player_abilities).await;
    debug!("PlayerAbilities sent");

    // Set held item
    let held_item_change = PlayClientbound::SetHeldItem {
        slot: 0 // TODO should be the same as when disconnected
    };
    send_packet(&mut stream, held_item_change).await;
    debug!("SetHeldItem sent");

    // Update recipes
    let update_recipes = PlayClientbound::UpdateRecipes {
        data: RawBytes {
            data: &[0]
        }
    };
    send_packet(&mut stream, update_recipes).await;
    debug!("UpdateRecipes sent");

    // Entity event
    let entity_event = PlayClientbound::EntityEvent {
        entity_id: player_id as i32,
        entity_status: 28
    };
    send_packet(&mut stream, entity_event).await;
    debug!("EntityEvent sent");

    // Declare commands
    let declare_commands = PlayClientbound::DeclareCommands {
        count: VarInt(0),
        data: RawBytes {
            data: &[0]
        }
    };
    send_packet(&mut stream, declare_commands).await;
    debug!("DeclareCommands sent");

    // Unlock recipes
    let unlock_recipes = PlayClientbound::UnlockRecipes {
        action: minecraft_protocol::components::recipes::UnlockRecipesAction::Init {
            crafting_recipe_book_open: false,
            crafting_recipe_book_filter_active: false,
            smelting_recipe_book_open: false,
            smelting_recipe_book_filter_active: false,
            blast_furnace_recipe_book_open: false,
            blast_furnace_recipe_book_filter_active: false,
            smoker_recipe_book_open: false,
            smoker_recipe_book_filter_active: false,
            displayed_recipes: Array::default(),
            added_recipes: Array::default()
        }
    };
    send_packet(&mut stream, unlock_recipes).await;
    debug!("UnlockRecipes sent");

    // Spawn player
    let player_position = PlayClientbound::PlayerPositionAndLook {
        x: 0.0,
        y: 60.0,
        z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        flags: 0,
        teleport_id: VarInt(1),
    };
    send_packet(&mut stream, player_position).await;
    debug!("PlayerPositionAndLook sent");

    // Send server metadata
    let server_data = PlayClientbound::ServerData {
        motd: "{\"text\":\"A Minecraft Server\"}",
        icon: None,
        enforces_secure_chat: false,
    };
    send_packet(&mut stream, server_data).await;
    debug!("ServerData sent");

    // Spawn message
    let spawn_message = PlayClientbound::SystemChatMessage {
        content: "{\"text\":\"Hello world\"}",
        overlay: false,
    };
    send_packet(&mut stream, spawn_message).await;
    debug!("SystemChatMessage sent");

    // TODO: update players info (x2)

    // Set entity metadata
    let mut entity_metadata = BTreeMap::new();
    entity_metadata.insert(9, EntityMetadataValue::Float { value: 20.0 });
    entity_metadata.insert(16, EntityMetadataValue::VarInt { value: VarInt(18) });
    entity_metadata.insert(17, EntityMetadataValue::Byte { value: 127 });
    let set_entity_metadata = PlayClientbound::SetEntityMetadata {
        entity_id: VarInt::from(player_id),
        metadata: EntityMetadata { items: entity_metadata.clone() }
    };
    send_packet(&mut stream, set_entity_metadata).await;
    debug!("SetEntityMetadata sent");

    // Initialize world border
    let world_border_init = PlayClientbound::InitializeWorldBorder {
        x: 0.0,
        y: 0.0,
        old_diameter: 60000000.0,
        new_diameter: 60000000.0,
        speed: VarLong(0),
        portal_teleport_boundary: VarInt(29999984),
        warning_blocks: VarInt(5),
        warning_time: VarInt(15),
    };
    send_packet(&mut stream, world_border_init).await;
    debug!("InitializeWorldBorder sent");

    // Update time
    let time_update = PlayClientbound::UpdateTime {
        world_age: 0,
        time_of_day: 0,
    };
    send_packet(&mut stream, time_update).await;
    debug!("UpdateTime sent");

    // Set spawn position
    let set_spawn_position = PlayClientbound::SetSpawnPosition {
        location: minecraft_protocol::packets::Position { x: 0, y: 70, z: 0 },
        angle: 0.0,
    };
    send_packet(&mut stream, set_spawn_position).await;
    debug!("SetSpawnPosition sent");

    // Set center chunk
    let set_center_chunk = PlayClientbound::SetCenterChunk {
        chunk_x: VarInt(0), // TODO: should be the same as when disconnected
        chunk_z: VarInt(0), // TODO: should be the same as when disconnected
    };
    send_packet(&mut stream, set_center_chunk).await;
    debug!("SetCenterChunk sent");

    // Set inventory
    let set_container_content = PlayClientbound::SetContainerContent {
        window_id: 0,
        state_id: VarInt(1),
        slots: Array::default(),
        carried_item: Slot { item: None }
    };
    send_packet(&mut stream, set_container_content).await;
    debug!("SetContainerContent sent");

    // Set entity metadata (again)
    let set_entity_metadata = PlayClientbound::SetEntityMetadata {
        entity_id: VarInt::from(player_id),
        metadata: EntityMetadata { items: entity_metadata }
    };
    send_packet(&mut stream, set_entity_metadata).await;
    debug!("SetEntityMetadata sent");

    // Update entity attributes
    let mut entity_attributes = BTreeMap::new();
    entity_attributes.insert("minecraft:generic.attack_speed", EntityAttribute { value: 4.0, modifiers: Array::default() });
    entity_attributes.insert("minecraft:generic.max_health", EntityAttribute { value: 20.0, modifiers: Array::default() });
    entity_attributes.insert("minecraft:generic.movement_speed", EntityAttribute { value: 0.10000000149011612, modifiers: Array::default() });
    let update_entity_attributes = PlayClientbound::UpdateEntityAttributes {
        entity_id: VarInt::from(player_id),
        attributes: Map::from(entity_attributes)
    };
    send_packet(&mut stream, update_entity_attributes).await;
    debug!("UpdateEntityAttributes sent");

    // Update advancements
    let update_advancements = PlayClientbound::UpdateAdvancements {
        reset: true,
        advancement_mapping: Map::default(),
        advancements_to_remove: Array::default(),
        progress_mapping: Map::default(),
    };
    send_packet(&mut stream, update_advancements).await;
    debug!("UpdateAdvancements sent");

    // Set health
    let set_health = PlayClientbound::SetHealth {
        health: 20.0,
        food: VarInt(20),
        food_saturation: 5.0,
    };
    send_packet(&mut stream, set_health).await;
    debug!("UpdateHealth sent");

    // Set experience
    let set_experience = PlayClientbound::SetExperience {
        experience_level: VarInt(0),
        experience_bar: 0.0,
        total_experience: VarInt(0),
    };
    send_packet(&mut stream, set_experience).await;
    debug!("SetExperience sent");

    // Chunk batch start
    let chunk_data = PlayClientbound::ChunkBatchStart;
    send_packet(&mut stream, chunk_data).await;
    debug!("ChunkBatchStart sent");

    let empty_chunk = Chunk {
        block_count: 0,
        blocks: PalettedData::Single { value: 0 },
        biomes: PalettedData::Single { value: 4 },
    };
    let dirt_chunk = Chunk {
        block_count: 4096,
        blocks: PalettedData::Single { value: minecraft_protocol::ids::blocks::Block::GrassBlock.default_state_id() },
        biomes: PalettedData::Single { value: 4 },
    };
    let mut flat_column = Vec::new();
    flat_column.push(dirt_chunk);
    for _ in 0..23 {
        flat_column.push(empty_chunk.clone());
    }
    let serialized: Vec<u8> = Chunk::into_data(flat_column).unwrap();
    let mut heightmaps = HashMap::new();
    heightmaps.insert(String::from("MOTION_BLOCKING"), NbtTag::LongArray(vec![0; 37]));
    let heightmaps = NbtTag::Compound(heightmaps);
    
    for cx in -3..=3 {
        for cz in -3..=3 {
            let chunk_data = PlayClientbound::ChunkData {
                value: ChunkData {
                    chunk_x: cx,
                    chunk_z: cz,
                    heightmaps: heightmaps.clone(),
                    data: Array::from(serialized.clone()),
                    block_entities: Array::default(),
                    sky_light_mask: Array::default(),
                    block_light_mask: Array::default(),
                    empty_sky_light_mask: Array::default(),
                    empty_block_light_mask: Array::default(),
                    sky_light: Array::default(),
                    block_light: Array::default(),
                }
            };
            send_packet(&mut stream, chunk_data).await;
        }
    }
    debug!("ChunkData sent");

    // Chunk batch end
    let chunk_data = PlayClientbound::ChunkBatchFinished { batch_size: VarInt(49) };
    send_packet(&mut stream, chunk_data).await;
    debug!("ChunkBatchFinished sent");

    // Get chunk batch acknoledgement
    let packet = receive_packet(&mut stream).await;
    let packet = PlayServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap();
    let PlayServerbound::ChunkBatchReceived { chunks_per_tick } = packet else {
        error!("Expected ChunkBatchAcknoledgement packet, got: {packet:?}");
        return Err(());
    };
    debug!("ChunkBatchAcknoledgement received");

    Ok(PlayerInfo {
        addr: logged_in_player_info.addr,
        username: logged_in_player_info.username,
        uuid: logged_in_player_info.uuid,
        locale: locale.to_owned(),
        render_distance: render_distance.try_into().unwrap_or(5),
        chat_mode,
        chat_colors,
        displayed_skin_parts,
        main_hand,
        enable_text_filtering,
        allow_server_listing,
    })
}

struct PlayerHandler {
    info: PlayerInfo,
    position: Position,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}

impl PlayerHandler {
    async fn on_server_message(&mut self, message: ServerMessage) {
        use ServerMessage::*;
        match message {
            Tick => {
                
            }
        }
    }

    async fn on_packet<'a>(&mut self, packet: PlayServerbound<'a>) {
        use PlayServerbound::*;
        match packet {
            SetPlayerPosition { x, y, z, on_ground } => {
                self.position.x = x;
                self.position.y = y;
                self.position.z = z;
                self.on_ground = on_ground;
                // TODO: make sure the movement is allowed
            },
            SetPlayerRotation { yaw, pitch, on_ground } => {
                self.yaw = yaw;
                self.pitch = pitch;
                self.on_ground = on_ground;
            }
            SetPlayerPositionAndRotation { x, y, z, yaw, pitch, on_ground } => {
                self.position.x = x;
                self.position.y = y;
                self.position.z = z;
                self.yaw = yaw;
                self.pitch = pitch;
                self.on_ground = on_ground;
                // TODO: make sure the movement is allowed
            },
            packet => warn!("Unsupported packet received: {packet:?}"),
        }
    }
}

pub async fn handle_player(mut stream: TcpStream, player_info: PlayerInfo, mut server_msg_rcvr: BroadcastReceiver<ServerMessage>) -> Result<(), ()> {
    let mut handler = PlayerHandler {
        info: player_info,
        position: Position { x: 0.0, y: 60.0, z: 0.0 },
        yaw: 0.0,
        pitch: 0.0,
        on_ground: false,
    };
    
    let mut receive_packet_fut = Box::pin(receive_packet(&mut stream).fuse());
    let mut receive_server_message_fut = Box::pin(server_msg_rcvr.recv().fuse());
    loop {
        // Select the first event that happens
        enum Event {
            Packet(Vec<u8>),
            Message(Result<ServerMessage, BroadcastRecvError>),
        }
        let event = futures::select! {
            packet = receive_packet_fut => Event::Packet(packet),
            message = receive_server_message_fut => Event::Message(message),
        };
        match event {
            Event::Packet(packet) => {
                drop(receive_packet_fut);
                receive_packet_fut = Box::pin(receive_packet(&mut stream).fuse());

                let packet = PlayServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap();
                handler.on_packet(packet).await;
            },
            Event::Message(Ok(message)) => {
                drop(receive_server_message_fut);
                receive_server_message_fut = Box::pin(server_msg_rcvr.recv().fuse());

                handler.on_server_message(message).await;
            },
            Event::Message(Err(recv_error)) => {
                error!("Failed to receive message: {recv_error:?}");
                return Err(());
            }
        }
    }
}

pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    server_msg_rcvr: BroadcastReceiver<ServerMessage>,
) -> Result<(), ()> {
    // Receive handshake
    let packet = receive_packet(&mut stream).await;
    let HandshakeServerbound::Hello { protocol_version, server_address, server_port, next_state } = HandshakeServerbound::deserialize_uncompressed_minecraft_packet(packet.as_slice()).unwrap();
    match next_state {
        ConnectionState::Login => {
            let player_info = login(&mut stream, addr).await?;
            let player_info = handshake(&mut stream, player_info).await?;
            handle_player(stream, player_info, server_msg_rcvr).await
        },
        ConnectionState::Status => {
            status(&mut stream).await;
            Ok(())
        },
        _ => {
            error!("Unexpected next state: {next_state:?}");
            Err(())
        }
    }
}
