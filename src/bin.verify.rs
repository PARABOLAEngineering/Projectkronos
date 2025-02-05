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
    "⚷", "☊", "⚸", "⚴", "⊕", "⚶", "⯓", "☄︎ "
];

const SIGNS: [&str; 12] = ["♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓"];

fn decimal_to_dms(decimal_degrees: f64) -> (i32, i32, f64) {
    let total_seconds = (decimal_degrees * 3600.0).round() as i32;
    let degrees = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = (total_seconds % 3600) % 60;
    (degrees, minutes, seconds as f64)
}

fn format_position(deg: f64) -> String {
    let (degrees, minutes, seconds) = decimal_to_dms(deg);
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
    
    let search_jd = if args.len() == 2 {
        // Direct JD search
        args[1].parse::<f64>()?
    } else if args.len() == 7 {
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
    } else {
        println!("Usage:");
        println!("  {} <julian_date>            - Search by Julian Date", args[0]);
        println!("  {} YYYY MM DD HH MM SS      - Search by calendar date/time", args[0]);
        println!("\nExamples:");
        println!("  {} 2451545.0                - Search JD directly", args[0]);
        println!("  {} 2024 2 4 15 30 45        - Feb 4, 2024 at 15:30:45 UTC", args[0]);
        return Ok(());
    };

    let date_time = jd_to_datetime(search_jd);
    println!("
╭──────────────────────────────────────────────╮
│            ZODIAC EPHEMERIS QUERY            │ 
╰──────────────────────────────────────────────╯");

    println!("\n🔍 Time: {} UTC", date_time.format("%Y-%m-%d %H:%M:%S"));
    println!("   JD:   {:.6}", search_jd);

    // Configure Swiss Ephemeris for validation  
    unsafe {
        swe_set_ephe_path(std::ffi::CString::new("./ephe")?.as_ptr());
        swe_set_jpl_file(std::ffi::CString::new("de441.eph")?.as_ptr());
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

    println!("\n╭────────┬─────────────────┬────────────┬───────────╮");
    println!("│ Body   │    Position     │   Speed    │  Status   │");
    println!("├────────┼─────────────────┼────────────┼───────────┤");

    let groups = [
        (0..3, "Personal Planets"),
        (3..7, "Social Planets"), 
        (7..10, "Outer Planets"),
        (10..13, "Nodes & Points"),
        (13..17, "Asteroids"),
        (17..18, "Angles & Special Points")
    ];
    
    for &(ref range, title) in groups.iter() {
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
                    let swe_pos = xx[0].rem_euclid(360.0);
                    let speed = xx[3];

                    print!("│ {:<4} {} │ {} │ {} │",
                        SYMBOLS[i], 
                        BODIES[i].chars().take(2).collect::<String>(),
                        format_position(swe_pos),
                        format_speed(speed)
                    );
                    
                    // Status indicators
                    if speed < 0.0 {
                        println!("   ℞      │");
                    } else if speed.abs() < 0.0001 {
                        println!("   STAT   │"); 
                    } else {
                        println!("   DIR    │");
                    }
                }
            }
        }
        if range.end < 18 {
            println!("├────────┼─────────────────┼────────────┼───────────┤");
        }
    }
    println!("╰────────┴─────────────────┴────────────┴───────────╯"); 

    Ok(())
}