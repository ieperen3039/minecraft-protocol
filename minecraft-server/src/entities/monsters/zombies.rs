use minecraft_protocol::network;

use super::*;

#[derive(Default)]
#[MinecraftEntity(
    inheritable,
    ancestors { Monster, PathfinderMob, Mob, LivingEntity, Entity },
    descendants { ZombieVillager, Husk, Drowned, ZombifiedPiglin },
    defines {
        Entity.init(self, server_msg_rcvr: BroadcastReceiver<ServerMessage>);
    }
)]
pub struct Zombie {
    pub monster: Monster,
    pub is_baby: bool,
    pub unused: isize,
    pub is_becoming_drowned: bool,
}

impl Handler<Zombie> {
    pub async fn init(self, server_msg_rcvr: BroadcastReceiver<ServerMessage>) {
        self.insert_task("newton", tokio::spawn(newton_task(self.clone(), server_msg_rcvr.resubscribe()))).await;
        self.insert_task("zombie-ai", tokio::spawn(zombie_ai_task(self.clone(), server_msg_rcvr))).await;
    }
}

pub async fn zombie_ai_task<T: EntityDescendant + ZombieDescendant>(h: Handler<T>, mut server_msg_rcvr: BroadcastReceiver<ServerMessage>) where AnyEntity: TryAsEntityRef<T> {
    loop {
        let msg = server_msg_rcvr.recv().await.unwrap();

        if !matches!(&msg, &ServerMessage::Tick) {
            continue;
        }
        
        let self_position = h.observe(|e| e.get_entity().position.clone()).await.unwrap();
        let chunk = self_position.chunk_column();
        let player_positions = h.world.observe_entities(chunk, |entity| {
            let network_entity = entity.to_network().unwrap();
            TryAsEntityRef::<Player>::try_as_entity_ref(entity).map(|player| {
                (player.get_entity().position.clone(), network_entity)
            })
        }).await;

        let Some((target_position, network_entity)) = player_positions.get(0) else { continue };
        let target_object = CollisionShape {
            x1: target_position.x - network_entity.width() as f64 / 2.0,
            y1: target_position.y,
            z1: target_position.z - network_entity.width() as f64 / 2.0,
            x2: target_position.x + network_entity.width() as f64 / 2.0,
            y2: target_position.y + network_entity.height() as f64,
            z2: target_position.z + network_entity.width() as f64 / 2.0,
        };
        
        let mut translation = Translation {
            x: target_position.x - self_position.x,
            y: target_position.y - self_position.y,
            z: target_position.z - self_position.z,
        };
        translation.set_norm(0.23000000417232513);

        let authorized_translation = h.world.try_move(&target_object, &translation).await;

        h.mutate(|e| {
            e.get_entity_mut().position += authorized_translation;
            ((), EntityChanges::position())
        }).await;
    }
}

#[derive(Default)]
#[MinecraftEntity(
    ancestors { Zombie, Monster, PathfinderMob, Mob, LivingEntity, Entity },
)]
pub struct ZombieVillager {
    pub zombie: Zombie,
    pub is_converting: bool,
    pub villager_data: Vec<u8>,
}

#[derive(Default)]
#[MinecraftEntity(
    ancestors { Zombie, Monster, PathfinderMob, Mob, LivingEntity, Entity },
)]
pub struct Husk {
    pub zombie: Zombie,
}

#[derive(Default)]
#[MinecraftEntity(
    ancestors { Zombie, Monster, PathfinderMob, Mob, LivingEntity, Entity },
)]
pub struct Drowned {
    pub zombie: Zombie,
}

#[derive(Default)]
#[MinecraftEntity(
    ancestors { Zombie, Monster, PathfinderMob, Mob, LivingEntity, Entity },
)]
pub struct ZombifiedPiglin {
    pub zombie: Zombie,
}