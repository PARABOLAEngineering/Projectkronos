use std::fs::File;
use std::io::{Write};
use swisseph_sys::*;
use std::time::Instant;

const AURORA_LAT: f64 = 39.7294319;
const AURORA_LON: f64 = -104.8319195;

// The main house systems we care about
const HOUSE_SYSTEMS: [(char, &str); 5] = [
    ('P', "Placidus"),
    ('K', "Koch"),
    ('E', "Equal"),
    ('W', "Whole Sign"),
    ('R', "Regiomontanus"),
];

struct HouseKernel {
    location: (f64, f64),             // lat, lon
    house_positions: [[f64; 12]; 5],  // 5 systems × 12 houses
}

impl HouseKernel {
    fn new(lat: f64, lon: f64) -> Result<Self, Box<dyn std::error::Error>> {
        let mut house_positions = [[0.0; 12]; 5];
        let mut cusps = [0.0; 13];
        let mut ascmc = [0.0; 10];

        println!("Calculating house positions for {:.4}°N, {:.4}°W", lat, lon.abs());

        unsafe {
            swe_set_ephe_path("./ephe\0".as_ptr() as *const i8);

            // Calculate houses for each system
            for (i, (system, name)) in HOUSE_SYSTEMS.iter().enumerate() {
                println!("\nCalculating {} houses:", name);
                
                let ret = swe_houses(
                    2451545.0,  // J2000 as reference
                    lat, lon,
                    *system as i32,
                    cusps.as_mut_ptr(),
                    ascmc.as_mut_ptr()
                );

                if ret >= 0 {
                    // Store house positions
                    for h in 0..12 {
                        house_positions[i][h] = cusps[h + 1];
                        println!("House {}: {:.6}°", h + 1, cusps[h + 1]);
                    }
                } else {
                    println!("Failed to calculate {} houses", name);
                }
            }
        }

        Ok(Self {
            location: (lat, lon),
            house_positions,
        })
    }

    fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create("houses.kernel")?;
        
        // Write location
        file.write_all(&self.location.0.to_le_bytes())?;
        file.write_all(&self.location.1.to_le_bytes())?;
        
        // Write house positions
        for system in &self.house_positions {
            for house in system {
                file.write_all(&house.to_le_bytes())?;
            }
        }

        println!("\n✨ House kernel written");
        println!("Size: {} bytes", std::fs::metadata("houses.kernel")?.len());
        Ok(())
    }

    fn verify(&self) {
        println!("\nVerification Report:");
        println!("Location: {:.4}°N, {:.4}°W", 
                self.location.0, 
                self.location.1.abs());

        for (i, (_, name)) in HOUSE_SYSTEMS.iter().enumerate() {
            println!("\n{} Houses:", name);
            println!("╭────────┬───────────────╮");
            println!("│ House  │   Position    │");
            println!("├────────┼───────────────┤");
            
            for (h, pos) in self.house_positions[i].iter().enumerate() {
                let next_pos = if h == 11 {
                    self.house_positions[i][0]
                } else {
                    self.house_positions[i][h + 1]
                };
                
                let span = (next_pos - pos).rem_euclid(360.0);
                
                println!("│   {}    │  {:.6}°   │ (span: {:.6}°)", 
                        h + 1, pos, span);
            }
            println!("╰────────┴───────────────╯");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    println!("🏠 House Kernel Generator Starting");
    
    let kernel = HouseKernel::new(AURORA_LAT, AURORA_LON)?;
    kernel.write()?;
    kernel.verify();

    println!("\n✨ Completed in {:?}", start_time.elapsed());
    Ok(())
}