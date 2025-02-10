use swisseph_sys::*;

const AURORA_LAT: f64 = 39.7294319;
const AURORA_LON: f64 = -104.8319195;
const HOUSE_SYSTEMS: [(char, &str); 8] = [
    ('P', "Placidus"),
    ('K', "Koch"),
    ('O', "Porphyrius"),
    ('R', "Regiomontanus"),
    ('C', "Campanus"),
    ('E', "Equal"),
    ('V', "Vehlow"),
    ('W', "Whole Sign")
];

fn main() {
    unsafe {
        swe_set_ephe_path("./ephe\0".as_ptr() as *const i8);
        
        let jd = 2460000.5; // Sample date
        let mut cusps = [0.0; 13];
        let mut ascmc = [0.0; 10];

        println!("\nHouse positions for Aurora, CO at JD {}", jd);
        println!("Latitude: {}°N, Longitude: {}°W\n", AURORA_LAT, AURORA_LON.abs());

        for (system, name) in HOUSE_SYSTEMS {
            let ret = swe_houses(
                jd,
                AURORA_LAT,
                AURORA_LON,
                system as i32,
                cusps.as_mut_ptr(),
                ascmc.as_mut_ptr()
            );

            if ret >= 0 {
                println!("╭──────────────────────────────────╮");
                println!("│ {} System ({}):", name, system);
                println!("├──────────────────────────────────┤");
                
                // Print special points
                println!("│ ASC: {:.6}°", ascmc[0]);
                println!("│ MC:  {:.6}°", ascmc[1]);
                println!("│ ARMC:{:.6}°", ascmc[2]);
                
                println!("├──────────────────────────────────┤");
                println!("│ House Cusps:");
                
                // Print house cusps and their differences
                for i in 1..13 {
                    let cusp = cusps[i];
                    let next_cusp = if i == 12 { cusps[1] } else { cusps[i + 1] };
                    let diff = (next_cusp - cusp).rem_euclid(360.0);
                    
                    println!("│ H{:2}: {:.6}° (span: {:.6}°)", 
                            i, cusp, diff);
                }
                println!("╰──────────────────────────────────╯\n");

                // Print differences from ARMC for analysis
                println!("Offsets from ARMC:");
                for i in 1..13 {
                    let offset = (cusps[i] - ascmc[2]).rem_euclid(360.0);
                    println!("House {}: {:.6}°", i, offset);
                }
                println!();
            }
        }
    }
}