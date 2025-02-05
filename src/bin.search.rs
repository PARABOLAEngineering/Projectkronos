// src/bin/search.rs
use std::fs::File;
use std::io::Read;
use swisseph_sys::*;
use medusa::SE_AST_OFFSET;

const BODIES: [&str; 20] = [
    "Sun", "Moon", "Mercury", "Venus", "Mars",
    "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto",
    "Chiron", "True Node", "Mean Apogee",
    "Vesta", "Juno", "Ceres", "Pallas", "Asc", "Armc", "15550"
];

fn decimal_to_dms(decimal_degrees: f64) -> (i32, i32, f64) {
    let total_seconds = (decimal_degrees * 3600.0).round() as i32;
    let degrees = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = (total_seconds % 3600) % 60;
    (degrees, minutes, seconds as f64)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <julian_date>", args[0]);
        return Ok(());
    }

    let search_jd = args[1].parse::<f64>()?;
    
    println!("🔍 Searching positions for JD {}", search_jd);

    // Read kernel
    let mut file = File::open("zenith.kernel")?;
    let mut timestamp_bytes = [0u8; 8];
    file.read_exact(&mut timestamp_bytes)?;
    let timestamp = f64::from_le_bytes(timestamp_bytes);

    let mut base_positions = Vec::with_capacity(20);
    for _ in 0..20 {
        let mut pos_bytes = [0u8; 8];
        file.read_exact(&mut pos_bytes)?;
        base_positions.push(f64::from_le_bytes(pos_bytes));
    }

    // Calculate current positions
    let mut xx = [0.0; 6];
    let mut serr = [0i8; 256];
    let bodies = [
     SE_SUN, SE_MOON, 
       SE_MERCURY, SE_VENUS, 
    SE_MARS, SE_JUPITER, 
        SE_SATURN, SE_URANUS, 
      SE_NEPTUNE, SE_PLUTO,
    SE_CHIRON, SE_TRUE_NODE,
      SE_MEAN_APOG, SE_VESTA,
        SE_JUNO, SE_CERES,
        SE_PALLAS, SE_ASC, SE_ARMC
    ];

    println!("\nCelestial Positions:");
    println!("═══════════════════════════════════════");

    for (i, &body) in bodies.iter().enumerate() {
        unsafe {
            let ret = swisseph_sys::swe_calc_ut(
                search_jd,
                body as i32,
                (swisseph_sys::SEFLG_SPEED | swisseph_sys::SEFLG_SWIEPH) as i32,
                xx.as_mut_ptr(),
                serr.as_mut_ptr()
            );

            if ret >= 0 {
                let position = xx[0].rem_euclid(360.0);
                let speed = xx[3];
                let (deg, min, sec) = decimal_to_dms(position);
                
                println!("{:12} │ {}°{}'{:.0}\" {} {:.6}°/day", 
                    BODIES[i],
                    deg,
                    min,
                    sec,
                    if speed < 0.0 { "☌" } else { " " },
                    speed.abs()
                );
            }
        }
    }

    Ok(())
}