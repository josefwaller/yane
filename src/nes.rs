/// The NES.
pub struct Nes {
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

impl Nes {
    pub fn new() -> Nes {
        Nes {
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
    use super::Nes;

    #[test]
    fn test_init() {
        // Should not throw
        Nes::new();
    }
}
