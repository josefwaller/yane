/// The CPU of the NES.
/// Holds the CPU registers and performs all logic to update the state.
pub struct Cpu {
    /// Accumulator
    a: u8,
    /// X index registers
    x: u8,
    /// Y index registers
    y: u8,
    /// Program counter
    p_c: u16,
    /// Stack pointer
    s_p: u8,
    /// Status register
    p: u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0x0,
            x: 0x0,
            y: 0x0,
            p_c: 0x00,
            s_p: 0x0,
            p: 0x0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;

    #[test]
    fn test_init() {
        // Should not throw
        Cpu::new();
    }
}
