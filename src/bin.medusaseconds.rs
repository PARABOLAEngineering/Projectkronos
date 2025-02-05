use std::fs::File;
use std::io::{Read, Write};
use medusa::SE_AST_OFFSET;
use swisseph_sys::*;
use std::time::Instant;

const BASE_DATE: f64 = 625615.0;
const JD_SECOND: f64 = 1.0 / 86400.0;

struct ZenithKernel {
    timestamp: f64,
    base_positions: [f64; 20],
    time_delta: f64,
    precision: Precision,
}

#[derive(Clone, Copy, Debug)]
enum Precision {
    Second,
    Minute,
}

impl Precision {
    fn to_jd(&self) -> f64 {
        match self {
            Precision::Second => JD_SECOND,
            Precision::Minute => JD_SECOND * 60.0,
        }
    }
}

impl ZenithKernel {
    fn new(start_jd: f64, end_jd: f64, precision: Precision) -> Result<Self, Box<dyn std::error::Error>> {
        let mut xx = [0.0; 6];
        let mut serr = [0i8; 256];
        let mut base_positions = [0.0; 20];

        let bodies = [SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS,
                     SE_JUPITER, SE_SATURN, SE_URANUS, SE_NEPTUNE, SE_PLUTO,
                     SE_CHIRON, SE_TRUE_NODE, SE_MEAN_APOG, SE_VESTA, 
                     SE_JUNO, SE_CERES, SE_PALLAS, SE_ASC, SE_ARMC, (SE_AST_OFFSET + 5550)];

        println!("Calculating base positions for JD {}:", start_jd);
        for (i, &body) in bodies.iter().enumerate() {
            unsafe {
                let ret = swe_calc_ut(start_jd, body as i32,
                    (SEFLG_SPEED | SEFLG_SWIEPH) as i32,
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

        let interval = match precision {
            Precision::Second => "seconds",
            Precision::Minute => "minutes",
        };
        
        let total_intervals = ((end_jd - start_jd) / precision.to_jd()).ceil() as i32;
        println!("\nCalculating changes for {} {}...", total_intervals, interval);

        Ok(Self {
            timestamp: start_jd,
            base_positions,
            time_delta: end_jd - start_jd,
            precision,
        })
    }

    fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create("zenith.kernel")?;
        
        // Write header with precision flag
        file.write_all(&[match self.precision {
            Precision::Second => 1u8,
            Precision::Minute => 2u8,
        }])?;
        
        file.write_all(&self.timestamp.to_le_bytes())?;
        for pos in &self.base_positions {
            file.write_all(&(*pos as f64).to_le_bytes())?;
        }
        
        println!("\n✨ Kernel written");
        println!("Size: {} bytes", std::fs::metadata("zenith.kernel")?.len());
        println!("Precision: {:?}", self.precision);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    let args: Vec<String> = std::env::args().collect();
    
    let start_jd: f64 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(BASE_DATE);

    let end_jd: f64 = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(start_jd + 365.25);

    // Default to minute precision unless -s flag is present
    let precision = if args.iter().any(|arg| arg == "-s") {
        Precision::Second
    } else {
        Precision::Minute
    };

    println!("🚀 Zenith Engine Starting");
    println!("Processing JD {} to {}", start_jd, end_jd);
    println!("Precision: {:?}", precision);
    
    let kernel = ZenithKernel::new(start_jd, end_jd, precision)?;
    kernel.write()?;

    println!("\n✨ Completed in {:?}", start_time.elapsed());
    Ok(())
}