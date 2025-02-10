use std::fs::File;
use std::io::Read;
use swisseph_sys::*;
use medusa::SE_AST_OFFSET;
use chrono::{DateTime, TimeZone, Utc};

const BODIES: [&str; 18] = [
    "Sun", "Moon", "Mercury", "Venus", "Mars",
    "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto",
    "Chiron", "True Node", "Mean Apogee",
    "Vesta", "Juno", "Ceres", "Pallas", "15550"
];

const SYMBOLS: [&str; 18] = [
    "☉", "☽", "☿", "♀", "♂", "♃", "♄", "♅", "♆", "⯓",
    "⚷", "☊", "⚸", "⚶", "⚵", "⚳", "⚴", "☄︎ "
];

const SIGNS: [&str; 12] = ["♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓"];

const GROUPS: [(std::ops::Range<usize>, &str); 6] = [
    (0..3, "Personal Planets"),
    (3..7, "Social Planets"),
    (7..10, "Outer Planets"),
    (10..13, "Nodes & Points"),
    (13..17, "Asteroids"),
    (17..18, "Angles & Special Points")
];

#[derive(Debug, Clone, Copy)]
enum ZodiacMode {
    Tropical,
    FaganBradley,
    Lahiri,
    TrueCitra
}

impl ZodiacMode {
    fn name(&self) -> &'static str {
        match self {
            ZodiacMode::Tropical => "Tropical",
            ZodiacMode::FaganBradley => "Fagan/Bradley",
            ZodiacMode::Lahiri => "Lahiri",
            ZodiacMode::TrueCitra => "True Citra"
        }
    }
}

fn decimal_to_dms(decimal_degrees: f64) -> (i32, i32, f64) {
    let total_seconds = (decimal_degrees * 3600.0).round() as i32;
    let degrees = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = (total_seconds % 3600) % 60;
    (degrees, minutes, seconds as f64)
}

fn format_position(deg: f64, mode: ZodiacMode, ayanamsa_values: &[f64]) -> String {
    // Convert tropical to sidereal if needed
    let adjusted_deg = match mode {
        ZodiacMode::Tropical => deg,
        ZodiacMode::FaganBradley => deg - ayanamsa_values[0],
        ZodiacMode::Lahiri => deg - ayanamsa_values[1],
        ZodiacMode::TrueCitra => deg - ayanamsa_values[2],
    }.rem_euclid(360.0);

    let (degrees, minutes, seconds) = decimal_to_dms(adjusted_deg);
    let sign_num = ((degrees % 360) / 30) as usize;
    let sign_deg = degrees % 30;
    format!("{}{}°{:02}'{:02}\"", SIGNS[sign_num], sign_deg, minutes, seconds)
}

fn format_speed(speed: f64) -> String {
    if speed.abs() < 0.0001 {
        "   STAT   ".to_string()
    } else if speed < 0.0 {
        format!(" ℞{:6.2} ", speed.abs())
    } else {
        format!("  {:6.2}  ", speed)
    }
}

fn jd_to_datetime(jd: f64) -> DateTime<Utc> {
    let unix_time = (jd - 2440587.5) * 86400.0;
    Utc.timestamp_opt(unix_time as i64, 0).unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    // Add -s flag for sidereal mode with optional ayanamsa specification
    let zodiac_mode = args.iter().find(|arg| arg.starts_with("-s"))
        .map_or(ZodiacMode::Tropical, |arg| {
            match arg.as_str() {
                "-sf" => ZodiacMode::FaganBradley,
                "-sl" => ZodiacMode::Lahiri,
                "-sc" => ZodiacMode::TrueCitra,
                _ => ZodiacMode::FaganBradley // Default to Fagan/Bradley
            }
        });

    let search_jd = if args.len() >= 7 {
        // Calendar time search: YYYY MM DD HH MM SS
        let year: i32 = args[1].parse()?;
        let month: u32 = args[2].parse()?;
        let day: u32 = args[3].parse()?;
        let hour: u32 = args[4].parse()?;
        let minute: u32 = args[5].parse()?;
        let second: u32 = args[6].parse()?;

        let dt = Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .ok_or("Invalid date/time")?;
        
        (dt.timestamp() as f64 / 86400.0) + 2440587.5
    } else if args.len() == 3 && args[1].parse::<f64>().is_ok() {
        // Direct JD search
        args[1].parse::<f64>()?
    } else {
        println!("Usage:");
        println!("  {} [-s[f|l|c]] <julian_date>       - Search by Julian Date", args[0]);
        println!("  {} [-s[f|l|c]] YYYY MM DD HH MM SS - Search by calendar date/time", args[0]);
        println!("\nSidereal Modes:");
        println!("  -sf  Fagan/Bradley (default sidereal)");
        println!("  -sl  Lahiri");
        println!("  -sc  True Citra");
        println!("\nExamples:");
        println!("  {} 2451545.0                - J2000 Tropical", args[0]);
        println!("  {} -sl 2024 2 4 15 30 45    - Time in Lahiri Sidereal", args[0]);
        return Ok(());
    };

    let date_time = jd_to_datetime(search_jd);
    println!("
╭──────────────────────────────────────────────╮
│            PARABOLA EPHEMERIS                │
╰──────────────────────────────────────────────╯");

    println!("\n🔍 Time: {} UTC", date_time.format("%Y-%m-%d %H:%M:%S"));
    println!("   JD:   {:.6}", search_jd);
    println!("   Mode: {}", zodiac_mode.name());

    // Configure Swiss Ephemeris for validation
    unsafe {
        swe_set_ephe_path(std::ffi::CString::new("./ephe")?.as_ptr());
        swe_set_jpl_file(std::ffi::CString::new("de441.eph")?.as_ptr());
    }

    // Read kernel
    let mut file = File::open("zenith.kernel")?;
    let mut timestamp_bytes = [0u8; 8];
    file.read_exact(&mut timestamp_bytes)?;
    let timestamp = f64::from_le_bytes(timestamp_bytes);

    let mut base_positions = Vec::with_capacity(18);
    for _ in 0..18 {
        let mut pos_bytes = [0u8; 8];
        file.read_exact(&mut pos_bytes)?;
        base_positions.push(f64::from_le_bytes(pos_bytes));
    }

    // Read ayanamsa values
    let mut ayanamsa_values = [0.0; 3];
    for i in 0..3 {
        let mut ayan_bytes = [0u8; 8];
        file.read_exact(&mut ayan_bytes)?;
        ayanamsa_values[i] = f64::from_le_bytes(ayan_bytes);
    }

    // Calculate current positions
    let mut xx = [0.0; 6];
    let mut serr = [0i8; 256];
    let bodies = [
        SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS, SE_JUPITER, 
        SE_SATURN, SE_URANUS, SE_NEPTUNE, SE_PLUTO, SE_CHIRON, 
        SE_TRUE_NODE, SE_MEAN_APOG, SE_VESTA, SE_JUNO, SE_CERES,
        SE_PALLAS, SE_ASC, SE_ARMC, (SE_AST_OFFSET + 5550)
    ];

    println!("\n╭────────┬─────────────────┬────────────┬───────────┬─────────╮");
    println!("│ Body   │    Position     │   Speed    │  Status   │  Δ SwE  │");
    println!("├────────┼─────────────────┼────────────┼───────────┼─────────┤");

    for &(ref range, _title) in GROUPS.iter() {
        for i in range.clone() {
            unsafe {
                let ret = swe_calc_ut(
                    search_jd,
                    bodies[i] as i32,
                    (SEFLG_SPEED | SEFLG_SWIEPH) as i32,
                    xx.as_mut_ptr(),
                    serr.as_mut_ptr()
                );

                if ret >= 0 {
                    let kernel_pos = base_positions[i];
                    let swe_pos = xx[0].rem_euclid(360.0);
                    let speed = xx[3];
                    let diff = (kernel_pos - swe_pos).abs();
                    
                    print!("│ {:<4} {} │ {} │ {} │", 
                        SYMBOLS[i],
                        BODIES[i].chars().take(2).collect::<String>(),
                        format_position(swe_pos, zodiac_mode, &ayanamsa_values),
                        format_speed(speed)
                    );
                    
                    // Status indicators
                    if speed < 0.0 {
                        print!("   ℞      │");
                    } else if speed.abs() < 0.0001 {
                        print!("   STAT   │");
                    } else {
                        print!("   DIR    │");
                    }

                    // Difference indicator
                    if diff > 0.0001 {
                        println!(" {:6.3}° │", diff);
                    } else {
                        println!("   OK   │");
                    }
                }
            }
        }
        if range.end < 18 {
            println!("├────────┼─────────────────┼────────────┼───────────┼─────────┤");
        }
    }
    println!("╰────────┴─────────────────┴────────────┴───────────┴─────────╯");

    Ok(())
}