use std::{
    env, fs, process,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use fastanvil::{complete, Block, ChunkData, RCoord, RegionFileLoader, RegionLoader};
use rayon::prelude::*;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let path = match args.get(1) {
        Some(x) => x,
        None => {
            println!("Specify a path");
            process::exit(1);
        }
    };

    let loader = RegionFileLoader::new(path.into());
    let regions: Vec<(isize, isize)> = fs::read_dir(path)?
        .filter_map(|x| x.ok())
        .map(|x| {
            let binding = x.file_name();
            let filename = binding.to_string_lossy();
            let splits: Vec<&str> = filename.split('.').collect();
            let x: isize = splits[1].parse().unwrap();
            let z: isize = splits[2].parse().unwrap();
            (x, z)
        })
        .collect();

    let region_count = regions.len();
    let completed_region = AtomicUsize::new(0);

    regions.par_iter().for_each(|(region_x, region_y)| {
        match loader.region(RCoord(*region_x), RCoord(*region_y)) {
            Ok(region) => {
                region
                    .expect("region not existing")
                    .iter()
                    .filter_map(|x| x.ok())
                    .collect::<Vec<ChunkData>>()
                    .into_par_iter()
                    .for_each(|chunk| {
                        let chunk_x = chunk.x;
                        let chunk_z = chunk.z;

                        let chunk = complete::Chunk::from_bytes(&chunk.data);
                        match chunk {
                            Ok(chunk) => {
                                chunk
                                    .iter_blocks()
                                    .collect::<Vec<&Block>>()
                                    .into_par_iter()
                                    .for_each(|block| {
                                        if block.name() == "minecraft:dragon_egg" {
                                            println!("YAY! Dragon egg is in chunk {chunk_x}.{chunk_z} in r.{region_x}.{region_y}");
                                        }
                                    });
                            },
                            Err(e) => {
                                println!("Error reading chunk {chunk_x}.{chunk_z} in r.{region_x}.{region_y}: {e}")
                            }
                        }
                    });

                let completed_count = completed_region.fetch_add(1, Ordering::Relaxed);
                println!("Finished region r.{region_x}.{region_y}: {completed_count}/{region_count}")
            }
            Err(e) => {
                println!("Error loading region r.{region_x}.{region_y}: {e}")
            }
        }
    });

    Ok(())
}
