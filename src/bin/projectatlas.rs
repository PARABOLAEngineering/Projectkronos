use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use swisseph_sys::*;
use rayon::prelude::*;

const HOUSE_SYSTEMS: [char; 8] = ['P', 'K', 'O', 'R', 'C', 'E', 'V', 'W'];

#[derive(Debug, Clone, PartialEq)]
struct Location {
    packed_coords: u64,
    flags: u8,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct HousePattern {
    offsets: [u16; 12],  // Centidegree offsets from ARMC
}

struct VestaGenerator {
    locations: Vec<Location>,
    patterns: HashMap<char, Vec<(Location, HousePattern)>>,  // House system char -> pattern list
}

impl VestaGenerator {
    fn new() -> Self {
        Self {
            locations: Vec::new(),
            patterns: HashMap::new(),
        }
    }

    fn pack_coordinates(lat: f64, lon: f64) -> Location {
        let lat_packed = (lat.abs() * 10000.0) as u64;
        let lon_packed = (lon.abs() * 10000.0) as u64;
        
        Location {
            packed_coords: (lat_packed * 10000000) + lon_packed,
            flags: ((if lat < 0.0 { 1 } else { 0 } << 1) | 
                    (if lon < 0.0 { 1 } else { 0 })) as u8,
            lat,
            lon,
        }
    }

    fn load_cities(&mut self, path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split('\t').collect();
            
            if fields.len() >= 5 {
                if let (Ok(lat), Ok(lon)) = (
                    fields[4].parse::<f64>(),
                    fields[5].parse::<f64>(),
                ) {
                    self.locations.push(Self::pack_coordinates(lat, lon));
                }
            }
        }

        // Sort by packed coordinates for binary search later
        self.locations.sort_by_key(|loc| loc.packed_coords);
        Ok(())
    }

    fn calculate_house_pattern(&self, location: &Location, system: char) -> HousePattern {
        let mut cusps = [0.0; 13];
        let mut ascmc = [0.0; 10];
        let armc = unsafe {
            swe_houses(
                2451545.0, // J2000 as reference
                location.lat,
                location.lon,
                system as i32,
                cusps.as_mut_ptr(),
                ascmc.as_mut_ptr()
            )
        };

        // Convert to offsets from ARMC
        let mut offsets = [0u16; 12];
        for i in 0..12 {
            let diff = (cusps[i + 1] - ascmc[2]).rem_euclid(360.0);
            offsets[i] = (diff * 100.0) as u16; // Store as centidegrees
        }

        HousePattern { offsets }
    }

    fn generate_patterns(&mut self) {
        for &system in HOUSE_SYSTEMS.iter() {
            let patterns: Vec<(Location, HousePattern)> = self.locations
                .par_iter()  // Process locations in parallel
                .map(|loc| {
                    let pattern = self.calculate_house_pattern(loc, system);
                    (loc.clone(), pattern)
                })
                .collect();

            self.patterns.insert(system, patterns);
        }
    }

    fn deduplicate_patterns(&mut self) {
        for (system, patterns) in self.patterns.iter_mut() {
            let mut unique_patterns: HashMap<[u16; 12], HousePattern> = HashMap::new();
            let mut deduplicated = Vec::new();

            for (loc, pattern) in patterns.drain(..) {
                let similar_pattern = unique_patterns.iter().find(|(_, p)| {
                    pattern.offsets.iter().zip(p.offsets.iter())
                        .all(|(a, b)| a.max(b) - a.min(b) <= 1) // 0.01° tolerance
                });

                if let Some((_, existing)) = similar_pattern {
                    deduplicated.push((loc, existing.clone()));
                } else {
                    unique_patterns.insert(pattern.offsets, pattern.clone());
                    deduplicated.push((loc, pattern));
                }
            }

            *patterns = deduplicated;
            println!("System {}: {} unique patterns", system, unique_patterns.len());
        }
    }

    fn write_kernel(&self, path: &str) -> io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write magic number and version
        writer.write_all(b"VESTA\x01")?;

        // Write number of locations
        writer.write_all(&(self.locations.len() as u32).to_le_bytes())?;

        // Write location index
        for loc in &self.locations {
            writer.write_all(&loc.packed_coords.to_le_bytes())?;
            writer.write_all(&[loc.flags])?;
        }

        // Write patterns for each house system
        for &system in HOUSE_SYSTEMS.iter() {
            if let Some(patterns) = self.patterns.get(&system) {
                // Write number of patterns
                writer.write_all(&(patterns.len() as u32).to_le_bytes())?;

                // Write patterns
                for (loc, pattern) in patterns {
                    writer.write_all(&loc.packed_coords.to_le_bytes())?;
                    for &offset in &pattern.offsets {
                        writer.write_all(&offset.to_le_bytes())?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut generator = VestaGenerator::new();
    
    println!("Loading cities...");
    generator.load_cities("/home/kelsey/development/projectkronosfinal/cities500.txt")?;
    println!("Loaded {} locations", generator.locations.len());

    println!("Calculating house patterns...");
    generator.generate_patterns();

    println!("Deduplicating patterns...");
    generator.deduplicate_patterns();

    println!("Writing vesta kernel...");
    generator.write_kernel("vesta.kernel")?;
    
    println!("Done!");
    Ok(())
}