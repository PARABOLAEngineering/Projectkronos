use std::fs::File;
use std::io::{Read, Write};
use swisseph_sys::*;
use std::time::Instant;

const EPOCH: f64 = 2453307.0;
const AURORA_LAT: f64 = 39.7294319;  // Aurora, CO coordinates
const AURORA_LON: f64 = -104.8319195;
const HOUSE_SYSTEMS: [char; 8] = ['P', 'K', 'O', 'R', 'C', 'E', 'V', 'W'];

fn pack_coordinates(lat: f64, lon: f64) -> f64 {
    let lat_packed = (lat.abs() * 10000.0) as u64;
    let lon_packed = (lon.abs() * 10000.0) as u64;
    let packed = (lat_packed * 10000000) + lon_packed;
    let signs = ((if lat < 0.0 { 1 } else { 0 } << 1) | 
                 if lon < 0.0 { 1 } else { 0 }) as u64;
    f64::from_bits(packed << 2 | signs)
}

struct ZenithKernel {
    precision: u8,
    timestamp: f64,
    base_positions: [f64; 18],    // Tropical positions
    sidereal_positions: [f64; 18], // Sidereal positions
    location: f64,                // Packed lat/lon
    house_offsets: [f64; 8],      // House system patterns
}

impl ZenithKernel {
    fn new(start_jd: f64) -> Result<Self, Box<dyn std::error::Error>> {
        let mut xx = [0.0; 6];
        let mut serr = [0i8; 256];
        let mut base_positions = [0.0; 18];
        let mut sidereal_positions = [0.0; 18];
        
        println!("Calculating positions for JD {}", start_jd);

        unsafe {
            swe_set_ephe_path("./ephe\0".as_ptr() as *const i8);
            swe_set_jpl_file("de441.eph\0".as_ptr() as *const i8);

            let bodies = [SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS,
                         SE_JUPITER, SE_SATURN, SE_URANUS, SE_NEPTUNE, SE_PLUTO,
                         SE_CHIRON, SE_TRUE_NODE, SE_MEAN_APOG, SE_VESTA, 
                         SE_JUNO, SE_CERES, SE_PALLAS, SE_ASC, SE_ARMC];

            // Calculate tropical positions
            for (i, &body) in bodies.iter().enumerate() {
                let ret = swe_calc_ut(
                    start_jd,
                    body as i32,
                    (SEFLG_SPEED | SEFLG_JPLEPH) as i32,
                    xx.as_mut_ptr(),
                    serr.as_mut_ptr()
                );

                if ret >= 0 {
                    let pos = xx[0].rem_euclid(360.0);
                    println!("Body {} tropical: {:.6}°", i, pos);
                    base_positions[i] = pos;
                }
            }

            // Calculate sidereal positions (using Lahiri ayanamsha)
            swe_set_sid_mode(SE_SIDM_LAHIRI as i32, 0.0, 0.0);
            
            for (i, &body) in bodies.iter().enumerate() {
                let ret = swe_calc_ut(
                    start_jd,
                    body as i32,
                    (SEFLG_SPEED | SEFLG_JPLEPH | SEFLG_SIDEREAL) as i32,
                    xx.as_mut_ptr(),
                    serr.as_mut_ptr()
                );

                if ret >= 0 {
                    let pos = xx[0].rem_euclid(360.0);
                    println!("Body {} sidereal: {:.6}°", i, pos);
                    sidereal_positions[i] = pos;
                }
            }
        }

        // Calculate house offsets for Aurora
        let mut house_offsets = [0.0; 8];
        let mut cusps = [0.0; 13];
        let mut ascmc = [0.0; 10];

        for (i, &system) in HOUSE_SYSTEMS.iter().enumerate() {
            unsafe {
                let ret = swe_houses(
                    start_jd,
                    AURORA_LAT,
                    AURORA_LON,
                    system as i32,
                    cusps.as_mut_ptr(),
                    ascmc.as_mut_ptr()
                );

                if ret >= 0 {
                    let armc = ascmc[2];
                    let mut pattern = 0.0;
                    
                    // Pack offsets into pattern
                    for h in 0..12 {
                        let offset = (cusps[h + 1] - armc).rem_euclid(360.0);
                        pattern += offset / (h + 1) as f64;
                    }
                    
                    house_offsets[i] = pattern;
                    println!("House system {}: pattern = {:.6}", system, pattern);
                }
            }
        }

        Ok(Self {
            precision: 1,  // Second-level precision
            timestamp: start_jd,
            base_positions,
            sidereal_positions,
            location: pack_coordinates(AURORA_LAT, AURORA_LON),
            house_offsets,
        })
    }

    fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create("zenith.kernel")?;
        
        file.write_all(&[self.precision])?;
        file.write_all(&self.timestamp.to_le_bytes())?;
        
        // Write tropical positions
        for pos in &self.base_positions {
            file.write_all(&pos.to_le_bytes())?;
        }
        
        // Write sidereal positions
        for pos in &self.sidereal_positions {
            file.write_all(&pos.to_le_bytes())?;
        }
        
        // Write location and house patterns
        file.write_all(&self.location.to_le_bytes())?;
        for offset in &self.house_offsets {
            file.write_all(&offset.to_le_bytes())?;
        }

        println!("\n✨ Kernel written");
        println!("Size: {} bytes", std::fs::metadata("zenith.kernel")?.len());
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    let args: Vec<String> = std::env::args().collect();
    let start_jd = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(EPOCH);

    println!("🚀 Zenith Engine Starting");
    println!("Processing JD {} for Aurora, CO", start_jd);
    
    let kernel = ZenithKernel::new(start_jd)?;
    kernel.write()?;

    println!("\n✨ Completed in {:?}", start_time.elapsed());
    Ok(())
}