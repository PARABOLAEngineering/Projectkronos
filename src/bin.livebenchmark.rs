use std::fs::File;
use std::io::Read;
use swisseph_sys::*;
use medusa::SE_AST_OFFSET;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use colored::*;
use std::time::Instant;
use chrono::{TimeZone, Utc};

const CHECK_INTERVAL: f64 = 1.0 / 86400.0; // Check every minute

fn validate_kernel(start_jd: f64, end_jd: f64) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Setup interface
    println!("\n🔍 PARABOLA KERNEL VALIDATOR");
    println!("════════════════════════════\n");

    // Configure Swiss Ephemeris
    unsafe {
        swe_set_ephe_path(std::ffi::CString::new("./ephe")?.as_ptr());
        swe_set_jpl_file(std::ffi::CString::new("de441.eph")?.as_ptr());
    }

    let bodies = [
        ("Sun ☉", SE_SUN),
        ("Moon ☽", SE_MOON),
        ("Mercury ☿", SE_MERCURY),
        ("Venus ♀", SE_VENUS),
        ("Mars ♂", SE_MARS),
        ("Jupiter ♃", SE_JUPITER),
        ("Saturn ♄", SE_SATURN),
        ("Uranus ♅", SE_URANUS),
        ("Neptune ♆", SE_NEPTUNE),
        ("Pluto ⯓", SE_PLUTO)
    ];

    let mp = MultiProgress::new();
    
    // Overall progress
    let total_days = (end_jd - start_jd).ceil() as u64;
    let total_pb = mp.add(ProgressBar::new(total_days));
    total_pb.set_style(ProgressStyle::default_bar()
        .template("{prefix:.bold.dim} {spinner:.green} [{bar:40.cyan/blue}] {msg}")
        .unwrap()
        .progress_chars("█▇▆▅▄▃▂▁  "));
    total_pb.set_prefix("Time Progress");

    // Create progress bar for each planet
    let mut body_pbs = Vec::new();
    for (name, _) in &bodies {
        let pb = mp.add(ProgressBar::new(1000));
        pb.set_style(ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {spinner:.green} [{bar:40.red/yellow}] {msg}")
            .unwrap()
            .progress_chars("█▇▆▅▄▃▂▁  "));
        pb.set_prefix(format!("{:.<12}", name));
        body_pbs.push(pb);
    }

    let mut xx = [0.0; 6];
    let mut serr = [0i8; 256];

    let mut current_jd = start_jd;
    let mut max_errors = vec![0.0f64; bodies.len()];
    let mut checks = vec![0u64; bodies.len()];
    let mut total_checks = 0u64;

    println!("\n🌟 Validating from {} to {}", 
        format_date(start_jd)?.bright_green(),
        format_date(end_jd)?.bright_green()
    );

    while current_jd <= end_jd {
        for (i, (_, body)) in bodies.iter().enumerate() {
            unsafe {
                let ret = swe_calc_ut(
                    current_jd,
                    (*body).try_into().unwrap(),
                    (SEFLG_SPEED | SEFLG_SWIEPH) as i32,
                    xx.as_mut_ptr(),
                    serr.as_mut_ptr()
                );

                if ret >= 0 {
                    let kernel_pos = 0.0; // Replace with actual kernel lookup
                    let swe_pos = xx[0].rem_euclid(360.0);
                    let error = (kernel_pos - swe_pos).abs();
                    
                    max_errors[i] = max_errors[i].max(error);
                    checks[i] += 1;

                    // Update progress - scale error to 0-1000
                    let progress = ((1.0 - (error / 1.0)) * 1000.0) as u64;
                    body_pbs[i].set_position(progress);
                    
                    // Color-coded error display
                    let error_str = if error < 0.0001 {
                        "PERFECT".green()
                    } else if error < 0.001 {
                        format!("{:.6}°", error).yellow()
                    } else {
                        format!("{:.6}°", error).red()
                    };
                    
                    body_pbs[i].set_message(format!("Max Difference: {}", error_str));
                }
            }
        }

        total_checks += 1;
        
        // Update overall progress
        let days_done = (current_jd - start_jd).ceil() as u64;
        total_pb.set_position(days_done);
        total_pb.set_message(format!("{} ({} checks)", 
            format_date(current_jd)?.blue(),
            total_checks.to_string().yellow()
        ));

        current_jd += CHECK_INTERVAL;
    }

    // Final summary
    println!("\n\n📊 VALIDATION SUMMARY");
    println!("══════════════════\n");
    
    for (i, (name, _)) in bodies.iter().enumerate() {
        let accuracy = 100.0 * (1.0 - (max_errors[i] / 360.0));
        let accuracy_str = if accuracy > 99.9999 {
            "100.0000%".green()
        } else {
            format!("{:8.4}%", accuracy).yellow()
        };
        
        println!("{:12} │ Accuracy: {} │ Checks: {}", 
            name,
            accuracy_str,
            checks[i].to_string().blue()
        );
    }

    let elapsed = start_time.elapsed();
    println!("\n✨ Validation complete in {:.2?}", elapsed);
    println!("📝 Total positions checked: {}", total_checks.to_string().green());
    
    let checks_per_sec = total_checks as f64 / elapsed.as_secs_f64();
    println!("⚡ Speed: {} checks/second", 
        format!("{:.2}", checks_per_sec).bright_yellow()
    );

    Ok(())
}

fn format_date(jd: f64) -> Result<String, Box<dyn std::error::Error>> {
    let unix_time = (jd - 2440587.5) * 86400.0;
    let dt = Utc.timestamp_opt(unix_time as i64, 0).unwrap();
    Ok(dt.format("%Y-%m-%d").to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        println!("
╭──────────────────────────────────────────────╮
│      PARABOLA KERNEL VALIDATOR - HELP        │
╰──────────────────────────────────────────────╯

Usage:
  {} [start_jd] [end_jd]

Examples:
  {} -13000 17000    - Validate years -13000 to 17000
  {} 2451545 2460000 - Validate specific JD range

If no dates provided, validates full DE441 range.
", args[0], args[0], args[0]);
        return Ok(());
    }

    let (start_jd, end_jd) = if args.len() > 2 {
        (args[1].parse()?, args[2].parse()?)
    } else {
        (-1845369.5, 7930192.5)  // Full DE441 range
    };

    validate_kernel(start_jd, end_jd)?;
    Ok(())
}