// #![feature(test)]

use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufReader, Write, Cursor};
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use anvil_region::position::{RegionChunkPosition, RegionPosition};
use anvil_region::provider::{FolderRegionProvider, RegionProvider};
use feather_blocks::BlockId;
use indicatif::{MultiProgress, ProgressBar, ProgressIterator, ProgressStyle};
use nbt::CompoundTag;
use threadpool::ThreadPool;

use crate::coord::Point;
use crate::parser::parse_pbf;
use crate::renderer::{Pixel, render, Tile};
use byteorder::{ReadBytesExt, LittleEndian};
use std::convert::TryInto;

mod parser;
mod coord;
mod renderer;

static SCALE: usize = 2;

fn main() {
    let zoom = 17;

    let store = parse_pbf("herblay.pbf", zoom).expect("read pbf file");
    let store = Arc::new(store);

    let min_point = &store.min_point;
    let max_point = &store.max_point;

    let diff_x = max_point.x - min_point.x;
    let diff_y = max_point.y - min_point.y;

    println!("{:?}", min_point);
    println!("{:?}", max_point);

    let size = 32f64 * 16f64;

    let count_x = (diff_x / size).ceil() as i32;
    let count_y = (diff_y / size).ceil() as i32;

    let pool = ThreadPool::new(16);

    let count_lock = Arc::new(AtomicU64::new(0));

    println!("start");

    for x in 0..count_x {
        for y in 0..count_y {
            let tile = Tile {
                top_left: Point {
                    x: min_point.x + (x as f64) * size,
                    y: min_point.y + (y as f64) * size,
                },
                bottom_right: Point {
                    x: min_point.x + ((x as f64) + 1f64) * size,
                    y: min_point.y + ((y as f64) + 1f64) * size,
                },
            };

            let store = store.clone();
            let count_lock = count_lock.clone();
            pool.execute(move || {
                let pixels = render(&store, &tile, size, 3f64).expect("render pixels");
                fill_region(count_lock, x as u32, y as u32, pixels);
            });
        }
    }

    let count_clone = count_lock.clone();
    let t = thread::spawn(move || {
        let bar = ProgressBar::new((count_x * count_y * 32 * 32) as u64);
        let sty = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] ({per_sec}) {bar:40.cyan/blue} {pos:>7}/{len:7} [ETA {eta_precise}]")
            .progress_chars("##-");
        bar.set_style(sty);
        loop {
            let current = count_clone.load(Ordering::SeqCst);
            bar.set_position(current);
            if current >= (32 * 32 * count_x * count_y) as u64 {
                break;
            }
        }
        bar.finish_with_message("Done");
    });

    pool.join();
    t.join();
}

fn fill_region(count_lock: Arc<AtomicU64>, region_x: u32, region_y: u32, pixels: Vec<Pixel>) {
    let min_chunk_x = region_x << 5;
    let min_chunk_y = region_y << 5;

    let provider = FolderRegionProvider::new("world/region");

    let region_position = RegionPosition::new(region_x as i32, region_y as i32);

    let mut region = provider.get_region(region_position).unwrap();

    for chunk_x in min_chunk_x..(min_chunk_x + 32) {
        for chunk_y in min_chunk_y..(min_chunk_y + 32) {

            let region_chunk_position = RegionChunkPosition::from_chunk_position(chunk_x as i32, chunk_y as i32);

            let mut chunk_compound_tag = CompoundTag::new();
            let mut level_compound_tag = CompoundTag::new();
            level_compound_tag.insert_i32("xPos", chunk_x as i32);
            level_compound_tag.insert_i32("zPos", chunk_y as i32);
            level_compound_tag.insert_i64("LastUpdate", 0);
            level_compound_tag.insert_str("Status", "full");

            let mut section = CompoundTag::new();
            section.insert_i8_vec("BlockLight", vec![0i8; 2048]);
            section.insert_i8("Y", 10);

            let mut all_blocks: Vec<BlockId> = vec![
                BlockId::coal_block(),
                BlockId::oak_planks(),
                BlockId::cobblestone(),
                BlockId::grass_block(),

                BlockId::coal_block(),
                BlockId::oak_planks(),
                BlockId::cobblestone(),
                BlockId::grass_block(),

                BlockId::coal_block(),
                BlockId::oak_planks(),
                BlockId::cobblestone(),
                BlockId::grass_block(),

                BlockId::coal_block(),
                BlockId::oak_planks(),
                BlockId::cobblestone(),
                BlockId::grass_block(),
            ];
            // all_blocks.extend(vec![BlockId::air();2044]);

            let blocks : Vec<CompoundTag> = all_blocks.iter().map(|b| {
                let mut block = CompoundTag::new();
                block.insert_str("Name", b.kind().name());
                block.insert_compound_tag_vec("Properties", vec![]);
                block
            }).collect();

            section.insert_compound_tag_vec("Palette", blocks);

            let mut indexes = Vec::new();
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let img_x = (chunk_x - min_chunk_x) * 16 + x;
                        let img_z = (chunk_y - min_chunk_y) * 16 + z;
                        let color = &pixels[(img_z * 512 + img_x) as usize];
                        let block_index = match color {
                            Pixel(255, 255, 255) => 0u8,
                            Pixel(255, 0, 0) => 1u8,
                            Pixel(0, 255, 0) => 2u8,
                            _ => 3u8
                        };
                        indexes.push(block_index)
                    }
                }
            }

            let mut states = Vec::new();
            for chunk in indexes.chunks(8) {
                states.push(i64::from_be_bytes([
                    chunk[7],
                    chunk[6],
                    chunk[5],
                    chunk[4],
                    chunk[3],
                    chunk[2],
                    chunk[1],
                    chunk[0]
                ].try_into().unwrap()));
            }

            section.insert_i64_vec("BlockStates", states);

            level_compound_tag.insert_compound_tag_vec("Sections", vec![section]);

            chunk_compound_tag.insert_compound_tag("Level", level_compound_tag);

            region.write_chunk(region_chunk_position, chunk_compound_tag);

            count_lock.fetch_add(1, Ordering::SeqCst);
            // println!("{}", chunk_x * 32 + chunk_y);
        }
    }
}
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