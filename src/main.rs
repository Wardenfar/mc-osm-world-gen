mod parser;
mod coord;
mod renderer;

use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use feather_base::{Biome, BlockPosition, Chunk, ChunkPosition};
use feather_base::anvil::level::{LevelData, SuperflatGeneratorOptions};
use feather_base::anvil::region::{create_region, Error, load_region, RegionHandle, RegionPosition};
use feather_blocks::BlockId;
use feather_common::{Game, World};
use feather_common::world_source::flat::FlatWorldSource;
use feather_common::world_source::region::RegionWorldSource;
use feather_common::world_source::WorldSource;
use feather_worldgen::{SuperflatWorldGenerator, WorldGenerator};
use indicatif::{MultiProgress, ProgressBar, ProgressIterator, ProgressStyle};
use protobuf::{CodedInputStream, Message};
use threadpool::ThreadPool;

// use renderer::coords::Coords;
// use renderer::draw::drawer::{Drawer, TileRenderedPixels};
// use renderer::draw::tile_pixels::TilePixels;
// use renderer::geodata::reader::{GeodataReader, OsmEntities};
// use renderer::mapcss::parser::parse_file;
// use renderer::mapcss::styler::{Styler, StyleType};
// use renderer::tile::{coords_to_max_zoom_tile, coords_to_zoom_tile, MAX_ZOOM, Tile, tile_to_max_zoom_tile_range};

static SCALE: usize = 2;

fn main() {
    // let level = generate_level();
    // let mut file = File::create("world/level.dat").expect("open level.dat");
    // level.save_to_file(&mut file).expect("write to level.dat");

    // match renderer::geodata::importer::import("herblay.osm", "herblay.bin") {
    //     Ok(_) => println!("All good"),
    //     Err(err) => {
    //         for cause in err.chain() {
    //             eprintln!("{}", cause);
    //         }
    //         std::process::exit(1);
    //     }
    // }

    // let text_scale: f64 = 0.0;
    // let rules = parse_file(".".as_ref(), "test.mapcss").expect("Read rules");
    // let styler = Styler::new(rules, &StyleType::MapsMe, Option::from(text_scale));
    // let styler_arc = Arc::new(styler);
    //
    // let storage = GeodataReader::load("herblay.bin").unwrap();
    // let storage_arc = Arc::new(storage);
    //
    // let bbox = storage_arc.boundingbox();
    // let min_corner = (bbox.min_lat, bbox.min_lon);
    // let max_corner = (bbox.max_lat, bbox.max_lon);
    //
    // let zoom = 15;
    //
    // let min_tile = coords_to_zoom_tile(&min_corner, zoom);
    // let max_tile = coords_to_zoom_tile(&max_corner, zoom);
    //
    // let min_x = min(max_tile.x, min_tile.x);
    // let min_y = min(max_tile.y, min_tile.y);
    // let max_x = max(max_tile.x, min_tile.x) + 1;
    // let max_y = max(max_tile.y, min_tile.y) + 1;
    //
    // let range_x = (max_x - min_x);
    // let range_y = (max_y - min_y);
    //
    // let pool = ThreadPool::new(16);
    //
    // let count_lock = Arc::new(AtomicU64::new(0));
    //
    // println!("start");
    //
    // for x in min_x..max_x {
    //     for y in min_y..max_y {
    //         let tile = Tile {
    //             zoom,
    //             x,
    //             y,
    //         };
    //         let drawer = Drawer::new("output".as_ref());
    //
    //         let storage_clone = storage_arc.clone();
    //         let styler_clone = styler_arc.clone();
    //         let count_clone = count_lock.clone();
    //         pool.execute(move || {
    //             let entities = storage_clone.get_entities_in_tile_with_neighbors(&tile, &None);
    //             let pixels = render_tile(entities, &drawer, &tile, &styler_clone);
    //             fill_region(count_clone, x - min_x, y - min_y, pixels);
    //         });
    //     }
    // }
    //
    // let count_clone = count_lock.clone();
    // let t = thread::spawn(move || {
    //     let bar = ProgressBar::new((range_x * range_y * 32 * 32) as u64);
    //     let sty = ProgressStyle::default_bar()
    //         .template("[{elapsed_precise}] ({per_sec}) {bar:40.cyan/blue} {pos:>7}/{len:7} [ETA {eta_precise}]")
    //         .progress_chars("##-");
    //     bar.set_style(sty);
    //     loop {
    //         let current = count_clone.load(Ordering::SeqCst);
    //         bar.set_position(current);
    //         if current >= (32 * 32 * range_x * range_y) as u64 {
    //             break
    //         }
    //     }
    //     bar.finish_with_message("Done");
    // });
    //
    // pool.join();
    // t.join();
}

// fn fill_region(count_lock: Arc<AtomicU64>, region_x: u32, region_y: u32, pixels: TileRenderedPixels) {
//     let min_chunk_x = region_x << 5;
//     let min_chunk_y = region_y << 5;
//
//     let mut region = create_region(
//         Path::new("world"),
//         RegionPosition::from_chunk(
//             ChunkPosition::new(min_chunk_x as i32, min_chunk_y as i32)
//         )).expect("create region");
//
//     for chunk_x in min_chunk_x..(min_chunk_x + 32) {
//         for chunk_y in min_chunk_y..(min_chunk_y + 32) {
//             let mut chunk = Chunk::new(ChunkPosition::new(chunk_x as i32, chunk_y as i32));
//             for x in 0..16 {
//                 for y in 0..16 {
//                     let img_x = (chunk_x - min_chunk_x) * 16 + x;
//                     let img_y = (chunk_y - min_chunk_y) * 16 + y;
//                     let color = pixels.triples[(img_y * 512 + img_x) as usize];
//                     let block = match color {
//                         (255, 255, 255) => BlockId::oak_planks(),
//                         (255, 0, 0) => BlockId::coal_block(),
//                         _ => BlockId::grass_block()
//                     };
//                     chunk.set_block_at(x as usize, 10, y as usize, block);
//                 }
//             }
//             region.save_chunk(&chunk, &Vec::new(), &Vec::new());
//             count_lock.fetch_add(1, Ordering::SeqCst);
//         }
//     }
// }
//
// fn render_tile(entities: OsmEntities, drawer: &Drawer, tile: &Tile, styler: &Styler) -> TileRenderedPixels {
//     // let filename = format!("tiles/tile_{}_{}.png", tile.x, tile.y);
//
//     // if Path::new(&filename).exists() {
//     //     return;
//     // }
//
//     let mut pixels = TilePixels::new(SCALE);
//
//     let pixels = drawer.draw_to_pixels(&entities, &tile, &mut pixels, SCALE, &styler);
//
//     pixels
// }
//
// fn fill_chunk(region: &mut RegionHandle, x: i32, z: i32) {
//     let &mut pos = &mut ChunkPosition::new(x, z);
//
//     let mut chunk = Chunk::new(pos);
//
//     for y in 0..7 {
//         chunk.fill_section(y, BlockId::oak_planks());
//     }
//
//     let entities = Vec::new();
//     let block_entities = Vec::new();
//
//     region.save_chunk(&chunk, &entities, &block_entities).expect("Cant save the chunk");
// }