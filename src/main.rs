use std::io::Cursor;
use std::sync::Arc;

use anvil_region::position::{RegionChunkPosition, RegionPosition};
use anvil_region::region::Region;
use feather_blocks::BlockId;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use nbt::CompoundTag;
use std::path::PathBuf;
use tiny_skia::Pixmap;
use tokio::sync::mpsc::Sender;
use tracing::Level;
use tracing::{span, Instrument};
use tracing_flame::FlameLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::coord::Point;
use crate::parser::{parse_pbf, Store};
use crate::renderer::{render, Tile};

mod coord;
mod parser;
mod renderer;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    pbf: PathBuf,
    #[clap(short, long, default_value_t = 17)]
    zoom: usize,
}

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();
    //setup_global_subscriber();

    let span = span!(Level::TRACE, "program");
    program().instrument(span).await;
}

pub const REGION_BLOCK_SIZE: u32 = 32 * 16;
pub const REGION_BLOCK_SIZE_F64: f64 = REGION_BLOCK_SIZE as f64;

#[allow(unused)]
fn setup_global_subscriber() -> impl Drop {
    let (flame_layer, _guard) = FlameLayer::with_file("./tracing.folded").unwrap();

    let subscriber = Registry::default().with(flame_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");
    _guard
}

async fn program() {
    let args = Args::parse();

    let zoom = args.zoom;

    let store = parse_pbf(args.pbf, zoom).expect("read pbf file");

    let store = Arc::new(store);

    let min_point = &store.min_point;
    let max_point = &store.max_point;

    let diff_x = max_point.x - min_point.x;
    let diff_y = max_point.y - min_point.y;

    println!("{:?}", min_point);
    println!("{:?}", max_point);

    let count_x = (diff_x / REGION_BLOCK_SIZE_F64).ceil() as i32;
    let count_y = (diff_y / REGION_BLOCK_SIZE_F64).ceil() as i32;

    let mut futures = FuturesUnordered::new();

    println!("start");

    let (tx, mut rx) = tokio::sync::mpsc::channel::<u64>(1024);

    for x in 0..count_x {
        for y in 0..count_y {
            let tile = Tile {
                top_left: Point {
                    x: min_point.x + (x as f64) * REGION_BLOCK_SIZE_F64,
                    y: min_point.y + (y as f64) * REGION_BLOCK_SIZE_F64,
                },
                bottom_right: Point {
                    x: min_point.x + ((x as f64) + 1f64) * REGION_BLOCK_SIZE_F64,
                    y: min_point.y + ((y as f64) + 1f64) * REGION_BLOCK_SIZE_F64,
                },
            };

            let store = store.clone();
            let tx = tx.clone();

            let span = span!(Level::TRACE, "one region");
            let fut = region(x, y, tile, store, tx);
            let handle = tokio::spawn(fut.instrument(span));
            futures.push(handle);
        }
    }

    let bar = ProgressBar::new((count_x * count_y * 32 * 32) as u64);
    let sty = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] ({per_sec}) {bar:40.cyan/blue} {pos:>7}/{len:7} [ETA {eta_precise}]")
        .progress_chars("##-");
    bar.set_style(sty);
    bar.set_position(0);

    let mut counter = 0;

    loop {
        tokio::select! {
            Some(val) = rx.recv() => {
                counter += val;
                bar.set_position(counter);
            }
            _ = futures.next() => {
                if futures.is_empty() {
                    bar.finish_with_message("Done");
                    break
                }
            }
        };
    }
}

async fn region(x: i32, y: i32, tile: Tile, store: Arc<Store>, tx: Sender<u64>) {
    let pixels = render(&store, &tile, 3f32);
    pixels.save_png(format!("{}_{}.png", x, y)).unwrap();
    let span = span!(Level::TRACE, "fill_region", x = x, y = y);
    fill_region(tx, x, y, pixels).instrument(span).await;
}

async fn fill_region(tx: Sender<u64>, region_x: i32, region_y: i32, pixels: Pixmap) {
    let min_chunk_x = region_x << 5;
    let min_chunk_y = region_y << 5;

    let mut buffer = Cursor::new(Vec::<u8>::with_capacity(4_500_000));

    let mut region = Region::load(RegionPosition::new(region_x, region_y), &mut buffer).unwrap();

    for chunk_x in min_chunk_x..(min_chunk_x + 32) {
        for chunk_y in min_chunk_y..(min_chunk_y + 32) {
            let region_chunk_position = RegionChunkPosition::from_chunk_position(chunk_x, chunk_y);

            let mut chunk_compound_tag = CompoundTag::new();
            chunk_compound_tag.insert_i32("DataVersion", 16);
            let mut level_compound_tag = CompoundTag::new();
            level_compound_tag.insert_i32("xPos", chunk_x);
            level_compound_tag.insert_i32("zPos", chunk_y);
            level_compound_tag.insert_i64("LastUpdate", 0);
            level_compound_tag.insert_str("Status", "full");

            let mut section = CompoundTag::new();
            section.insert_i8_vec("BlockLight", vec![0i8; 2048]);
            section.insert_i8("Y", 0);

            let all_blocks: Vec<BlockId> = vec![
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

            let blocks: Vec<CompoundTag> = all_blocks
                .iter()
                .map(|b| {
                    let mut block = CompoundTag::new();
                    block.insert_str("Name", b.kind().name());
                    block.insert_compound_tag_vec("Properties", vec![]);
                    block
                })
                .collect();

            section.insert_compound_tag_vec("Palette", blocks);

            let mut indexes = Vec::with_capacity(16 * 16 * 16);
            for _ in 0..16 {
                for y in 0..16_u32 {
                    for x in 0..16_u32 {
                        let img_x = (chunk_x - min_chunk_x) as u32 * 16 + x;
                        let img_y = (chunk_y - min_chunk_y) as u32 * 16 + y;
                        let color = pixels.pixel(img_x, img_y).unwrap();
                        let block_index = match (color.red(), color.green(), color.blue()) {
                            (255, 255, 255) => 0u8,
                            (255, 0, 0) => 1u8,
                            (0, 255, 0) => 2u8,
                            (0, 0, 0) => 3u8,
                            _ => panic!("invalid color"),
                        };
                        indexes.push(block_index)
                    }
                }
            }

            let mut states = Vec::with_capacity(indexes.len() / 8);
            for chunk in indexes.chunks(8) {
                states.push(i64::from_be_bytes([
                    chunk[7], chunk[6], chunk[5], chunk[4], chunk[3], chunk[2], chunk[1], chunk[0],
                ]));
            }

            section.insert_i64_vec("BlockStates", states);

            level_compound_tag.insert_compound_tag_vec("Sections", vec![section]);

            chunk_compound_tag.insert_compound_tag("Level", level_compound_tag);

            region
                .write_chunk(region_chunk_position, chunk_compound_tag)
                .unwrap();
        }
        tx.send(32).await.unwrap();
    }

    let path = format!("world/region/r.{}.{}.mca", region_x, region_y);
    tokio::fs::write(path, &buffer.into_inner()).await.unwrap();
}
