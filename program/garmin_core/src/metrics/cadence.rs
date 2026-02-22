#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Cadence(f64);

impl Cadence {
    pub fn new(spm: f64) -> Self {
        assert!(spm >= 0.0 && spm < 300.0, "Invalid cadence");
        Self(spm)
    }

    pub fn spm(self) -> f64 {
        self.0
    }
}
