#[cfg(test)]
mod test;

use std::{
    fmt,
    io::{self, Read, Write},
};

/// Opcode of NOP
pub const NOP: u8 = 0x0;
/// Opcode of LDA _addr_
pub const LDA: u8 = 0x10;
/// Opcode of STA _addr_
pub const STA: u8 = 0x20;
/// Opcode of ADD _addr_
pub const ADD: u8 = 0x30;
/// Opcode of OR _addr_
pub const OR: u8 = 0x40;
/// Opcode of AND _addr_
pub const AND: u8 = 0x50;
/// Opcode of NOT
pub const NOT: u8 = 0x60;
/// Opcode of JMP _addr_
pub const JMP: u8 = 0x80;
/// Opcode of JN _addr_
pub const JN: u8 = 0x90;
/// Opcode of JZ _addr_
pub const JZ: u8 = 0xA0;
/// Opcode of HLT
pub const HLT: u8 = 0xF0;

const MEM_HEADER: [u8; 4] = [0x03, 0x4E, 0x44, 0x52];
const STATE_HEADER: [u8; 4] = [0x04, 0x4E, 0x44, 0x52];

#[derive(Debug, Clone, Copy)]
pub struct InstrInfo {
    pub mnemonic: &'static str,
    pub needs_operand: bool,
}

impl InstrInfo {
    pub fn new(opcode: u8) -> Option<Self> {
        match opcode {
            NOP => Some(Self { mnemonic: "NOP", needs_operand: false }),
            LDA => Some(Self { mnemonic: "LDA", needs_operand: true }),
            STA => Some(Self { mnemonic: "STA", needs_operand: true }),
            ADD => Some(Self { mnemonic: "ADD", needs_operand: true }),
            OR => Some(Self { mnemonic: "OR", needs_operand: true }),
            AND => Some(Self { mnemonic: "AND", needs_operand: true }),
            NOT => Some(Self { mnemonic: "NOT", needs_operand: false }),
            JMP => Some(Self { mnemonic: "JMP", needs_operand: true }),
            JZ => Some(Self { mnemonic: "JZ", needs_operand: true }),
            JN => Some(Self { mnemonic: "JN", needs_operand: true }),
            HLT => Some(Self { mnemonic: "HLT", needs_operand: false }),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Machine {
    ri: u8,
    pc: u8,
    ac: u8,
    mem: [u8; 256],
    cycling: bool,
    cycles: u64,
    accesses: u64,
}

impl Machine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, addr: u8) -> u8 {
        self.accesses = self.accesses.saturating_add(1);
        self.mem[addr as usize]
    }

    pub fn write(&mut self, addr: u8, byte: u8) {
        self.accesses = self.accesses.saturating_add(1);
        self.mem[addr as usize] = byte;
    }

    pub fn cycle(&mut self) {
        self.cycles += 1;
        self.fetch();
        self.decode_exec();
    }

    pub fn fetch(&mut self) {
        self.ri = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn decode_exec(&mut self) {
        match self.ri & 0xF0 {
            NOP => self.exec_nop(),
            LDA => self.exec_lda(),
            STA => self.exec_sta(),
            ADD => self.exec_add(),
            OR => self.exec_or(),
            AND => self.exec_and(),
            NOT => self.exec_not(),
            JMP => self.exec_jmp(),
            JZ => self.exec_jz(),
            JN => self.exec_jn(),
            HLT => self.exec_hlt(),
            _ => (),
        }
    }

    fn exec_nop(&mut self) {}

    fn exec_lda(&mut self) {
        self.fetch();
        self.ac = self.read(self.ri);
    }

    fn exec_sta(&mut self) {
        self.fetch();
        self.write(self.ri, self.ac);
    }

    fn exec_add(&mut self) {
        self.fetch();
        self.ac = self.ac.wrapping_add(self.read(self.ri));
    }

    fn exec_or(&mut self) {
        self.fetch();
        self.ac |= self.read(self.ri);
    }

    fn exec_and(&mut self) {
        self.fetch();
        self.ac &= self.read(self.ri);
    }

    fn exec_not(&mut self) {
        self.ac = !self.ac;
    }

    fn exec_jmp(&mut self) {
        self.fetch();
        self.pc = self.ri;
    }

    fn exec_jz(&mut self) {
        self.fetch();
        if self.ac == 0 {
            self.pc = self.ri;
        }
    }

    fn exec_jn(&mut self) {
        self.fetch();
        if self.ac & 0x80 != 0 {
            self.pc = self.ri;
        }
    }

    fn exec_hlt(&mut self) {
        self.cycling = false;
    }
}

fn make_error<T>() -> io::Result<T> {
    let err = "Invalid or corrupted file";
    Err(io::Error::new(io::ErrorKind::InvalidData, err))
}

impl common::Machine for Machine {
    type Word = u8;

    fn step(&mut self) {
        self.cycle();
    }

    fn execute(&mut self) {
        self.cycling = true;
        while self.cycling {
            self.cycle();
        }
    }

    fn save_mem<W>(&self, mut output: W) -> io::Result<()>
    where
        W: Write,
    {
        output.write_all(&MEM_HEADER)?;

        for &byte in self.mem.iter() {
            output.write_all(&[byte, 0x00])?;
        }

        Ok(())
    }

    fn load_mem<R>(&mut self, mut input: R) -> io::Result<()>
    where
        R: Read,
    {
        let mut buf = [0; 4];

        input.read_exact(&mut buf)?;

        if buf != MEM_HEADER {
            make_error()?;
        }

        for byte in self.mem.iter_mut() {
            input.read_exact(&mut buf[.. 2])?;
            *byte = buf[0];
        }

        Ok(())
    }

    fn save_state<W>(&self, mut output: W) -> io::Result<()>
    where
        W: Write,
    {
        output.write_all(&STATE_HEADER)?;

        output.write_all(&[self.ri, self.pc, self.ac])?;
        output.write_all(&[if self.cycling { 1 } else { 0 }])?;
        output.write_all(&self.cycles.to_le_bytes())?;
        output.write_all(&self.accesses.to_le_bytes())?;

        for &byte in self.mem.iter() {
            output.write_all(&[byte, 0x00])?;
        }

        Ok(())
    }

    fn load_state<R>(&mut self, mut input: R) -> io::Result<()>
    where
        R: Read,
    {
        let mut buf = [0; 8];

        input.read_exact(&mut buf)?;

        if &buf[.. 4] != &STATE_HEADER {
            make_error()?;
        }

        self.ri = buf[4];
        self.pc = buf[5];
        self.ac = buf[6];
        self.cycling = buf[7] != 0;

        input.read_exact(&mut buf)?;
        self.cycles = u64::from_le_bytes(buf);

        input.read_exact(&mut buf)?;
        self.accesses = u64::from_le_bytes(buf);

        for byte in self.mem.iter_mut() {
            input.read_exact(&mut buf[.. 2])?;
            *byte = buf[0];
        }

        Ok(())
    }

    fn display_mem_data<W, B>(
        &mut self,
        bounds: B,
        mut output: W,
        hex: bool,
    ) -> io::Result<()>
    where
        B: IntoIterator<Item = u8>,
        W: Write,
    {
        for addr in bounds {
            if hex {
                write!(
                    output,
                    "{:02X} = {:02X}\n",
                    addr, self.mem[addr as usize]
                )?
            } else {
                write!(
                    output,
                    "{:03} = {:03}\n",
                    addr, self.mem[addr as usize]
                )?
            }
        }

        Ok(())
    }

    fn display_mem_opcodes<W, B>(
        &mut self,
        bounds: B,
        mut output: W,
        hex: bool,
    ) -> io::Result<()>
    where
        B: IntoIterator<Item = u8>,
        W: Write,
    {
        let mut needs_operand = false;

        for addr in bounds {
            if hex {
                write!(
                    output,
                    "{:02X} = {:02X}",
                    addr, self.mem[addr as usize]
                )?
            } else {
                write!(output, "{:03} = {:03}", addr, self.mem[addr as usize])?
            }

            if needs_operand {
                needs_operand = false;
            } else {
                if let Some(info) = InstrInfo::new(self.mem[addr as usize]) {
                    needs_operand = info.needs_operand;
                    write!(output, "{:^8}", info.mnemonic)?
                }
            }
        }

        Ok(())
    }

    fn display_registers<W>(
        &mut self,
        mut output: W,
        hex: bool,
    ) -> io::Result<()>
    where
        W: Write,
    {
        if hex {
            write!(output, "ac = {:02X}\n", self.ac)?;
            write!(output, "pc = {:02X}\n", self.pc)?;
        } else {
            write!(output, "ac = {:03}\n", self.ac)?;
            write!(output, "pc = {:03}\n", self.pc)?;
        }

        Ok(())
    }

    fn write_raw(&mut self, addr: u8, byte: u8) {
        self.mem[addr as usize] = byte;
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("neander::Machine")
            .field("ri", &self.ri)
            .field("pc", &self.pc)
            .field("ac", &self.ac)
            .field("mem", &(&self.mem as &[u8]))
            .field("cycling", &self.cycling)
            .field("cycles", &self.cycles)
            .field("accesses", &self.accesses)
            .finish()
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            ri: 0,
            pc: 0,
            ac: 0,
            mem: [0; 256],
            cycling: false,
            cycles: 0,
            accesses: 0,
        }
    }
}

impl PartialEq for Machine {
    fn eq(&self, other: &Self) -> bool {
        self.ri == other.ri
            && self.pc == other.pc
            && self.ac == other.ac
            && self.cycling == other.cycling
            && self.cycles == other.cycles
            && self.accesses == other.accesses
            && &self.mem as &[u8] == &other.mem as &[u8]
    }
}

impl Eq for Machine {}
