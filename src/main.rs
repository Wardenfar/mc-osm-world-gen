use std::fs::File;
use std::path::Path;

use feather_base::{Biome, BlockPosition, Chunk, ChunkPosition};
use feather_base::anvil::level::{LevelData, SuperflatGeneratorOptions};
use feather_base::anvil::region::{create_region, Error, load_region, RegionHandle, RegionPosition};
use feather_blocks::BlockId;
use feather_common::{Game, World};
use feather_common::world_source::flat::FlatWorldSource;
use feather_common::world_source::region::RegionWorldSource;
use feather_common::world_source::WorldSource;
use feather_worldgen::{SuperflatWorldGenerator, WorldGenerator};

fn main() {
    // let level = generate_level();
    // let mut file = File::create("world/level.dat").expect("open level.dat");
    // level.save_to_file(&mut file).expect("write to level.dat");

    let mut region = create_region(
        Path::new("world"),
        RegionPosition::from_chunk(
            ChunkPosition::new(0, 0)
        )).expect("create region");

    for x in 0..15 {
        for z in 0..15 {
            println!("{}/{}", x * 16 + z, 16 * 16);
            fill_chunk(&mut region, x, z);
        }
    }
}

fn fill_chunk(region: &mut RegionHandle, x: i32, z: i32) {
    let &mut pos = &mut ChunkPosition::new(x, z);

    let mut chunk = Chunk::new(pos);

    for y in 0..7 {
        chunk.fill_section(y, BlockId::oak_planks());
    }

    let entities = Vec::new();
    let block_entities = Vec::new();

    region.save_chunk(&chunk, &entities, &block_entities).expect("Cant save the chunk");
}

fn generate_level() -> LevelData {
    LevelData {
        allow_commands: true,
        border_center_x: 0.0,
        border_center_z: 0.0,
        border_damage_per_block: 0.0,
        border_safe_zone: 0.0,
        border_size: 0.0,
        clear_weather_time: 0,
        data_version: 0,
        day_time: 0,
        difficulty: 0,
        difficulty_locked: 0,
        game_type: 0,
        hardcore: false,
        initialized: false,
        last_played: 0,
        raining: false,
        rain_time: 0,
        seed: 0,
        spawn_x: 0,
        spawn_y: 100,
        spawn_z: 0,
        thundering: false,
        thunder_time: 0,
        time: 0,
        version: Default::default(),
        generator_name: String::from("minecraft:flat"),
        generator_options: None,
    }
}
