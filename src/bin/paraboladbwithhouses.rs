use std::fs::File;
use std::io::Read;
use chrono::{DateTime, TimeZone, Utc};

const BODIES: [&str; 20] = [
    "Sun", "Moon", "Mercury", "Venus", "Mars",
    "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto",
    "Chiron", "True Node", "Mean Apogee",
    "Vesta", "Juno", "Ceres", "Pallas", "ASC", "ARMC",
    "15550"
];

const SYMBOLS: [&str; 20] = [
    "☉", "☽", "☿", "♀", "♂", "♃", "♄", "♅", "♆", "⯓",
    "⚷", "☊", "⚸", "⚴", "⊕", "⚶", "⯓", "Asc", "MC",
    "☄︎"
];

const SIGNS: [&str; 12] = ["♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓"];

fn format_position(deg: f64) -> String {
    let total_deg = deg.trunc() as i32;
    let minutes = ((deg - total_deg as f64) * 60.0).round() as i32;
    let sign_num = ((total_deg % 360) / 30) as usize;
    let sign_deg = total_deg % 30;
    format!("{}{}°{:02}'", SIGNS[sign_num], sign_deg, minutes)
}

trait PadString {
    fn pad_to_width(&self, width: usize) -> String;
}

impl PadString for String {
    fn pad_to_width(&self, width: usize) -> String {
        if self.len() >= width {
            self.clone()
        } else {
            let padding = " ".repeat(width - self.len());
            format!("{}{}", self, padding)
        }
    }
}

fn jd_to_datetime(jd: f64) -> DateTime<Utc> {
    let unix_time = (jd - 2440587.5) * 86400.0;
    Utc.timestamp_opt(unix_time as i64, 0).unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read zenith kernel exactly like parabola-db
    let mut file = File::open("zenith.kernel")?;
    let mut timestamp_bytes = [0u8; 8];
    file.read_exact(&mut timestamp_bytes)?;
    let timestamp = f64::from_le_bytes(timestamp_bytes);

    let mut positions = Vec::with_capacity(20);
    for _ in 0..20 {
        let mut pos_bytes = [0u8; 8];
        file.read_exact(&mut pos_bytes)?;
        positions.push(f64::from_le_bytes(pos_bytes));
    }

    // Read houses exactly like working house reader
    let mut house_file = File::open("houses.kernel")?;
    let mut loc_bytes = [0u8; 16];
    house_file.read_exact(&mut loc_bytes)?;

    let mut house_positions = vec![Vec::with_capacity(12); 5];
    for system in &mut house_positions {
        for _ in 0..12 {
            let mut pos_bytes = [0u8; 8];
            house_file.read_exact(&mut pos_bytes)?;
            system.push(f64::from_le_bytes(pos_bytes));
        }
    }

    // Print output
    let date_time = jd_to_datetime(timestamp);
    println!("\n🔍 Time: {} UTC", date_time.format("%Y-%m-%d %H:%M:%S"));
    println!("   JD:   {:.6}\n", timestamp);

    println!("╭────────┬─────────────────╮");
    println!("│ Body   │    Position     │");
    println!("├────────┼─────────────────┤");

    for i in 0..20 {
        print!("│ {:<4} {} │ {} │\n",
            SYMBOLS[i],
            BODIES[i].chars().take(2).collect::<String>(),
            format_position(positions[i]).pad_to_width(15)
        );
        if i < 19 {
            println!("├────────┼─────────────────┤");
        }
    }
    println!("╰────────┴─────────────────╯\n");

    // Print houses
    let names = ["Placidus", "Koch", "Equal", "Whole Sign", "Regiomontanus"];
    for (i, name) in names.iter().enumerate() {
        println!("{} Houses:", name);
        println!("╭───────┬───────────────╮");
        println!("│ House │   Position    │");
        println!("├───────┼───────────────┤");

        for h in 0..12 {
            println!("│  {:2}   │ {} │",
                h + 1,
                format_position(house_positions[i][h]).pad_to_width(13)
            );
            if h < 11 {
                println!("├───────┼───────────────┤");
            }
        }
        println!("╰───────┴───────────────╯\n");
    }

    Ok(())
}