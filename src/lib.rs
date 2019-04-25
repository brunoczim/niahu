use std::{fmt, io};

pub const NOP: u8 = 0x0;
pub const LDA: u8 = 0x10;
pub const STA: u8 = 0x20;
pub const ADD: u8 = 0x30;
pub const OR: u8 = 0x40;
pub const AND: u8 = 0x50;
pub const NOT: u8 = 0x60;
pub const JMP: u8 = 0x80;
pub const JN: u8 = 0x90;
pub const JZ: u8 = 0xA0;
pub const HLT: u8 = 0xF0;

pub const CYCLES_PER_ASYNC_CALL: usize = 100;

pub const HEADER: [u8; 4] = [0x03, 0x4E, 0x44, 0x52];

#[derive(Debug, Clone, Default)]
struct Stats {
    running: bool,
    cycles: usize,
    accesses: usize,
}

/// A neander machine, with information needed for the simulator.
#[derive(Clone)]
pub struct Machine {
    /// The accumulator. The register used for implicit operands.
    pub ac: u8,
    /// Neander memory. Direct access to memory will not count for statistics.
    pub mem: [u8; 256],
    pc: u8,
    stats: Stats,
}

impl Machine {
    /// Creates a new zeroed machine.
    pub fn new() -> Self {
        Self { pc: 0, ac: 0, mem: [0; 256], stats: Stats::default() }
    }

    /// Resets machine statistics such as instruction cycles and memory
    /// accesses.
    pub fn reset_stats(&mut self) {
        self.stats.accesses = 0;
        self.stats.cycles = 0;
    }

    /// Resets machine statistics and also sets program counter (PC) to a new
    /// given value.
    pub fn reset_with_pc(&mut self, new_pc: u8) {
        self.reset_stats();
        self.pc = new_pc;
    }

    /// Returns the current value of program counter (PC).
    pub fn pc(&self) -> u8 {
        self.pc
    }

    /// Counts how much memory accesses happened since an execution began.
    pub fn accesses(&self) -> usize {
        self.stats.accesses
    }

    /// Counts how much fetch-decode-execute cycles happened since an execution
    /// began.
    pub fn cycles(&self) -> usize {
        self.stats.cycles
    }

    /// Reads a byte from an address and update statistics on memory accesses.
    pub fn read(&mut self, ptr: u8) -> u8 {
        self.stats.accesses += 1;
        self.mem[ptr as usize]
    }

    /// Writes a byte into an address and update statistics on memory accesses.
    pub fn write(&mut self, ptr: u8, val: u8) {
        self.stats.accesses += 1;
        self.mem[ptr as usize] = val;
    }

    /// Reads a byte from current PC address, handles statistics on memory
    /// access, and also increments PC.
    pub fn read_code(&mut self) -> u8 {
        let byte = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    /// Stops execution when not done step-by-step.
    pub fn halt(&mut self) {
        self.stats.running = false;
    }

    /// Performs a whole fetch-decode-execute cycle. Statistics about code
    /// execution are handled.
    pub fn cycle(&mut self) {
        self.stats.running = true;

        self.stats.cycles += 1;

        let opcode = self.read_code() & 0xF0;
        match opcode {
            HLT => self.halt(),

            NOT => self.ac = !self.ac,

            LDA => {
                let addr = self.read_code();
                self.ac = self.read(addr);
            },

            STA => {
                let addr = self.read_code();
                self.write(addr, self.ac);
            },

            ADD => {
                let addr = self.read_code();
                self.ac = self.ac.wrapping_add(self.read(addr));
            },

            OR => {
                let addr = self.read_code();
                self.ac |= self.read(addr);
            },

            AND => {
                let addr = self.read_code();
                self.ac &= self.read(addr);
            },

            JMP => self.pc = self.read_code(),

            JN => {
                let addr = self.read_code();
                if self.ac & 0x80 != 0 {
                    self.pc = addr;
                }
            },

            JZ => {
                let addr = self.read_code();
                if self.ac == 0 {
                    self.pc = addr;
                }
            },

            _ => (),
        }
    }

    /// Executes a whole algorithm in a single call: this method will only stop
    /// when HLT is found. Note that infinite loops will freeze everything.
    pub fn execute_sync(&mut self) {
        loop {
            self.cycle();

            if !self.stats.running {
                break;
            }
        }
    }

    /// Executes a few cycles of an algorithm. This should be called if
    /// asynchronous execution is being coded. A boolean is returned indicating
    /// whether HLT was found.
    pub fn execute_async_round(&mut self) -> bool {
        for _ in 0 .. CYCLES_PER_ASYNC_CALL {
            self.cycle();

            if !self.stats.running {
                break;
            }
        }

        !self.stats.running
    }

    /// Encodes the memory into a file or any writable IO device.
    pub fn encode<W>(&self, mut output: W) -> io::Result<()>
    where
        W: io::Write,
    {
        output.write_all(&HEADER)?;

        for &byte in self.mem.iter() {
            output.write_all(&[byte, 0x0])?;
        }

        Ok(())
    }

    /// Decodes the memory from a file or any readable IO device.
    pub fn decode<R>(&mut self, mut input: R) -> io::Result<()>
    where
        R: io::Read,
    {
        let mut header = [0; 4];
        input.read_exact(&mut header)?;

        if header != HEADER {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof))?;
        }

        for byte in self.mem.iter_mut() {
            let mut buf = [0; 2];
            input.read_exact(&mut buf)?;
            *byte = buf[0];
        }

        Ok(())
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmtr,
            "Machine {{ pc: {:?}, ac: {:?}, mem: {:?}, stats: {:?} }}",
            self.pc, self.ac, &self.mem as &[u8], &self.stats
        )
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sub_algo() {
        let mut vm = Machine::new();
        vm.mem[0x0] = LDA;
        vm.mem[0x1] = 0x81;
        vm.mem[0x2] = NOT;
        vm.mem[0x3] = ADD;
        vm.mem[0x4] = 0x83;
        vm.mem[0x5] = ADD;
        vm.mem[0x6] = 0x80;
        vm.mem[0x7] = STA;
        vm.mem[0x8] = 0x82;
        vm.mem[0x9] = HLT;

        vm.mem[0x80] = 150;
        vm.mem[0x81] = 3;
        vm.mem[0x83] = 1;

        vm.execute();

        assert_eq!(vm.mem[0x82], 147);
        assert_eq!(vm.cycles(), 6);
        assert_eq!(vm.accesses(), 14);
    }

    #[test]
    fn mul_algo() {
        let mut vm = Machine::new();
        vm.mem[0x0] = LDA;
        vm.mem[0x1] = 0x85;
        vm.mem[0x2] = STA;
        vm.mem[0x3] = 0x82;
        vm.mem[0x4] = LDA;
        vm.mem[0x5] = 0x81;
        vm.mem[0x6] = STA;
        vm.mem[0x7] = 0x83;
        vm.mem[0x8] = JZ;
        vm.mem[0x9] = 0x18;
        vm.mem[0xA] = ADD;
        vm.mem[0xB] = 0x84;
        vm.mem[0xC] = STA;
        vm.mem[0xD] = 0x83;
        vm.mem[0xE] = LDA;
        vm.mem[0xF] = 0x80;
        vm.mem[0x10] = ADD;
        vm.mem[0x11] = 0x82;
        vm.mem[0x12] = STA;
        vm.mem[0x13] = 0x82;
        vm.mem[0x14] = LDA;
        vm.mem[0x15] = 0x83;
        vm.mem[0x16] = JMP;
        vm.mem[0x17] = 0x8;
        vm.mem[0x18] = HLT;

        vm.mem[0x80] = 5;
        vm.mem[0x81] = 11;
        vm.mem[0x84] = 255;
        vm.mem[0x85] = 0;

        vm.execute();

        assert_eq!(vm.mem[0x82], 55);
        assert_eq!(vm.cycles(), 94);
        assert_eq!(vm.accesses(), 257);
    }

    #[test]
    fn is_pos() {
        let mut vm = Machine::new();
        vm.mem[0x0] = LDA;
        vm.mem[0x1] = 0x80;
        vm.mem[0x2] = NOT;
        vm.mem[0x3] = JN;
        vm.mem[0x4] = 0xA;
        vm.mem[0x5] = LDA;
        vm.mem[0x6] = 0x83;
        vm.mem[0x7] = STA;
        vm.mem[0x8] = 0x81;
        vm.mem[0x9] = HLT;
        vm.mem[0xA] = LDA;
        vm.mem[0xB] = 0x82;
        vm.mem[0xC] = STA;
        vm.mem[0xD] = 0x81;
        vm.mem[0xE] = HLT;

        vm.mem[0x80] = 128;
        vm.mem[0x82] = 1;
        vm.mem[0x83] = 0;

        vm.execute();

        assert_eq!(vm.mem[0x81], 0);
    }
}
