use std::fs::File;
use std::io::{Read, Write};
use swisseph_sys::*;
use std::time::Instant;

const EPOCH: f64 = 2453307.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    let args: Vec<String> = std::env::args().collect();
    let start_jd: f64 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(EPOCH);

    let mut xx = [0.0; 6];
    let mut serr = [0i8; 256];
    let mut base_positions = [0.0; 20];

    println!("🚀 Zenith Engine Starting");
    println!("Processing JD {}", start_jd);

    // Set up DE441
    unsafe {
        swe_set_ephe_path("./ephe\0".as_ptr() as *const i8);
        swe_set_jpl_file("de441.eph\0".as_ptr() as *const i8);
    }

    let bodies = [SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS,
                 SE_JUPITER, SE_SATURN, SE_URANUS, SE_NEPTUNE, SE_PLUTO,
                 SE_CHIRON, SE_TRUE_NODE, SE_MEAN_APOG, SE_VESTA, 
                 SE_JUNO, SE_CERES, SE_PALLAS, SE_ASC, SE_ARMC, 
                 (SE_AST_OFFSET + 5550)];

    for (i, &body) in bodies.iter().enumerate() {
        unsafe {
            let ret = swe_calc_ut(start_jd, body as i32,
                (SEFLG_SPEED | SEFLG_JPLEPH) as i32,  // Using JPL mode
                xx.as_mut_ptr(), serr.as_mut_ptr());

            if ret >= 0 {
                let pos = xx[0].rem_euclid(360.0);
                println!("Planet {}: {:.6}°", i, pos);
                base_positions[i] = pos;
            } else {
                println!("Failed to calculate position for body {}", i);
            }
        }
    }

    let mut file = File::create("zenith.kernel")?;
    file.write_all(&(start_jd as f64).to_le_bytes())?;
    for pos in &base_positions {
        file.write_all(&(*pos as f64).to_le_bytes())?;
    }

    println!("\n✨ Completed in {:?}", start_time.elapsed());
    println!("Size: {} bytes", std::fs::metadata("zenith.kernel")?.len());
    Ok(())
}