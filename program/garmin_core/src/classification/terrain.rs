use std::str::FromStr;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Terrain {
    Road,
    Trail,
    Unknown,
}

impl FromStr for Terrain {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(Terrain::Road), // adjust based on your enum
            "trail_running" => Ok(Terrain::Trail),
            _ => Err(()),
        }
    }
}
