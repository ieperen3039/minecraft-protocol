#[allow(unused_imports)]
use super::play_clientbound::ClientboundPacket;
use super::*;
use crate::components::*;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, MinecraftPacketPart)]
#[discriminant(VarInt)]
pub enum ServerboundPacket<'a> {
    /// *Response to [ClientboundPacket::SynchronizePlayerPosition]*
    ConfirmTeleportation {
        /// The ID given in [ClientboundPacket::SynchronizePlayerPosition::teleport_id]
        teleport_id: VarInt,
    },

    /// *Request for [ClientboundPacket::NbtQueryResponse]*
    QueryBlockNbt {
        /// An incremental ID so that the client can verify that the response matches
        transaction_id: VarInt,
        /// The location of the block to check
        position: Position,
    },

    /// Appears to only be used on singleplayer; the difficulty buttons are still disabled in multiplayer.
    ChangeDifficulty {
        new_difficulty: difficulty::Difficulty,
    },

    AcknowledgeMessage {
        message_id: VarInt,
    },

    ChatCommand {
        /// The command typed by the client.
        command: &'a str,
        /// The timestamp that the command was executed.
        timestamp: i64,
        /// The salt for the following argument signatures.
        salt: u64,
        /// Argument Signatures (not implemented)
        signatures: RawBytes<'a>,
    },

    /// Used to send a chat message to the server. The message may not be longer than 256 characters or else the server will kick the client.
    ///
    /// The server will broadcast the same chat message to all players on the server (including the player that sent the message), prepended with player's name. Specifically, it will respond with a translate chat component, "chat.type.text" with the first parameter set to the display name of the player (including some chat component logic to support clicking the name to send a PM) and the second parameter set to the message. See processing chat for more information.
    ///
    /// *See also [ClientboundPacket::ChatMessage]*
    ChatMessage {
        /// The message may not be longer than 256 characters or else the server will kick the client.
        message: Chat<'a>,
        timestamp: i64,
        /// The salt used to verify the signature hash.
        salt: u64,
        /// Signature (not implemented)
        signatures: RawBytes<'a>,
    },

    PlayerSession {
        session_id: UUID,
        /// The time the play session key expires in epoch milliseconds.
        expires_at: i64,
        /// A byte array of an X.509-encoded public key.
        /// Maximum length in Notchian server is 512 bytes.
        public_key: Array<'a, u8, VarInt>,
        /// The signature consists of the player UUID, the key expiration timestamp, and the public key data. These values are hashed using SHA-1 and signed using Mojang's private RSA key.
        /// Maximum length in Notchian server is 4096 bytes.
        key_signature: Array<'a, u8, VarInt>,
    },

    /// Notifies the server that the chunk batch has been received by the client. The server uses the value sent in this packet to adjust the number of chunks to be sent in a batch.
    ///
    /// The Notchian server will stop sending further chunk data until the client acknowledges the sent chunk batch. After the first acknowledgement, the server adjusts this number to allow up to 10 unacknowledged batches.
    ChunkBatchReceived {
        /// Desired chunks per tick.
        chunks_per_tick: f32,
    },

    /// *Request for [ClientboundPacket::Statistics]*
    ClientStatus { action: game_state::ClientStatus },

    /// Sent when the player connects, or when settings are changed
    ClientSettings {
        /// e.g. `en_GB`
        locale: &'a str,
        /// Client-side render distance, in chunks
        render_distance: u8,
        chat_mode: chat::ChatMode,
        /// “Colors” multiplayer setting
        chat_colors_enabled: bool,
        /// Bit mask, see [the wiki](https://wiki.vg/Protocol#Client_Settings)
        displayed_skin_parts: u8,
        main_hand: slots::MainHand,
        /// Enables filtering of text on signs and written book titles.
        /// Currently always false (i.e. the filtering is disabled)
        enable_text_filtering: bool,
        /// Servers usually list online players, this option should let you not show up in that list.
        allow_server_listings: bool,
    },

    /// Sent when the client needs to tab-complete a minecraft:ask_server suggestion type.
    /// 
    /// *Request for [ClientboundPacket::TabComplete]*
    CommandSuggestionsRequest {
        /// The id of the transaction that the server will send back to the client in the response of this packet.
        /// Client generates this and increments it each time it sends another tab completion that doesn't get a response.
        transaction_id: VarInt,
        /// All text behind the cursor without the `/` (e.g. to the left of the cursor in left-to-right languages like English).
        text: &'a str,
    },

    /// This packet switches the connection state to configuration.
    /// 
    /// *Response to [ClientboundPacket::StartConfiguration]*
    AcknowledgeConfiguration,

    /// Used when clicking on window buttons
    ClickWindowButton {
        /// The ID of the window sent by [ClientboundPacket::OpenWindow].
        window_id: i8,
        /// Meaning depends on window type; see [the wiki](https://wiki.vg/Protocol#Click_Window_Button)
        button_id: u8,
    },

    /// This packet is sent by the player when it clicks on a slot in a window.
    ///
    /// *Request for [ClientboundPacket::WindowConfirmation]*
    ClickWindowSlot {
        /// The ID of the window which was clicked. 0 for player inventory.
        window_id: i8,
        /// The last received State ID from either a [ClientboundPacket::SetSlot] or a [ClientboundPacket::WindowItems] packet
        state_id: VarInt,
        /// The clicked slot number, see [the wiki](https://wiki.vg/Protocol#Click_Window)
        slot: i16,
        /// The button used in the click, see [the wiki](https://wiki.vg/Protocol#Click_Window)
        button: i8,
        /// Inventory operation mode, see [the wiki](https://wiki.vg/Protocol#Click_Window)
        mode: VarInt,
        /// New values for affected slots
        new_slot_values: Map<'a, i16, slots::Slot, VarInt>,
        /// Item carried by the cursor
        /// Has to be empty (item ID = -1) for drop mode, otherwise nothing will happen.
        /// Is always empty for mode 2 and mode 5 packets.
        carried_item: slots::Slot,
    },

    /// This packet is sent by the client when closing a window.
    /// Notchian clients send a Close Window packet with `window_id` = 0 to close their inventory even though there is never an [ClientboundPacket::OpenWindow] packet for the inventory.
    CloseWindow {
        /// The ID of the window that was closed. 0 for player inventory.
        window_id: i8,
    },

    /// Mods and plugins can use this to send their data.
    /// Minecraft itself uses some [plugin channels](https://wiki.vg/Plugin_channel).
    /// These internal channels are in the `minecraft` namespace.
    ///
    /// [More documentation](http://dinnerbone.com/blog/2012/01/13/minecraft-plugin-channels-messaging/)
    ///
    /// *See also [ClientboundPacket::PluginMessage]*
    PluginMessage {
        /// Name of the [plugin channel](https://wiki.vg/Plugin_channel) used to send the data.
        identifier: Identifier<'a>,
        /// Any data, depending on the channel.
        /// `minecraft:` channels are documented [here](https://wiki.vg/Plugin_channel).
        data: RawBytes<'a>,
    },

    EditBook {
        /// The hotbar slot where the written book is located
        slot: VarInt,
        /// Text from each page (max 200).
        /// Maximum string length is 8192 chars.
        entries: Array<'a, &'a str, VarInt>,
        /// Some if book is being signed, None if book is being edited.
        title: Option<String>,
    },

    /// Used when F3+I is pressed while looking at an entity.
    /// 
    /// *Request for [ClientboundPacket::NbtQueryResponse]*
    QueryEntityNbt {
        /// An incremental ID so that the client can verify that the response matches
        transaction_id: VarInt,
        /// The ID of the entity to query.
        entity_id: VarInt,
    },

    /// This packet is sent from the client to the server when the client attacks or right-clicks another entity (a player, minecart, etc).
    /// A Notchian server only accepts this packet if the entity being attacked/used is visible without obstruction and within a 4-unit radius of the player's position.
    /// The target X, Y, and Z fields represent the difference between the vector location of the cursor at the time of the packet and the entity's position.
    /// Note that middle-click in creative mode is interpreted by the client and sent as a [ServerboundPacket::CreativeInventoryAction] packet instead.
    InteractEntity {
        entity_id: VarInt,
        interaction_type: entity::EntityInteraction,
        sneaking: bool,
    },

    /// Sent when Generate is pressed on the [Jigsaw Block](http://minecraft.gamepedia.com/Jigsaw_Block) interface.
    GenerateStructure {
        /// Block entity location
        location: Position,
        /// Value of the levels slider/max depth to generate
        levels: VarInt,
        keep_jigsaws: bool,
    },

    /// The server will frequently send out a keep-alive (see Clientbound Keep Alive), each containing a random ID. The client must respond with the same packet.
    /// 
    /// *Response to [ClientboundPacket::KeepAlive]*
    KeepAlive {
        /// The id sent in the [ClientboundPacket::KeepAlive] packet
        keep_alive_id: u64,
    },

    /// Appears to only be used on singleplayer; the difficulty buttons are still disabled in multiplayer.
    LockDifficulty { locked: bool },

    /// Updates the player's position on the server.
    ///
    /// Checking for moving too fast is achieved like this:
    /// - Each server tick, the player's current position is stored
    /// - When a player moves, the changes in `x`, `y`, and `z` coordinates are compared with the positions from the previous tick (`Δx`, `Δy`, `Δz`)
    /// - *Total movement distance* squared is computed as `Δx² + Δy² + Δz²`
    /// - The *expected movement distance* squared is computed as `velocityX² + veloctyY² + velocityZ²`
    /// - If the *total movement distance* squared value minus the *expected movement distance* squared value is more than 100 (300 if the player is using an elytra), they are moving too fast.
    /// 
    /// If the player is moving too fast, it will be logged that "<player> moved too quickly! " followed by the change in x, y, and z, and the player will be [teleported](ClientboundPacket::TeleportEntity) back to their current (before this packet) serverside position.
    /// Also, if the absolute value of X or the absolute value of Z is a value greater than 3.2×107, or X, Y, or Z are not finite (either positive infinity, negative infinity, or NaN), the client will be kicked for “Invalid move player packet received”.
    SetPlayerPosition {
        x: f64,
        /// The feet position (`feet_y = head_y - 1.62`)
        y: f64,
        z: f64,
        /// `true` if the client is on the ground, `false` otherwise
        on_ground: bool,
    },

    /// A combination of [ServerboundPacket::SetPlayerRotation] and [ServerboundPacket::SetPlayerPosition]
    SetPlayerPositionAndRotation {
        x: f64,
        /// The feet position (`feet_y = head_y - 1.62`)
        y: f64,
        z: f64,
        /// Absolute rotation on the X Axis, in degrees.
        /// [Learn more about yaw and pitch](https://wiki.vg/Protocol#Player_Rotation)
        yaw: f32,
        /// Absolute rotation on the Y Axis, in degrees
        /// [Learn more about yaw and pitch](https://wiki.vg/Protocol#Player_Rotation)
        pitch: f32,
        /// `true` if the client is on the ground, `false` otherwise
        on_ground: bool,
    },

    /// Updates the direction the player is looking in
    SetPlayerRotation {
        /// Absolute rotation on the X Axis, in degrees.
        /// [Learn more about yaw and pitch](https://wiki.vg/Protocol#Player_Rotation)
        yaw: f32,
        /// Absolute rotation on the Y Axis, in degrees, where 0 is looking straight ahead, -90 is looking straight up, and 90 is looking straight down.
        /// [Learn more about yaw and pitch](https://wiki.vg/Protocol#Player_Rotation)
        pitch: f32,
        /// `true` if the client is on the ground, `false` otherwise
        on_ground: bool,
    },

    /// This packet is used to indicate whether the player is on ground (walking/swimming), or airborne (jumping/falling).
    ///
    /// Vanilla clients will send Player Position once every 20 ticks even for a stationary player.
    /// 
    /// This packet is used to indicate whether the player is on ground (walking/swimming), or airborne (jumping/falling).
    ///
    /// When dropping from sufficient height, fall damage is applied when this state goes from false to true.
    /// The amount of damage applied is based on the point where it last changed from true to false.
    /// Note that there are several movement related packets containing this state.
    SetPlayerOnGround {
        /// `true` if the client is on the ground, `false` otherwise
        on_ground: bool,
    },

    /// Sent when a player moves in a vehicle.
    /// Fields are the same as in [ServerboundPacket::PlayerPositionAndRotation].
    /// Note that all fields use absolute positioning and do not allow for relative positioning.
    MoveVehicle {
        /// Absolute position
        x: f64,
        /// Absolute position
        y: f64,
        /// Absolute position
        z: f64,
        /// Absolute rotation on the X Axis, in degrees.
        /// [Learn more about yaw and pitch](https://wiki.vg/Protocol#Player_Rotation)
        yaw: f32,
        /// Absolute rotation on the Y Axis, in degrees
        /// [Learn more about yaw and pitch](https://wiki.vg/Protocol#Player_Rotation)
        pitch: f32,
    },

    /// Used to visually update whether boat paddles are turning.
    /// The server will update the [Boat entity metadata](https://wiki.vg/Entities#Boat) to match the values here.
    /// 
    /// Right paddle turning is set to true when the left button or forward button is held, left paddle turning is set to true when the right button or forward button is held.
    PaddleBoat {
        /// Left paddle turning is set to true when the right button or forward button is held.
        left_paddle_turnin: bool,
        /// Right paddle turning is set to true when the left button or forward button is held.
        right_paddle_turnin: bool,
    },

    /// Used to swap out an empty space on the hotbar with the item in the given inventory slot.
    /// The Notchain client uses this for pick block functionality (middle click) to retrieve items from the inventory.
    ///
    /// The server will first search the player's hotbar for an empty slot, starting from the current slot and looping around to the slot before it.
    /// If there are no empty slots, it will start a second search from the current slot and find the first slot that does not contain an enchanted item.
    /// If there still are no slots that meet that criteria, then the server will use the currently selected slot.
    ///
    /// After finding the appropriate slot, the server swaps the items and then send 3 packets:
    /// - [ClientboundPacket::SetSlot], with window ID set to -2 and slot set to the newly chosen slot and the item set to the item that is now in that slot (which was previously at the slot the client requested)
    /// - [ClientboundPacket::SetSlot], with window ID set to -2 and slot set to the slot the player requested, with the item that is now in that slot and was previously on the hotbar slot
    /// - [ClientboundPacket::HeldItemChange], with the slot set to the newly chosen slot.
    PickItem {
        /// See [inventory](https://wiki.vg/Inventory)
        slot_to_use: VarInt,
    },

    RequestPing {
        /// May be any number. Notchian clients use a system-dependent time value which is counted in milliseconds.
        payload: i64,
    },

    /// This packet is sent when a player clicks a recipe in the crafting book that is craftable (white border).
    PlaceRecipe {
        window_id: i8,
        recipe_id: Identifier<'a>,
        /// Affects the amount of items processed; true if shift is down when clicked.
        make_all: bool,
    },

    /// The vanilla client sends this packet when the player starts/stops flying with the `flags` field changed accordingly.
    PlayerAbilities {
        /// Bit mask. 0x02: is flying.
        flags: u8,
    },

    /// Sent when the player mines a block.
    /// A Notchian server only accepts digging packets with coordinates within a 6-unit radius between the center of the block and 1.5 units from the player's feet (not their eyes).
    DigBlock {
        /// The action the player is taking against the block
        status: crate::components::blocks::DiggingState,
        /// Block position
        location: Position,
        /// The face being hit
        face: crate::components::blocks::BlockFace,
        sequence: VarInt,
    },

    /// Sent by the client to indicate that it has performed certain actions: sneaking (crouching), sprinting, exiting a bed, jumping with a horse, and opening a horse's inventory while riding it.
    PlayerAction {
        player_id: VarInt,
        action_id: entity::PlayerAction,
        /// Only used by the [“start jump with horse” action](entity::PlayerAction::StartJumpWithHorse), in which case it ranges from 0 to 100. In all other cases it is 0.
        jump_boost: bool,
    },

    SteerVehicle {
        /// Movement to the left, can be negative to move to the right.
        to_the_left: f32,
        /// Movement forward, can be negative to move backward.
        forward: f32,
        /// Bit mask. 0x1: jump, 0x2: unmount.
        flags: u8,
    },

    /// *Response to [ClientboundPacket::Ping]*
    Pong {
        id: u32
    },

    /// Replaces Recipe Book Data, type 1.
    ChangeRecipeBookSettings {
        book: recipes::RecipeBook,
        is_open: bool,
        is_filter_active: bool,
    },

    /// Replaces Recipe Book Data, type 0.
    SetSeenRecipe {
        recipe_id: Identifier<'a>
    },

    /// Sent as a player is renaming an item in an anvil (each keypress in the anvil UI sends a new Name Item packet).
    /// If the new name is empty, then the item loses its custom name (this is different from setting the custom name to the normal name of the item).
    RenameItem {
        /// The item name may be no longer than 35 characters long, and if it is longer than that, then the rename is silently ignored.
        new_name: &'a str,
    },

    /// *Response to [ClientboundPacket::ResourcePackSend]*
    ResourcePackStatus {
        status: resource_pack::ResourcePackStatus,
    },

    SetSeenAdvancements {
        value: advancements::AdvancementTabPacket<'a>,
    },

    /// When a player selects a specific trade offered by a villager NPC.
    SelectTrade {
        /// The selected slot in the players current (trading) inventory
        selected_slot: VarInt,
    },

    /// Changes the effect of the current beacon.
    SetBeaconEffect {
        // todo, make this be a Potion
        /// A [potion ID](http://minecraft.gamepedia.com/Data_values#Potions)
        primary_effect: VarInt,
        // todo, make this be a Potion
        /// A [potion ID](http://minecraft.gamepedia.com/Data_values#Potions)
        secondary_effect: VarInt,
    },

    /// Sent when the player changes the slot selection.
    ///
    /// *See also [ClientboundPacket::HeldItemChange]*
    SetHeldItem {
        /// The slot which the player has selected (0..=8)
        slot: i16,
    },

    ProgramCommandBlock {
        location: Position,
        command: &'a str,
        mode: command_block::CommandBlockMode,
        /// Bit mask: 0x01: Track Output (if false, the output of the previous command will not be stored within the command block); 0x02: Is conditional; 0x04: Automatic.
        flags: u8,
    },

    ProgramCommandBlockMinecart {
        entity_id: VarInt,
        command: &'a str,
        /// If `false`, the output of the previous command will not be stored within the command block.
        track_output: bool,
    },

    /// While the user is in the standard inventory (i.e., not a crafting bench) in Creative mode, the player will send this packet.
    ///
    /// Clicking in the creative inventory menu is quite different from non-creative inventory management.
    /// Picking up an item with the mouse actually deletes the item from the server, and placing an item into a slot or dropping it out of the inventory actually tells the server to create the item from scratch.
    /// (This can be verified by clicking an item that you don't mind deleting, then severing the connection to the server; the item will be nowhere to be found when you log back in.)
    /// As a result of this implementation strategy, the "Destroy Item" slot is just a client-side implementation detail that means "I don't intend to recreate this item.".
    /// Additionally, the long listings of items (by category, etc.) are a client-side interface for choosing which item to create.
    /// Picking up an item from such listings sends no packets to the server; only when you put it somewhere does it tell the server to create the item in that location.
    /// 
    /// This action can be described as "set inventory slot".
    /// Picking up an item sets the slot to item ID -1.
    /// Placing an item into an inventory slot sets the slot to the specified item. Dropping an item (by clicking outside the window) effectively sets slot -1 to the specified item, which causes the server to spawn the item entity, etc..
    /// All other inventory slots are numbered the same as the non-creative inventory (including slots for the 2x2 crafting menu, even though they aren't visible in the vanilla client).
    SetCreativeModeSlot {
        id: i16,
        clicked_item: slots::Slot,
    },

    /// Sent when Done is pressed on the [Jigsaw Block](http://minecraft.gamepedia.com/Jigsaw_Block) interface.
    ProgramJigsawBlock {
        /// Block entity location
        location: Position,
        name: Identifier<'a>,
        target: Identifier<'a>,
        pool: Identifier<'a>,
        /// "Turns into" on the GUI, `final_state` in NBT
        final_state: &'a str,
        /// `rollable` if the attached piece can be rotated, else `aligned`
        joint_type: &'a str,
    },

    // TODO add enums
    ProgramStrutureBlock {
        /// Block entity location.
        location: Position,
        /// An additional action to perform beyond simply saving the given data; see below.
        action: VarInt,
        mode: VarInt,
        name: &'a str,
        /// Between -48 and 48.
        offset_x: i8,
        /// Between -48 and 48.
        offset_y: i8,
        /// Between -48 and 48.
        offset_z: i8,
        /// Between 0 and 48.
        size_x: i8,
        /// Between 0 and 48.
        size_y: i8,
        /// Between 0 and 48.
        size_z: i8,
        /// One of NONE (0), LEFT_RIGHT (1), FRONT_BACK (2).
        mirror: VarInt,
        /// One of NONE (0), CLOCKWISE_90 (1), CLOCKWISE_180 (2), COUNTERCLOCKWISE_90 (3).
        rotation: VarInt,
        metadata: &'a str,
        /// Between 0 and 1.
        integrity: f32,
        seed: VarLong,
        /// 0x01: Ignore entities; 0x02: Show air; 0x04: Show bounding box.
        flags: u8,
    },

    /// This message is sent from the client to the server when the “Done” button is pushed after placing a sign.
    /// The server only accepts this packet after [ClientboundPacket::OpenSignEditor], otherwise this packet is silently ignored.
    ///
    /// *Response to [ClientboundPacket::OpenSignEditor]*
    UpdateSign {
        /// Sign block Coordinates
        location: Position,
        /// Whether the updated text is in front or on the back of the sign
        is_front_text: bool,
        line1: &'a str,
        line2: &'a str,
        line3: &'a str,
        line4: &'a str,
    },

    /// Sent when the player's arm swings
    SwingArms { hand: slots::Hand },

    /// Teleports the player to the given entity.
    /// The player must be in spectator mode.
    ///
    /// The Notchian client only uses this to teleport to players, but it appears to accept any type of entity.
    /// The entity does not need to be in the same dimension as the player; if necessary, the player will be respawned in the right world.
    /// If the given entity cannot be found (or isn't loaded), this packet will be ignored. It will also be ignored if the player attempts to teleport to themselves.
    Spectate {
        /// UUID of the player to teleport to (can also be an entity UUID)
        target_uuid: UUID,
    },

    /// Upon placing a block, this packet is sent once.
    ///
    /// *Note*: The Cursor Position X/Y/Z fields (also known as in-block coordinates) are calculated using raytracing.
    /// The unit corresponds to sixteen pixels in the default resource pack.
    /// For example, let's say a slab is being placed against the south face of a full block.
    /// The Cursor Position X will be higher if the player was pointing near the right (east) edge of the face, lower if pointing near the left.
    /// The Cursor Position Y will be used to determine whether it will appear as a bottom slab (values 0.0–0.5) or as a top slab (values 0.5-1.0).
    /// The Cursor Position Z should be 1.0 since the player was looking at the southernmost part of the block.
    PlaceBlock {
        hand: slots::Hand,
        location: Position,
        face: crate::components::blocks::BlockFace,
        /// The position of the crosshair on the block, from 0 to 1 increasing from west to east.
        cursor_position_x: f32,
        /// The position of the crosshair on the block, from 0 to 1 increasing from bottom to top.
        cursor_position_y: f32,
        /// The position of the crosshair on the block, from 0 to 1 increasing from north to south.
        cursor_position_z: f32,
        /// `true` when the player's head is inside of a block.
        ///
        /// Inside block is true when a player's head (specifically eyes) are inside of a block's collision.
        /// In 1.13 and later versions, collision is rather complicated and individual blocks can have multiple collision boxes.
        /// For instance, a ring of vines has a non-colliding hole in the middle.
        /// This value is only true when the player is directly in the box.
        /// In practice, though, this value is only used by scaffolding to place in front of the player when sneaking inside of it (other blocks will place behind when you intersect with them -- try with glass for instance).
        inside_block: bool,
        sequence: VarInt,
    },

    /// Sent when pressing the Use Item key (default: right click) with an item in hand.
    UseItem {
        /// Hand used for the animation
        hand: slots::Hand,
        sequence: VarInt,
    },
}
