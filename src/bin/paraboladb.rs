use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
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

struct ParabolaReader {
    file: File,
    house_file: File,
}

impl ParabolaReader {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open("zenith.kernel")?;
        let house_file = File::open("houses.kernel")?;
        
        Ok(Self { file, house_file })
    }

    fn read_timestamp(&mut self, offset: u64) -> Result<f64, Box<dyn std::error::Error>> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut timestamp_bytes = [0u8; 8];
        self.file.read_exact(&mut timestamp_bytes)?;
        Ok(f64::from_le_bytes(timestamp_bytes))
    }

    fn read_positions(&mut self, offset: u64) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut positions = Vec::with_capacity(20);
        
        for _ in 0..20 {
            let mut pos_bytes = [0u8; 8];
            self.file.read_exact(&mut pos_bytes)?;
            positions.push(f64::from_le_bytes(pos_bytes));
        }
        
        Ok(positions)
    }

    fn read_houses(&mut self) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {
        // Skip location data
        self.house_file.seek(SeekFrom::Start(16))?;

        let mut house_positions = vec![Vec::with_capacity(12); 5];
        for system in &mut house_positions {
            for _ in 0..12 {
                let mut pos_bytes = [0u8; 8];
                self.house_file.read_exact(&mut pos_bytes)?;
                system.push(f64::from_le_bytes(pos_bytes));
            }
        }
        
        Ok(house_positions)
    }

    fn format_position(&self, deg: f64) -> String {
        let total_deg = deg.trunc() as i32;
        let minutes = ((deg - total_deg as f64) * 60.0).round() as i32;
        let sign_num = ((total_deg % 360) / 30) as usize;
        let sign_deg = total_deg % 30;
        format!("{}{}°{:02}'", SIGNS[sign_num], sign_deg, minutes)
    }

    fn find_closest_time(&mut self, target_jd: f64) -> Result<u64, Box<dyn std::error::Error>> {
        let mut offset = 0;
        let mut closest_offset = 0;
        let mut closest_diff = f64::MAX;

        loop {
            match self.read_timestamp(offset) {
                Ok(timestamp) => {
                    let diff = (timestamp - target_jd).abs();
                    if diff < closest_diff {
                        closest_diff = diff;
                        closest_offset = offset;
                    }
                    offset += 8 + (20 * 8);  // timestamp + positions
                },
                Err(_) => break
            }
        }

        Ok(closest_offset)
    }

    fn print_positions(&mut self, jd: f64) -> Result<(), Box<dyn std::error::Error>> {
        let offset = self.find_closest_time(jd)?;
        let timestamp = self.read_timestamp(offset)?;
        let positions = self.read_positions(offset + 8)?;
        let house_positions = self.read_houses()?;

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
                self.format_position(positions[i]).pad_to_width(15)
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
                    self.format_position(house_positions[i][h]).pad_to_width(13)
                );
                if h < 11 {
                    println!("├───────┼───────────────┤");
                }
            }
            println!("╰───────┴───────────────╯\n");
        }

        Ok(())
    }
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
    let args: Vec<String> = std::env::args().collect();
    let target_jd = if args.len() > 1 {
        args[1].parse()?
    } else {
        2451545.0  // J2000 if no argument
    };

    let mut reader = ParabolaReader::new()?;
    reader.print_positions(target_jd)?;

    Ok(())
}