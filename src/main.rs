use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::process::exit;

use feather_base::{Biome, BlockPosition, Chunk, ChunkPosition};
use feather_base::anvil::level::{LevelData, SuperflatGeneratorOptions};
use feather_base::anvil::region::{create_region, Error, load_region, RegionHandle, RegionPosition};
use feather_blocks::BlockId;
use feather_common::{Game, World};
use feather_common::world_source::flat::FlatWorldSource;
use feather_common::world_source::region::RegionWorldSource;
use feather_common::world_source::WorldSource;
use feather_worldgen::{SuperflatWorldGenerator, WorldGenerator};
use protobuf::{CodedInputStream, Message};

use renderer::coords::Coords;
use renderer::draw::drawer::Drawer;
use renderer::draw::tile_pixels::TilePixels;
use renderer::geodata::reader::{GeodataReader, OsmEntities};
use renderer::mapcss::parser::parse_file;
use renderer::mapcss::styler::{Styler, StyleType};
use renderer::tile::{coords_to_max_zoom_tile, MAX_ZOOM, Tile, tile_to_max_zoom_tile_range, coords_to_zoom_tile};
use threadpool::ThreadPool;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize, AtomicI32};

mod vector_tile;

fn main() {
    // let level = generate_level();
    // let mut file = File::create("world/level.dat").expect("open level.dat");
    // level.save_to_file(&mut file).expect("write to level.dat");

    // let mut file = File::open("0.mvt").expect("failed open mvt");
    // let mut reader = BufReader::new(file);
    //
    // let tile = Tile::parse_from_reader(&mut reader).unwrap();
    //
    // println!("{:?}", tile);

    // match renderer::geodata::importer::import("herblay.osm", "herblay.bin") {
    //     Ok(_) => println!("All good"),
    //     Err(err) => {
    //         for cause in err.chain() {
    //             eprintln!("{}", cause);
    //         }
    //         std::process::exit(1);
    //     }
    // }

    let storage = GeodataReader::load("herblay.bin").unwrap();

    let storage_arc = Arc::new(storage);

    let bbox = storage_arc.boundingbox();
    let min_corner = (bbox.min_lat, bbox.min_lon);
    let max_corner = (bbox.max_lat, bbox.max_lon);

    let zoom = 15;

    let min_tile = coords_to_zoom_tile(&min_corner, zoom);
    let max_tile = coords_to_zoom_tile(&max_corner, zoom);

    let min_x = min(max_tile.x, min_tile.x);
    let min_y = min(max_tile.y, min_tile.y);
    let max_x = max(max_tile.x, min_tile.x) + 1;
    let max_y = max(max_tile.y, min_tile.y) + 1;

    let range_x = (max_x - min_x);
    let range_y = (max_y - min_y);

    let pool = ThreadPool::new(16);

    let count_lock = Arc::new(AtomicI32::new(0));

    for x in min_x..max_x {
        for y in min_y..max_y {

            let tile = Tile {
                zoom,
                x,
                y,
            };
            let drawer = Drawer::new("output".as_ref());

            let storage_clone = storage_arc.clone();
            let count_clone  = Arc::clone(&count_lock);
            pool.execute(move || {
                let entities = storage_clone.get_entities_in_tile_with_neighbors(&tile, &None);
                render_tile(entities, &drawer, &tile);

                let current = count_clone.fetch_add(1, Ordering::SeqCst);
                println!("{} / {}", current, range_x * range_y)
            });
        }
    }

    pool.join()


    // let mut region = create_region(
    //     Path::new("world"),
    //     RegionPosition::from_chunk(
    //         ChunkPosition::new(0, 0)
    //     )).expect("create region");
    //
    // for x in 0..15 {
    //     for z in 0..15 {
    //         println!("{}/{}", x * 16 + z, 16 * 16);
    //         fill_chunk(&mut region, x, z);
    //     }
    // }
}

fn render_tile(entities: OsmEntities, drawer: &Drawer, tile: &Tile) {
    let filename = format!("tiles/tile_{}_{}.png", tile.x, tile.y);
    let scale = 2;
    let text_scale: f64 = 0.0;

    if Path::new(&filename).exists() {
        return
    }

    let mut pixels = TilePixels::new(scale);

    let rules = parse_file(".".as_ref(), "test.mapcss").expect("Read rules");
    let styler = Styler::new(rules, &StyleType::MapsMe, Option::from(text_scale));

    let png = drawer.draw_tile(&entities, &tile, &mut pixels, scale, &styler).expect("draw tile");

    {
        let mut file = File::create(filename).expect("open file");
        file.write_all(&png);
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
