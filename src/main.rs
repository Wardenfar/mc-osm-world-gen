use std::io::Cursor;
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::sync::Arc;
use std::{env, thread};

use anvil_region::position::{RegionChunkPosition, RegionPosition};
use anvil_region::provider::{FolderRegionProvider, RegionProvider};
use anvil_region::region::Region;
use feather_blocks::BlockId;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use nbt::CompoundTag;
use threadpool::ThreadPool;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::mpsc::Sender;
use tracing::Level;
use tracing::{span, Instrument};
use tracing_flame::FlameLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::coord::Point;
use crate::parser::{parse_pbf, Store};
use crate::renderer::{render, Pixel, Tile};

mod coord;
mod parser;
mod renderer;

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();
    //setup_global_subscriber();

    let span = span!(Level::TRACE, "program");
    program().instrument(span).await;
}

fn setup_global_subscriber() -> impl Drop {
    let (flame_layer, _guard) = FlameLayer::with_file("./tracing.folded").unwrap();

    let subscriber = Registry::default().with(flame_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");
    _guard
}

async fn program() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2, "args:  <pbf-file>");

    let zoom = 19;

    let store = parse_pbf(&args[1], zoom).expect("read pbf file");

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

    let count_lock = Arc::new(AtomicU64::new(0));

    let mut futures = FuturesUnordered::new();

    println!("start");

    let (tx, mut rx) = tokio::sync::mpsc::channel::<u64>(1024);

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
            let tx = tx.clone();

            let span = span!(Level::TRACE, "one region");
            let fut = region(size, x, y, tile, store, tx);
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

    let counter = AtomicU64::new(0);

    loop {
        tokio::select! {
            Some(val) = rx.recv() => {
                let current = counter.fetch_add(val, Ordering::SeqCst);
                bar.set_position(current);
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

async fn region(size: f64, x: i32, y: i32, tile: Tile, store: Arc<Store>, tx: Sender<u64>) {
    let pixels = render(&store, &tile, size, 3f64).expect("render pixels");
    let span = span!(Level::TRACE, "fill_region", x = x, y = y);
    fill_region(tx, x, y, pixels).instrument(span).await;
}

async fn fill_region(tx: Sender<u64>, region_x: i32, region_y: i32, pixels: Vec<Pixel>) {
    let min_chunk_x = region_x << 5;
    let min_chunk_y = region_y << 5;

    let mut buffer = Cursor::new(Vec::<u8>::new());

    let mut region = Region::load(RegionPosition::new(region_x, region_y), &mut buffer).unwrap();

    for chunk_x in min_chunk_x..(min_chunk_x + 32) {
        for chunk_y in min_chunk_y..(min_chunk_y + 32) {
            let region_chunk_position = RegionChunkPosition::from_chunk_position(chunk_x, chunk_y);

            let mut chunk_compound_tag = CompoundTag::new();
            let mut level_compound_tag = CompoundTag::new();
            level_compound_tag.insert_i32("xPos", chunk_x as i32);
            level_compound_tag.insert_i32("zPos", chunk_y as i32);
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

            let mut indexes = Vec::new();
            for _ in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let img_x = (chunk_x - min_chunk_x) * 16 + x;
                        let img_z = (chunk_y - min_chunk_y) * 16 + z;
                        let color = &pixels[(img_z * 512 + img_x) as usize];
                        let block_index = match color {
                            Pixel(255, 255, 255) => 0u8,
                            Pixel(255, 0, 0) => 1u8,
                            Pixel(0, 255, 0) => 2u8,
                            _ => 3u8,
                        };
                        indexes.push(block_index)
                    }
                }
            }

            let mut states = Vec::new();
            for chunk in indexes.chunks(8) {
                states.push(i64::from_be_bytes([
                    chunk[7], chunk[6], chunk[5], chunk[4], chunk[3], chunk[2], chunk[1], chunk[0],
                ]));
            }

            section.insert_i64_vec("BlockStates", states);

            level_compound_tag.insert_compound_tag_vec("Sections", vec![section]);

            chunk_compound_tag.insert_compound_tag("Level", level_compound_tag);

            {
                let span = span!(Level::TRACE, "write chunk to buffer");
                let _guard = span.enter();
                region
                    .write_chunk(region_chunk_position, chunk_compound_tag)
                    .unwrap();
            }
        }
        tx.send(32).await.unwrap();
    }

    let path = format!("world/region/r.{}.{}.mca", region_x, region_y);
    tokio::fs::write(path, &buffer.into_inner())
        .instrument(span!(Level::TRACE, "write region"))
        .await;
}
