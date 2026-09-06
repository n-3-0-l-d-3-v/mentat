use crate::Trap;

/// Linear, byte-addressable memory. All loads/stores in this ISA are 8-byte
/// little-endian words; bounds are checked on every access and reported as
/// a `Trap` rather than a host-language panic.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Memory {
    bytes: Vec<u8>,
}

pub const WORD_LEN: usize = 8;

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0u8; size],
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn load_u64(&self, addr: u64, block: usize, instr: usize) -> Result<u64, Trap> {
        let start = addr as usize;
        let end = start.checked_add(WORD_LEN).ok_or(Trap::OutOfBoundsMemory {
            addr,
            len: WORD_LEN as u8,
            size: self.bytes.len(),
            block,
            instr,
        })?;
        if end > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemory {
                addr,
                len: WORD_LEN as u8,
                size: self.bytes.len(),
                block,
                instr,
            });
        }
        let mut buf = [0u8; WORD_LEN];
        buf.copy_from_slice(&self.bytes[start..end]);
        Ok(u64::from_le_bytes(buf))
    }

    pub fn store_u64(
        &mut self,
        addr: u64,
        value: u64,
        block: usize,
        instr: usize,
    ) -> Result<(), Trap> {
        let start = addr as usize;
        let end = start.checked_add(WORD_LEN).ok_or(Trap::OutOfBoundsMemory {
            addr,
            len: WORD_LEN as u8,
            size: self.bytes.len(),
            block,
            instr,
        })?;
        if end > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemory {
                addr,
                len: WORD_LEN as u8,
                size: self.bytes.len(),
                block,
                instr,
            });
        }
        self.bytes[start..end].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_then_load_roundtrip() {
        let mut mem = Memory::new(64);
        mem.store_u64(8, 0xDEAD_BEEF, 0, 0).unwrap();
        assert_eq!(mem.load_u64(8, 0, 0).unwrap(), 0xDEAD_BEEF);
    }

    #[test]
    fn out_of_bounds_load_traps() {
        let mem = Memory::new(16);
        let err = mem.load_u64(9, 0, 0).unwrap_err();
        matches!(err, Trap::OutOfBoundsMemory { .. });
    }

    #[test]
    fn out_of_bounds_store_traps() {
        let mut mem = Memory::new(16);
        let err = mem.store_u64(1000, 1, 0, 0).unwrap_err();
        matches!(err, Trap::OutOfBoundsMemory { .. });
    }
}
