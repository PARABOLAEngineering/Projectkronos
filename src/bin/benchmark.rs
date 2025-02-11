use std::fs::File;
use std::time::Instant;
use memmap2::MmapOptions;
use swisseph_sys::*;

const ITERATIONS: u32 = 1_000_000;  // A million positions!
const START_JD: f64 = 2451545.0;    // J2000
const TIME_STEP: f64 = 1.0 / 86400.0; // One second

#[derive(Debug)]
struct FullChart {
    positions: Vec<f64>,
    houses: Vec<f64>,
}

struct KernelReader {
    zenith_map: memmap2::Mmap,
    house_map: memmap2::Mmap,
}

impl KernelReader {
    fn new() -> Result<Self, std::io::Error> {
        let zenith_file = File::open("zenith.kernel")?;
        let house_file = File::open("houses.kernel")?;
        
        let zenith_map = unsafe { MmapOptions::new().map(&zenith_file)? };
        let house_map = unsafe { MmapOptions::new().map(&house_file)? };

        Ok(Self { 
            zenith_map,
            house_map,
        })
    }

    fn read_chart(&self) -> FullChart {
        let mut positions = Vec::with_capacity(18);
        let mut houses = Vec::with_capacity(60);  // 12 houses × 5 systems

        // Read positions (skip precision byte and timestamp)
        let mut offset = 9;
        for _ in 0..18 {
            let pos_bytes = &self.zenith_map[offset..offset + 8];
            let pos = f64::from_le_bytes(pos_bytes.try_into().unwrap());
            positions.push(pos);
            offset += 8;
        }

        // Read houses (skip location)
        offset = 16;
        for _ in 0..60 {
            let house_bytes = &self.house_map[offset..offset + 8];
            let house = f64::from_le_bytes(house_bytes.try_into().unwrap());
            houses.push(house);
            offset += 8;
        }

        FullChart { positions, houses }
    }
}

fn calculate_with_swisseph(jd: f64) -> FullChart {
    let mut positions = Vec::with_capacity(18);
    let mut houses = Vec::with_capacity(60);
    
    unsafe {
        let mut xx = [0.0; 6];
        let mut serr = [0i8; 256];
        let mut cusps = [0.0; 13];
        let mut ascmc = [0.0; 10];
        
        let bodies = [SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS,
                     SE_JUPITER, SE_SATURN, SE_URANUS, SE_NEPTUNE, SE_PLUTO,
                     SE_CHIRON, SE_TRUE_NODE, SE_MEAN_APOG, SE_VESTA, 
                     SE_JUNO, SE_CERES, SE_PALLAS, SE_ASC, SE_ARMC];

        // Calculate positions
        for &body in &bodies {
            swe_calc_ut(
                jd,
                body as i32,
                (SEFLG_SPEED | SEFLG_JPLEPH) as i32,
                xx.as_mut_ptr(),
                serr.as_mut_ptr()
            );
            positions.push(xx[0].rem_euclid(360.0));
        }

        // Calculate houses for each system
        let systems = ['P', 'K', 'E', 'W', 'R'];
        for system in systems.iter() {
            swe_houses(
                jd,
                39.7294319,  // Aurora
                -104.8319195,
                *system as i32,
                cusps.as_mut_ptr(),
                ascmc.as_mut_ptr()
            );
            for i in 1..13 {
                houses.push(cusps[i]);
            }
        }
    }

    FullChart { positions, houses }
}

fn main() {
    unsafe {
        swe_set_ephe_path("./ephe\0".as_ptr() as *const i8);
        swe_set_jpl_file("de441.eph\0".as_ptr() as *const i8);
    }

    println!("🚀 Starting sequential position benchmark");
    println!("Running {} iterations", ITERATIONS);
    println!("Simulating planet winding at 1-second intervals\n");

    // Initialize memory mapped reader
    let kernel = match KernelReader::new() {
        Ok(k) => k,
        Err(e) => {
            println!("✗ Error memory mapping kernels: {}", e);
            return;
        }
    };

    // Verify reading
    let data = kernel.read_chart();
    println!("✓ Memory mapping successful");
    println!("  Read {} positions", data.positions.len());
    println!("  Read {} house positions", data.houses.len());
    println!("  First position: {:.6}°\n", data.positions[0]);

    // Warmup
    kernel.read_chart();
    calculate_with_swisseph(START_JD);

    // Benchmark memory mapped sequential reading
    let kernel_start = Instant::now();
    for i in 0..ITERATIONS {
        let _ = kernel.read_chart();
    }
    let kernel_time = kernel_start.elapsed();

    // Benchmark Swiss Ephemeris sequential calculation
    let swisseph_start = Instant::now();
    for i in 0..ITERATIONS {
        let jd = START_JD + (i as f64 * TIME_STEP);
        let _ = calculate_with_swisseph(jd);
    }
    let swisseph_time = swisseph_start.elapsed();

    // Print results
    println!("Memory Mapped Sequential Reading:");
    println!("  Total time: {:?}", kernel_time);
    println!("  Average time: {:?}", kernel_time / ITERATIONS);
    println!("  Positions per second: {:.2}", 
             ITERATIONS as f64 / kernel_time.as_secs_f64());

    println!("\nSwiss Ephemeris Sequential Calculation:");
    println!("  Total time: {:?}", swisseph_time);
    println!("  Average time: {:?}", swisseph_time / ITERATIONS);
    println!("  Positions per second: {:.2}", 
             ITERATIONS as f64 / swisseph_time.as_secs_f64());

    println!("\nSpeed difference: {:.2}x", 
             swisseph_time.as_nanos() as f64 / kernel_time.as_nanos() as f64);
}