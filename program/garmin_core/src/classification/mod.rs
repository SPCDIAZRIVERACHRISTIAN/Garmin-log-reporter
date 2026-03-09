use crate::activity::Activity;
use crate::errors::classification_error::ClassError;

pub mod intensity;
pub mod terrain;

pub use intensity::Intensity;
pub use terrain::Terrain;

pub fn classify_intensity(activity: &Activity) -> Result<Intensity, ClassError> {
    let avg = activity
        .avg_hr
        .ok_or(ClassError::MissingMetrics("avg_hr missing".to_string()))?;

    let max = activity
        .max_hr
        .ok_or(ClassError::MissingMetrics("max_hr missing".to_string()))?;

    let intensity_ratio = avg.0 as f64 / max.0 as f64;

    match intensity_ratio {
        r if r < 0.60 => Ok(Intensity::Recovery),
        r if r < 0.70 => Ok(Intensity::Easy),
        r if r < 0.80 => Ok(Intensity::Moderate),
        r if r < 0.90 => Ok(Intensity::Threshold),
        _ => Ok(Intensity::Vo2Max),
    }
}

pub fn classify_terrain(_activity: &Activity) -> Result<Terrain, ClassError> {
    Ok(Terrain::Unknown)
}
