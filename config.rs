use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct MedusaConfig {
    pub scan: ScanConfig,
    pub paths: PathConfig,
    pub memory: MemoryConfig,
    pub kernel: KernelConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, Deserialize)]
pub struct ScanConfig {
    pub start_jd: f64,
    pub end_jd: f64,
    pub interval: String,
    pub chunk_size: f64,
    pub parallel_chunks: usize,
}

#[derive(Debug, Deserialize)]
pub struct PathConfig {
    pub ephe_path: PathBuf,
    pub ephe_file: String,
    pub coords_path: PathBuf,
    pub coords_format: String,
    pub output_path: PathBuf,
    pub temp_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct MemoryConfig {
    pub batch_size: usize,
    pub max_memory_percent: f64,
}

#[derive(Debug, Deserialize)]
pub struct KernelConfig {
    pub position_bits: u8,
    pub metadata_offset: u8,
    pub eclipse_offset: u8,
    pub metadata_in_kernel: bool,
    pub include_eclipses: bool,
    pub version: u8,
    pub flags: Vec<String>,
    pub compression: bool,
    pub include_houses: bool,
    pub pack_metadata: bool,
    pub validate: bool,
}

#[derive(Debug, Deserialize)]
pub struct DebugConfig {
    pub enabled: bool,
    pub log_path: String,
}

impl MedusaConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: MedusaConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate paths exist
        if !self.paths.ephe_path.exists() {
            return Err(format!("Ephemeris path does not exist: {:?}", self.paths.ephe_path));
        }
        if !self.paths.coords_path.exists() {
            return Err(format!("Coordinates file does not exist: {:?}", self.paths.coords_path));
        }

        // Validate JD range
        if self.scan.end_jd <= self.scan.start_jd {
            return Err("End JD must be greater than start JD".to_string());
        }

        // Validate memory settings
        if self.memory.max_memory_percent > 100.0 || self.memory.max_memory_percent <= 0.0 {
            return Err("Memory percentage must be between 0 and 100".to_string());
        }

        Ok(())
    }
}