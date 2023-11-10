use super::*;

#[MinecraftEntity(
    inheritable
)]
pub struct Entity {
    pub position: Position,
    pub is_on_fire: bool,
    pub is_crouching: bool,
    pub is_sprinting: bool,
    pub is_swimming: bool,
    pub is_invisible: bool,
    pub is_glowing: bool,
    pub is_fying_with_elytra: bool,
    pub air_ticks: u32,
    pub name: Option<String>,
    pub is_name_visible: bool,
    pub is_silent: bool,
    pub has_no_gravity: bool,
    pub pose: Pose,
    pub ticks_frozen: u32,
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            position: Position { x: 0.0, y: 0.0, z: 0.0 },
            is_on_fire: false,
            is_crouching: false,
            is_sprinting: false,
            is_swimming: false,
            is_invisible: false,
            is_glowing: false,
            is_fying_with_elytra: false,
            air_ticks: 300,
            name: None,
            is_name_visible: false,
            is_silent: false,
            has_no_gravity: false,
            pose: Pose::Standing,
            ticks_frozen: 0,
        }
    }
}

impl TryAsEntityRef<Entity> for AnyEntity {
    fn try_as_entity_ref(&self) -> Option<&Entity> {
        Some(match self {
            AnyEntity::Entity(entity) => entity,
            AnyEntity::Display(display) => display.get_entity(),
            AnyEntity::BlockDisplay(block_display) => block_display.get_entity(),
            AnyEntity::ItemDisplay(item_display) => item_display.get_entity(),
            AnyEntity::TextDisplay(text_display) => text_display.get_entity(),
            AnyEntity::ThrownItemProjectile(throw_item_projectile) => throw_item_projectile.get_entity(),
            AnyEntity::ThrownEgg(throw_egg) => throw_egg.get_entity(),
            AnyEntity::ThrownEnderPearl(throw_ender_pearl) => throw_ender_pearl.get_entity(),
            AnyEntity::ThrownExperienceBottle(throw_experience_bottle) => throw_experience_bottle.get_entity(),
            AnyEntity::ThrownPotion(throw_potion) => throw_potion.get_entity(),
            AnyEntity::Snowball(snowball) => snowball.get_entity(),
            AnyEntity::AbstractArrow(abstract_arrow) => abstract_arrow.get_entity(),
            AnyEntity::Arrow(arrow) => arrow.get_entity(),
            AnyEntity::SpectralArrow(spectral_arrow) => spectral_arrow.get_entity(),
            AnyEntity::ThrownTrident(throw_trident) => throw_trident.get_entity(),
            AnyEntity::Boat(boat) => boat.get_entity(),
            AnyEntity::ChestBoat(chest_boat) => chest_boat.get_entity(),
            AnyEntity::LivingEntity(living_entity) => living_entity.get_entity(),
            AnyEntity::Player(player) => player.get_entity(),
            AnyEntity::Mob(mob) => mob.get_entity(),
            AnyEntity::AmbientCreature(ambient_creature) => ambient_creature.get_entity(),
            AnyEntity::Bat(bat) => bat.get_entity(),
            AnyEntity::PathfinderMob(pathfinder_mob) => pathfinder_mob.get_entity(),
            AnyEntity::WaterAnimal(water_animal) => water_animal.get_entity(),
            AnyEntity::Squid(squid) => squid.get_entity(),
            AnyEntity::AgeableMob(ageable_mob) => ageable_mob.get_entity(),
            AnyEntity::Animal(animal) => animal.get_entity(),
            AnyEntity::Sniffer(sniffer) => sniffer.get_entity(),
            AnyEntity::AbstractHorse(abstract_horse) => abstract_horse.get_entity(),
            AnyEntity::ZombieHorse(zombie_horse) => zombie_horse.get_entity(),
            AnyEntity::Horse(horse) => horse.get_entity(),
            AnyEntity::SkeletonHorse(skeleton_horse) => skeleton_horse.get_entity(),
            AnyEntity::Camel(camel) => camel.get_entity(),
            AnyEntity::ChestedHorse(chested_horse) => chested_horse.get_entity(),
            AnyEntity::Donkey(donkey) => donkey.get_entity(),
            AnyEntity::Llama(llama) => llama.get_entity(),
            AnyEntity::TraderLlama(trader_llama) => trader_llama.get_entity(),
            AnyEntity::Mule(mule) => mule.get_entity(),
            AnyEntity::Axolotl(axolotl) => axolotl.get_entity(),
            AnyEntity::Bee(bee) => bee.get_entity(),
            AnyEntity::Fox(fox) => fox.get_entity(),
            AnyEntity::Frog(frog) => frog.get_entity(),
            AnyEntity::Ocelot(ocelot) => ocelot.get_entity(),
            AnyEntity::Panda(panda) => panda.get_entity(),
            AnyEntity::Pig(pig) => pig.get_entity(),
            AnyEntity::Rabbit(rabbit) => rabbit.get_entity(),
            AnyEntity::Turtle(turtle) => turtle.get_entity(),
            AnyEntity::PolarBear(polar_bear) => polar_bear.get_entity(),
            AnyEntity::Chicken(chicken) => chicken.get_entity(),
            AnyEntity::Cow(cow) => cow.get_entity(),
            AnyEntity::Hoglin(hoglin) => hoglin.get_entity(),
            AnyEntity::Mooshroom(mooshroom) => mooshroom.get_entity(),
            AnyEntity::Sheep(sheep) => sheep.get_entity(),
            AnyEntity::Strider(strider) => strider.get_entity(),
            AnyEntity::TameableAnimal(tameable_animal) => tameable_animal.get_entity(),
            AnyEntity::Cat(cat) => cat.get_entity(),
            AnyEntity::Wolf(wolf) => wolf.get_entity(),
            AnyEntity::Parrot(parrot) => parrot.get_entity(),
            AnyEntity::AbstractVillager(abstract_villager) => abstract_villager.get_entity(),
            AnyEntity::Villager(villager) => villager.get_entity(),
            AnyEntity::WanderingTrader(wandering_trader) => wandering_trader.get_entity(),
            AnyEntity::AbstractGolem(abstract_golem) => abstract_golem.get_entity(),
            AnyEntity::IronGolem(iron_golem) => iron_golem.get_entity(),
            AnyEntity::SnowGolem(snow_golem) => snow_golem.get_entity(),
            AnyEntity::Shulker(shulker) => shulker.get_entity(),
            AnyEntity::Monster(monster) => monster.get_entity(),
            AnyEntity::BasePiglin(base_piglin) => base_piglin.get_entity(),
            AnyEntity::Piglin(piglin) => piglin.get_entity(),
            AnyEntity::PiglinBrute(piglin_brute) => piglin_brute.get_entity(),
            AnyEntity::Blaze(blaze) => blaze.get_entity(),
            AnyEntity::Creeper(creeper) => creeper.get_entity(),
            AnyEntity::Endermite(endermite) => endermite.get_entity(),
            AnyEntity::Giant(giant) => giant.get_entity(),
            AnyEntity::Goat(goat) => goat.get_entity(),
            AnyEntity::Guardian(guardian) => guardian.get_entity(),
            AnyEntity::ElderGuardian(elder_guardian) => elder_guardian.get_entity(),
            AnyEntity::Silverfish(silverfish) => silverfish.get_entity(),
            AnyEntity::Raider(raider) => raider.get_entity(),
            AnyEntity::AbstractIllager(abstract_illager) => abstract_illager.get_entity(),
            AnyEntity::Vindicator(vindicator) => vindicator.get_entity(),
            AnyEntity::Pillager(pillager) => pillager.get_entity(),
            AnyEntity::SpellcasterIllager(spellcaster_illager) => spellcaster_illager.get_entity(),
            AnyEntity::Evoker(evoker) => evoker.get_entity(),
            AnyEntity::Illusioner(illusioner) => illusioner.get_entity(),
            AnyEntity::Ravager(ravager) => ravager.get_entity(),
            AnyEntity::Witch(witch) => witch.get_entity(),
            AnyEntity::EvokerFangs(evoker_fangs) => evoker_fangs.get_entity(),
            AnyEntity::Vex(vex) => vex.get_entity(),
            AnyEntity::Skeleton(skeleton) => skeleton.get_entity(),
            AnyEntity::AbstractSkeleton(abstract_skeleton) => abstract_skeleton.get_entity(), 
            AnyEntity::WitherSkeleton(wither_skeleton) => wither_skeleton.get_entity(),
            AnyEntity::Stray(stray) => stray.get_entity(), 
            AnyEntity::Spider(spider) => spider.get_entity(),      
            AnyEntity::Warden(warden) => warden.get_entity(),
            AnyEntity::Wither(wither) => wither.get_entity(),
            AnyEntity::Zoglin(zoglin) => zoglin.get_entity(),
            AnyEntity::Zombie(zombie) => zombie.get_entity(),
            AnyEntity::ZombieVillager(zombie_villager) => zombie_villager.get_entity(),
            AnyEntity::Husk(husk) => husk.get_entity(),
            AnyEntity::Drowned(drowned) => drowned.get_entity(),
            AnyEntity::ZombifiedPiglin(zombified_piglin) => zombified_piglin.get_entity(),
            AnyEntity::Enderman(enderman) => enderman.get_entity(),
            AnyEntity::EnderDragon(ender_dragon) => ender_dragon.get_entity(),
            AnyEntity::Flying(flying) => flying.get_entity(),
            AnyEntity::Ghast(ghast) => ghast.get_entity(),
            AnyEntity::Phantom(phantom) => phantom.get_entity(),
            AnyEntity::Slime(slime) => slime.get_entity(),
            AnyEntity::LlamaSpit(llama_spit) => llama_spit.get_entity(),
            AnyEntity::EyeOfEnder(eye_of_ender) => eye_of_ender.get_entity(),
            AnyEntity::FallingBlock(falling_block) => falling_block.get_entity(),
            AnyEntity::AreaEffectCloud(area_effect_cloud) => area_effect_cloud.get_entity(),
            AnyEntity::FishingHook(fishing_hook) => fishing_hook.get_entity(),
            AnyEntity::EndCrystal(end_crystal) => end_crystal.get_entity(),
            AnyEntity::DragonFireball(dragon_fireball) => dragon_fireball.get_entity(),
            AnyEntity::SmallFireball(small_fireball) => small_fireball.get_entity(),
            AnyEntity::Fireball(fireball) => fireball.get_entity(),
            AnyEntity::WitherSkull(wither_skull) => wither_skull.get_entity(),
            AnyEntity::FireworkRocket(firework_rocket) => firework_rocket.get_entity(),
            AnyEntity::ItemFrame(item_frame) => item_frame.get_entity(),
            AnyEntity::GlowingItemFrame(glowing_item_frame) => glowing_item_frame.get_entity(),
            AnyEntity::Painting(painting) => painting.get_entity(),
            AnyEntity::ItemEntity(item_entity) => item_entity.get_entity(),
            AnyEntity::ArmorStand(armor_stand) => armor_stand.get_entity(),
            AnyEntity::Dolphin(dolphin) => dolphin.get_entity(),
            AnyEntity::AbstractFish(abstract_fish) => abstract_fish.get_entity(),
            AnyEntity::Cod(cod) => cod.get_entity(),
            AnyEntity::PufferFish(pufferfish) => pufferfish.get_entity(),
            AnyEntity::Salmon(salmon) => salmon.get_entity(),
            AnyEntity::TropicalFish(tropical_fish) => tropical_fish.get_entity(),
            AnyEntity::Tadpole(tadpole) => tadpole.get_entity(),
        })
    }

    fn try_as_entity_mut(&mut self) -> Option<&mut Entity> {
        Some(match self {
            AnyEntity::Entity(entity) => entity,
            AnyEntity::Display(display) => display.get_entity_mut(),
            AnyEntity::BlockDisplay(block_display) => block_display.get_entity_mut(),
            AnyEntity::ItemDisplay(item_display) => item_display.get_entity_mut(),
            AnyEntity::TextDisplay(text_display) => text_display.get_entity_mut(),
            AnyEntity::ThrownItemProjectile(throw_item_projectile) => throw_item_projectile.get_entity_mut(),
            AnyEntity::ThrownEgg(throw_egg) => throw_egg.get_entity_mut(),
            AnyEntity::ThrownEnderPearl(throw_ender_pearl) => throw_ender_pearl.get_entity_mut(),
            AnyEntity::ThrownExperienceBottle(throw_experience_bottle) => throw_experience_bottle.get_entity_mut(),
            AnyEntity::ThrownPotion(throw_potion) => throw_potion.get_entity_mut(),
            AnyEntity::Snowball(snowball) => snowball.get_entity_mut(),
            AnyEntity::AbstractArrow(abstract_arrow) => abstract_arrow.get_entity_mut(),
            AnyEntity::Arrow(arrow) => arrow.get_entity_mut(),
            AnyEntity::SpectralArrow(spectral_arrow) => spectral_arrow.get_entity_mut(),
            AnyEntity::ThrownTrident(throw_trident) => throw_trident.get_entity_mut(),
            AnyEntity::Boat(boat) => boat.get_entity_mut(),
            AnyEntity::ChestBoat(chest_boat) => chest_boat.get_entity_mut(),
            AnyEntity::LivingEntity(living_entity) => living_entity.get_entity_mut(),
            AnyEntity::Player(player) => player.get_entity_mut(),
            AnyEntity::Mob(mob) => mob.get_entity_mut(),
            AnyEntity::AmbientCreature(ambient_creature) => ambient_creature.get_entity_mut(),
            AnyEntity::Bat(bat) => bat.get_entity_mut(),
            AnyEntity::PathfinderMob(pathfinder_mob) => pathfinder_mob.get_entity_mut(),
            AnyEntity::WaterAnimal(water_animal) => water_animal.get_entity_mut(),
            AnyEntity::Squid(squid) => squid.get_entity_mut(),
            AnyEntity::AgeableMob(ageable_mob) => ageable_mob.get_entity_mut(),
            AnyEntity::Animal(animal) => animal.get_entity_mut(),
            AnyEntity::Sniffer(sniffer) => sniffer.get_entity_mut(),
            AnyEntity::AbstractHorse(abstract_horse) => abstract_horse.get_entity_mut(),
            AnyEntity::ZombieHorse(zombie_horse) => zombie_horse.get_entity_mut(),
            AnyEntity::Horse(horse) => horse.get_entity_mut(),
            AnyEntity::SkeletonHorse(skeleton_horse) => skeleton_horse.get_entity_mut(),
            AnyEntity::Camel(camel) => camel.get_entity_mut(),
            AnyEntity::ChestedHorse(chested_horse) => chested_horse.get_entity_mut(),
            AnyEntity::Donkey(donkey) => donkey.get_entity_mut(),
            AnyEntity::Llama(llama) => llama.get_entity_mut(),
            AnyEntity::TraderLlama(trader_llama) => trader_llama.get_entity_mut(),
            AnyEntity::Mule(mule) => mule.get_entity_mut(),
            AnyEntity::Axolotl(axolotl) => axolotl.get_entity_mut(),
            AnyEntity::Bee(bee) => bee.get_entity_mut(),
            AnyEntity::Fox(fox) => fox.get_entity_mut(),
            AnyEntity::Frog(frog) => frog.get_entity_mut(),
            AnyEntity::Ocelot(ocelot) => ocelot.get_entity_mut(),
            AnyEntity::Panda(panda) => panda.get_entity_mut(),
            AnyEntity::Pig(pig) => pig.get_entity_mut(),
            AnyEntity::Rabbit(rabbit) => rabbit.get_entity_mut(),
            AnyEntity::Turtle(turtle) => turtle.get_entity_mut(),
            AnyEntity::PolarBear(polar_bear) => polar_bear.get_entity_mut(),
            AnyEntity::Chicken(chicken) => chicken.get_entity_mut(),
            AnyEntity::Cow(cow) => cow.get_entity_mut(),
            AnyEntity::Hoglin(hoglin) => hoglin.get_entity_mut(),
            AnyEntity::Mooshroom(mooshroom) => mooshroom.get_entity_mut(),
            AnyEntity::Sheep(sheep) => sheep.get_entity_mut(),
            AnyEntity::Strider(strider) => strider.get_entity_mut(),
            AnyEntity::TameableAnimal(tameable_animal) => tameable_animal.get_entity_mut(),
            AnyEntity::Cat(cat) => cat.get_entity_mut(),
            AnyEntity::Wolf(wolf) => wolf.get_entity_mut(),
            AnyEntity::Parrot(parrot) => parrot.get_entity_mut(),
            AnyEntity::AbstractVillager(abstract_villager) => abstract_villager.get_entity_mut(),
            AnyEntity::Villager(villager) => villager.get_entity_mut(),
            AnyEntity::WanderingTrader(wandering_trader) => wandering_trader.get_entity_mut(),
            AnyEntity::AbstractGolem(abstract_golem) => abstract_golem.get_entity_mut(),
            AnyEntity::IronGolem(iron_golem) => iron_golem.get_entity_mut(),
            AnyEntity::SnowGolem(snow_golem) => snow_golem.get_entity_mut(),
            AnyEntity::Shulker(shulker) => shulker.get_entity_mut(),
            AnyEntity::Monster(monster) => monster.get_entity_mut(),
            AnyEntity::BasePiglin(base_piglin) => base_piglin.get_entity_mut(),
            AnyEntity::Piglin(piglin) => piglin.get_entity_mut(),
            AnyEntity::PiglinBrute(piglin_brute) => piglin_brute.get_entity_mut(),
            AnyEntity::Blaze(blaze) => blaze.get_entity_mut(),
            AnyEntity::Creeper(creeper) => creeper.get_entity_mut(),
            AnyEntity::Endermite(endermite) => endermite.get_entity_mut(),
            AnyEntity::Giant(giant) => giant.get_entity_mut(),
            AnyEntity::Goat(goat) => goat.get_entity_mut(),
            AnyEntity::Guardian(guardian) => guardian.get_entity_mut(),
            AnyEntity::ElderGuardian(elder_guardian) => elder_guardian.get_entity_mut(),
            AnyEntity::Silverfish(silverfish) => silverfish.get_entity_mut(),
            AnyEntity::Raider(raider) => raider.get_entity_mut(),
            AnyEntity::AbstractIllager(abstract_illager) => abstract_illager.get_entity_mut(),
            AnyEntity::Vindicator(vindicator) => vindicator.get_entity_mut(),
            AnyEntity::Pillager(pillager) => pillager.get_entity_mut(),
            AnyEntity::SpellcasterIllager(spellcaster_illager) => spellcaster_illager.get_entity_mut(),
            AnyEntity::Evoker(evoker) => evoker.get_entity_mut(),
            AnyEntity::Illusioner(illusioner) => illusioner.get_entity_mut(),
            AnyEntity::Ravager(ravager) => ravager.get_entity_mut(),
            AnyEntity::Witch(witch) => witch.get_entity_mut(),
            AnyEntity::EvokerFangs(evoker_fangs) => evoker_fangs.get_entity_mut(),
            AnyEntity::Vex(vex) => vex.get_entity_mut(),
            AnyEntity::Skeleton(skeleton) => skeleton.get_entity_mut(),
            AnyEntity::AbstractSkeleton(abstract_skeleton) => abstract_skeleton.get_entity_mut(), 
            AnyEntity::WitherSkeleton(wither_skeleton) => wither_skeleton.get_entity_mut(),
            AnyEntity::Stray(stray) => stray.get_entity_mut(), 
            AnyEntity::Spider(spider) => spider.get_entity_mut(),
            AnyEntity::Warden(warden) => warden.get_entity_mut(),
            AnyEntity::Wither(wither) => wither.get_entity_mut(),
            AnyEntity::Zoglin(zoglin) => zoglin.get_entity_mut(),
            AnyEntity::Zombie(zombie) => zombie.get_entity_mut(),
            AnyEntity::ZombieVillager(zombie_villager) => zombie_villager.get_entity_mut(),
            AnyEntity::Husk(husk) => husk.get_entity_mut(),
            AnyEntity::Drowned(drowned) => drowned.get_entity_mut(),
            AnyEntity::ZombifiedPiglin(zombified_piglin) => zombified_piglin.get_entity_mut(),
            AnyEntity::Enderman(enderman) => enderman.get_entity_mut(),
            AnyEntity::EnderDragon(ender_dragon) => ender_dragon.get_entity_mut(),
            AnyEntity::Flying(flying) => flying.get_entity_mut(),
            AnyEntity::Ghast(ghast) => ghast.get_entity_mut(),
            AnyEntity::Phantom(phantom) => phantom.get_entity_mut(),
            AnyEntity::Slime(slime) => slime.get_entity_mut(),
            AnyEntity::LlamaSpit(llama_spit) => llama_spit.get_entity_mut(),
            AnyEntity::EyeOfEnder(eye_of_ender) => eye_of_ender.get_entity_mut(),
            AnyEntity::FallingBlock(falling_block) => falling_block.get_entity_mut(),
            AnyEntity::AreaEffectCloud(area_effect_cloud) => area_effect_cloud.get_entity_mut(),
            AnyEntity::FishingHook(fishing_hook) => fishing_hook.get_entity_mut(),
            AnyEntity::EndCrystal(end_crystal) => end_crystal.get_entity_mut(),
            AnyEntity::DragonFireball(dragon_fireball) => dragon_fireball.get_entity_mut(),
            AnyEntity::SmallFireball(small_fireball) => small_fireball.get_entity_mut(),
            AnyEntity::Fireball(fireball) => fireball.get_entity_mut(),
            AnyEntity::WitherSkull(wither_skull) => wither_skull.get_entity_mut(),
            AnyEntity::FireworkRocket(firework_rocket) => firework_rocket.get_entity_mut(),
            AnyEntity::ItemFrame(item_frame) => item_frame.get_entity_mut(),
            AnyEntity::GlowingItemFrame(glowing_item_frame) => glowing_item_frame.get_entity_mut(),
            AnyEntity::Painting(painting) => painting.get_entity_mut(),
            AnyEntity::ItemEntity(item_entity) => item_entity.get_entity_mut(),
            AnyEntity::ArmorStand(armor_stand) => armor_stand.get_entity_mut(),
            AnyEntity::Dolphin(dolphin) => dolphin.get_entity_mut(),
            AnyEntity::AbstractFish(abstract_fish) => abstract_fish.get_entity_mut(),
            AnyEntity::Cod(cod) => cod.get_entity_mut(),
            AnyEntity::PufferFish(pufferfish) => pufferfish.get_entity_mut(),
            AnyEntity::Salmon(salmon) => salmon.get_entity_mut(),
            AnyEntity::TropicalFish(tropical_fish) => tropical_fish.get_entity_mut(),
            AnyEntity::Tadpole(tadpole) => tadpole.get_entity_mut(),
        })
    }
}
