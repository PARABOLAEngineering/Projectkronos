// src/bin/expand.rs
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::time::Instant;

// Cubic interpolation for smoother transitions
fn cubic_interpolate(pos0: f64, pos1: f64, pos2: f64, pos3: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;

    // Adjust all positions relative to pos1 to handle degree wrap-around
    let mut p0 = pos0 - pos1;
    let mut p2 = pos2 - pos1;
    let mut p3 = pos3 - pos1;
    
    // Handle wrap-around
    if p0 > 180.0 { p0 -= 360.0; } else if p0 < -180.0 { p0 += 360.0; }
    if p2 > 180.0 { p2 -= 360.0; } else if p2 < -180.0 { p2 += 360.0; }
    if p3 > 180.0 { p3 -= 360.0; } else if p3 < -180.0 { p3 += 360.0; }

    // Catmull-Rom spline coefficients
    let a = -0.5 * p0 + 1.5 * p2 - 1.5 * pos1 + 0.5 * p3;
    let b = p0 - 2.5 * p2 + 2.0 * pos1 - 0.5 * p3;
    let c = -0.5 * p0 + 0.5 * p2;
    let d = pos1;

    // Calculate interpolated value and handle wrap-around
    let mut result = a * t3 + b * t2 + c * t + d;
    result = result.rem_euclid(360.0);
    
    result
}

fn read_hour_positions(file: &mut File, hour_offset: i64, num_bodies: usize) -> Result<Vec<f64>, std::io::Error> {
    let pos_size: u64 = 8; // size of f64 in bytes
    let body_start = 8u64 + (hour_offset as u64 * num_bodies as u64 * pos_size); // 8 for base JD
    
    file.seek(SeekFrom::Start(body_start))?;
    
    let mut positions = Vec::with_capacity(num_bodies as usize);
    let mut pos_bytes = [0u8; 8];
    
    for _ in 0..num_bodies as usize {
        if file.read_exact(&mut pos_bytes).is_ok() {
            positions.push(f64::from_le_bytes(pos_bytes));
        } else {
            // If we can't read a position, replicate the last hour's position
            if let Some(&last_pos) = positions.last() {
                positions.push(last_pos);
            } else {
                positions.push(0.0);
            }
        }
    }
    
    Ok(positions)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("🔄 Loading base kernel...");

    let mut base_kernel = File::open("zenith.kernel")?;
    let kernel_size = base_kernel.metadata()?.len();
    
    // Read base JD
    let mut jd_bytes = [0u8; 8];
    base_kernel.read_exact(&mut jd_bytes)?;
    let base_jd = f64::from_le_bytes(jd_bytes);

    // Read first hour to determine number of bodies
    let mut first_hour = Vec::new();
    let mut pos_bytes = [0u8; 8];
    while let Ok(_) = base_kernel.read_exact(&mut pos_bytes) {
        first_hour.push(f64::from_le_bytes(pos_bytes));
    }
    let num_bodies = first_hour.len();  // Keep as usize for indexing

    // Calculate total hours in kernel
    let total_hours = ((kernel_size - 8) / (8 * num_bodies as u64)) as i64;
    
    println!("Base kernel loaded:");
    println!("Bodies: {}", num_bodies);
    println!("Hours: {}", total_hours);
    println!("Expanding to minute precision...");

    // Create minute kernel
    let mut minute_kernel = File::create("zenith.minute")?;
    minute_kernel.write_all(&base_jd.to_le_bytes())?;

    // Process each hour
    for hour in 0..total_hours {
        // Read 4 consecutive hours for cubic interpolation
        let h0 = if hour > 0 { 
            read_hour_positions(&mut base_kernel, hour - 1, num_bodies)?
        } else {
            first_hour.clone()
        };
        
        let h1 = if hour == 0 { 
            first_hour.clone() 
        } else { 
            read_hour_positions(&mut base_kernel, hour, num_bodies)?
        };
        
        let h2 = read_hour_positions(&mut base_kernel, hour + 1, num_bodies)?;
        let h3 = read_hour_positions(&mut base_kernel, hour + 2, num_bodies)?;

        // Interpolate each minute
        for minute in 0..60 {
            let t = minute as f64 / 60.0;
            
            // Interpolate each body's position
            for i in 0..num_bodies {
                let interpolated = cubic_interpolate(
                    h0[i], h1[i], h2[i], h3[i], t
                );
                minute_kernel.write_all(&interpolated.to_le_bytes())?;
            }
        }

        if hour % 24 == 0 {
            println!("Processing hour {} of {} ({:.1}%)", 
                    hour, total_hours, (hour as f64 * 100.0) / total_hours as f64);
        }
    }

    let duration = start_time.elapsed();
    println!("\n✨ Minute kernel generated in {:?}", duration);
    println!("Original size: {} bytes", kernel_size);
    println!("Minute kernel size: {} bytes", std::fs::metadata("zenith.minute")?.len());

    Ok(())
}