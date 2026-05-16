#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HeartRate(pub u16);

impl HeartRate {
    pub fn new(bpm: u16) -> Self {
        assert!(bpm > 0 && bpm < 250, "Invalid heart rate");
        Self(bpm)
    }

    pub fn bpm(self) -> u16 {
        self.0
    }
}
