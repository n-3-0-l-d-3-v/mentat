use isa::NUM_REGISTERS;

/// The bounded, fixed-size architectural register file. 64-bit signed
/// integer registers; there is no zero register and no aliasing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegisterFile {
    values: [i64; NUM_REGISTERS],
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self { values: [0; NUM_REGISTERS] }
    }
}

impl RegisterFile {
    pub fn get(&self, r: u8) -> i64 {
        self.values[r as usize]
    }

    pub fn set(&mut self, r: u8, v: i64) {
        self.values[r as usize] = v;
    }

    pub fn snapshot(&self) -> [i64; NUM_REGISTERS] {
        self.values
    }
}
