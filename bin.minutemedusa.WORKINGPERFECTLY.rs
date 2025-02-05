use std::fs::File;
use std::io::{Read, Write};
use medusa::SE_AST_OFFSET;
use swisseph_sys::*;
use std::time::Instant;

const BASE_DATE: f64 = 625615.0;
const MINUTES_PER_DAY: f64 = 1440.0;
const JD_MINUTE: f64 = 1.0 / MINUTES_PER_DAY;

struct ZenithKernel {
    timestamp: f64,
    base_positions: [f64; 20],
    time_delta: f64,
    precision: TimePrec,
}

#[derive(Clone, Copy)]
enum TimePrec {
    Minute,
    Hour,
    Day,
}

impl TimePrec {
    fn to_jd(&self) -> f64 {
        match self {
            TimePrec::Minute => JD_MINUTE,
            TimePrec::Hour => JD_MINUTE * 60.0,
            TimePrec::Day => 1.0,
        }
    }
    
    fn intervals_per_day(&self) -> u32 {
        match self {
            TimePrec::Minute => 1440,
            TimePrec::Hour => 24,
            TimePrec::Day => 1,
        }
    }
}

impl ZenithKernel {
    fn new(start_jd: f64, end_jd: f64, precision: TimePrec) -> Result<Self, Box<dyn std::error::Error>> {
        let mut xx = [0.0; 6];
        let mut serr = [0i8; 256];
        let mut base_positions = [0.0; 20];

        let bodies = [SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS,
                     SE_JUPITER, SE_SATURN, SE_URANUS, SE_NEPTUNE, SE_PLUTO,
                     SE_CHIRON, SE_TRUE_NODE, SE_MEAN_APOG, SE_VESTA, 
                     SE_JUNO, SE_CERES, SE_PALLAS, SE_ASC, SE_ARMC, (SE_AST_OFFSET + 5550)];

        println!("🔍 Initializing kernel with {} precision", match precision {
            TimePrec::Minute => "minute",
            TimePrec::Hour => "hour",
            TimePrec::Day => "day",
        });
        
        // Calculate initial positions
        println!("📊 Calculating base positions for JD {}", start_jd);
        for (i, &body) in bodies.iter().enumerate() {
            unsafe {
                let ret = swe_calc_ut(
                    start_jd,
                    body as i32,
                    (SEFLG_SPEED | SEFLG_SWIEPH) as i32,
                    xx.as_mut_ptr(),
                    serr.as_mut_ptr()
                );

                if ret >= 0 {
                    let pos = xx[0].rem_euclid(360.0);
                    println!("  {} → {:.6}°", body_name(body), pos);
                    base_positions[i] = pos;
                } else {
                    return Err(format!("Failed to calculate position for body {}", i).into());
                }
            }
        }

        let total_intervals = ((end_jd - start_jd) / precision.to_jd()).ceil() as u32;
        println!("\n⏱️  Processing {} intervals...", total_intervals);
        
        Ok(Self {
            timestamp: start_jd,
            base_positions,
            time_delta: end_jd - start_jd,
            precision,
        })
    }

    fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut file = File::create("zenith.kernel")?;
        
        // Write header information
        file.write_all(b"ZNTH")?; // Magic number
        file.write_all(&[1])?;    // Version
        file.write_all(&(self.precision as u8).to_le_bytes())?;
        file.write_all(&self.timestamp.to_le_bytes())?;
        file.write_all(&self.time_delta.to_le_bytes())?;

        // Write base positions with metadata
        for pos in &self.base_positions {
            // Convert position to centiseconds for higher precision storage
            let centisec = (*pos * 3600.0 * 100.0) as u32;
            file.write_all(&centisec.to_le_bytes())?;
        }

        // Calculate and write positions for each interval
        let intervals = (self.time_delta / self.precision.to_jd()).ceil() as u32;
        let mut xx = [0.0; 6];
        let mut serr = [0i8; 256];
        
        let pb = indicatif::ProgressBar::new(intervals as u64);
        pb.set_style(indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:50}] {pos}/{len} ({eta})")
            .unwrap());

        for interval in 0..intervals {
            let current_jd = self.timestamp + (interval as f64 * self.precision.to_jd());
            
            for (i, &body) in self.bodies().iter().enumerate() {
                unsafe {
                    if swe_calc_ut(
                        current_jd,
                        body as i32,
                        (SEFLG_SPEED | SEFLG_SWIEPH) as i32,
                        xx.as_mut_ptr(),
                        serr.as_mut_ptr()
                    ) >= 0 {
                        let pos = xx[0].rem_euclid(360.0);
                        let centisec = (pos * 3600.0 * 100.0) as u32;
                        file.write_all(&centisec.to_le_bytes())?;
                    }
                }
            }
            
            pb.inc(1);
        }
        
        pb.finish();

        let size = std::fs::metadata("zenith.kernel")?.len();
        println!("\n✨ Kernel written successfully:");
        println!("  📁 Size: {} bytes", size);
        println!("  ⏱️  Time: {:?}", start.elapsed());
        println!("  🎯 Precision: {} entries/day", self.precision.intervals_per_day());

        Ok(())
    }

    fn bodies(&self) -> Vec<u32> {
        vec![SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS,
             SE_JUPITER, SE_SATURN, SE_URANUS, SE_NEPTUNE, SE_PLUTO,
             SE_CHIRON, SE_TRUE_NODE, SE_MEAN_APOG, SE_VESTA, 
             SE_JUNO, SE_CERES, SE_PALLAS, SE_ASC, SE_ARMC, 
             (SE_AST_OFFSET + 5550)]
    }
}

fn body_name(body: u32) -> &'static str {
    match body {
        SE_SUN => "Sun",
        SE_MOON => "Moon",
        SE_MERCURY => "Mercury",
        SE_VENUS => "Venus",
        SE_MARS => "Mars",
        SE_JUPITER => "Jupiter",
        SE_SATURN => "Saturn",
        SE_URANUS => "Uranus",
        SE_NEPTUNE => "Neptune",
        SE_PLUTO => "Pluto",
        SE_CHIRON => "Chiron",
        SE_TRUE_NODE => "True Node",
        SE_MEAN_APOG => "Mean Apogee",
        SE_VESTA => "Vesta",
        SE_JUNO => "Juno",
        SE_CERES => "Ceres",
        SE_PALLAS => "Pallas",
        SE_ASC => "Ascendant",
        SE_ARMC => "ARMC",
        _ => "Unknown",
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

    // Default to minute precision if not specified
    let precision = args.get(3)
        .and_then(|s| match s.as_str() {
            "m" | "minute" => Some(TimePrec::Minute),
            "h" | "hour" => Some(TimePrec::Hour),
            "d" | "day" => Some(TimePrec::Day),
            _ => None
        })
        .unwrap_or(TimePrec::Minute);

    println!("🚀 Zenith Engine Starting");
    println!("🕒 Processing JD {} to {}", start_jd, end_jd);
    
    let kernel = ZenithKernel::new(start_jd, end_jd, precision)?;
    kernel.write()?;

    println!("\n✨ Completed in {:?}", start_time.elapsed());
    Ok(())
}