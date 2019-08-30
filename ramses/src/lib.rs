#[cfg(test)]
mod test;

use error::{Fallible, InvalidFile, WithPath};
use std::{
    fmt,
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// Opcode of NOP
pub const NOP: u8 = 0x0;
/// Opcode of STA _reg_, _am_
pub const STA: u8 = 0x10;
/// Opcode of LDA _reg_, _am_
pub const LDA: u8 = 0x20;
/// Opcode of ADD _reg_, _am_
pub const ADD: u8 = 0x30;
/// Opcode of OR _reg_, _am_
pub const OR: u8 = 0x40;
/// Opcode of AND _reg_, _am_
pub const AND: u8 = 0x50;
/// Opcode of NOT _reg_
pub const NOT: u8 = 0x60;
/// Opcode of JMP _am_
pub const JMP: u8 = 0x80;
/// Opcode of JN _am_
pub const JN: u8 = 0x90;
/// Opcode of JZ _am_
pub const JZ: u8 = 0xA0;
/// Opcode of JC _am_
pub const JC: u8 = 0xB0;
/// Opcode of JSR _am_
pub const JSR: u8 = 0xC0;
/// Opcode of NEG _reg_
pub const NEG: u8 = 0xD0;
/// Opcode of SHR _reg_
pub const SHR: u8 = 0xE0;
/// Opcode of HLT
pub const HLT: u8 = 0xF0;

const MEM_HEADER: [u8; 4] = [0x03, 0x52, 0x4D, 0x53];
const STATE_HEADER: [u8; 4] = [0x04, 0x52, 0x4D, 0x53];

pub fn is_mem_file<P>(path: &P) -> bool
where
    P: AsRef<Path> + ?Sized,
{
    path.as_ref().extension().map_or(false, |ext| ext == "mem")
}

pub fn is_state_file<P>(path: &P) -> bool
where
    P: AsRef<Path> + ?Sized,
{
    path.as_ref().extension().map_or(false, |ext| ext == "state")
}

#[derive(Debug, Clone, Copy)]
pub struct InstrInfo {
    pub mnemonic: &'static str,
    pub register: bool,
    pub operand: bool,
}

impl InstrInfo {
    pub fn new(opcode: u8) -> Option<Self> {
        match opcode & 0xF0 {
            NOP => {
                Some(Self { mnemonic: "NOP", register: false, operand: false })
            },
            STA => {
                Some(Self { mnemonic: "STA", register: true, operand: true })
            },
            LDA => {
                Some(Self { mnemonic: "LDA", register: true, operand: true })
            },
            ADD => {
                Some(Self { mnemonic: "ADD", register: true, operand: true })
            },
            OR => Some(Self { mnemonic: "OR", register: true, operand: true }),
            AND => {
                Some(Self { mnemonic: "AND", register: true, operand: true })
            },
            NOT => {
                Some(Self { mnemonic: "NOT", register: true, operand: false })
            },
            JMP => {
                Some(Self { mnemonic: "JMP", register: false, operand: true })
            },
            JN => Some(Self { mnemonic: "JN", register: false, operand: true }),
            JZ => Some(Self { mnemonic: "JZ", register: false, operand: true }),
            JC => Some(Self { mnemonic: "JC", register: false, operand: true }),
            JSR => {
                Some(Self { mnemonic: "JSR", register: false, operand: true })
            },
            NEG => {
                Some(Self { mnemonic: "NEG", register: true, operand: false })
            },
            SHR => {
                Some(Self { mnemonic: "SHR", register: true, operand: false })
            },
            HLT => {
                Some(Self { mnemonic: "HLT", register: false, operand: false })
            },
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Machine {
    ri: u8,
    pc: u8,
    ra: u8,
    rb: u8,
    rx: u8,
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

    pub fn write(&mut self, addr: u8, data: u8) {
        self.accesses = self.accesses.saturating_add(1);
        self.mem[addr as usize] = data;
    }

    pub fn set_pc(&mut self, data: u8) {
        self.pc = data;
        self.cycles = 0;
        self.accesses = 0;
    }

    pub fn write_raw(&mut self, addr: u8, data: u8) {
        self.mem[addr as usize] = data;
    }

    pub fn cycle(&mut self) {
        self.cycles += 1;
        self.fetch();
        self.decode_exec();
    }

    pub fn execute(&mut self) {
        self.cycling = true;
        while self.cycling {
            self.cycle();
        }
    }

    pub fn fetch(&mut self) {
        self.ri = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn decode_exec(&mut self) {
        match self.ri & 0xF0 {
            NOP => self.exec_nop(),
            STA => self.exec_sta(),
            LDA => self.exec_lda(),
            ADD => self.exec_add(),
            OR => self.exec_or(),
            AND => self.exec_and(),
            NOT => self.exec_not(),
            JMP => self.exec_jmp(),
            JN => self.exec_jn(),
            JZ => self.exec_jz(),
            HLT => self.exec_hlt(),
            _ => (),
        }
    }

    fn exec_nop(&mut self) {}

    fn exec_sta(&mut self) {
        self.fetch();
        self.write(self.ri, self.ra);
    }

    fn exec_lda(&mut self) {
        self.fetch();
        self.ra = self.read(self.ri);
    }

    fn exec_add(&mut self) {
        self.fetch();
        self.ra = self.ra.wrapping_add(self.read(self.ri));
    }

    fn exec_or(&mut self) {
        self.fetch();
        self.ra |= self.read(self.ri);
    }

    fn exec_and(&mut self) {
        self.fetch();
        self.ra &= self.read(self.ri);
    }

    fn exec_not(&mut self) {
        self.ra = !self.ra;
    }

    fn exec_jmp(&mut self) {
        self.fetch();
        self.pc = self.ri;
    }

    fn exec_jn(&mut self) {
        self.fetch();
        if self.ra & 0x80 != 0 {
            self.pc = self.ri;
        }
    }

    fn exec_jz(&mut self) {
        self.fetch();
        if self.ra == 0 {
            self.pc = self.ri;
        }
    }

    fn exec_hlt(&mut self) {
        self.cycling = false;
    }

    pub fn save_mem<W>(&self, mut output: W) -> Fallible<()>
    where
        W: Write,
    {
        output.write_all(&MEM_HEADER)?;

        for &byte in self.mem.iter() {
            output.write_all(&[byte, 0x00])?;
        }

        Ok(())
    }

    pub fn load_mem<R>(&mut self, mut input: R) -> Fallible<()>
    where
        R: Read,
    {
        let mut buf = [0; 4];

        input.read_exact(&mut buf)?;

        if buf != MEM_HEADER {
            Err(InvalidFile)?;
        }

        for byte in self.mem.iter_mut() {
            input.read_exact(&mut buf[.. 2])?;
            *byte = buf[0];
        }

        Ok(())
    }

    pub fn save_state<W>(&self, mut output: W) -> Fallible<()>
    where
        W: Write,
    {
        output.write_all(&STATE_HEADER)?;

        output.write_all(&[self.ri, self.pc, self.ra])?;
        output.write_all(&[if self.cycling { 1 } else { 0 }])?;
        output.write_all(&self.cycles.to_le_bytes())?;
        output.write_all(&self.accesses.to_le_bytes())?;

        for &byte in self.mem.iter() {
            output.write_all(&[byte, 0x00])?;
        }

        Ok(())
    }

    pub fn load_state<R>(&mut self, mut input: R) -> Fallible<()>
    where
        R: Read,
    {
        let mut buf = [0; 8];

        input.read_exact(&mut buf)?;

        if &buf[.. 4] != &STATE_HEADER {
            Err(InvalidFile)?;
        }

        self.ri = buf[4];
        self.pc = buf[5];
        self.ra = buf[6];
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

    pub fn save_at_path<P>(&self, path: &P) -> Fallible<()>
    where
        P: AsRef<Path> + ?Sized,
    {
        let file = File::create(path.as_ref())?;

        let res = if is_mem_file(path) {
            self.save_mem(file)
        } else if is_state_file(path) {
            self.save_state(file)
        } else {
            Err(InvalidFile.into())
        };

        res.map_err(|error| {
            WithPath { path: path.as_ref().into(), error }.into()
        })
    }

    pub fn load_from_path<P>(&mut self, path: &P) -> Fallible<()>
    where
        P: AsRef<Path> + ?Sized,
    {
        let file = File::open(path.as_ref())?;

        let res = if is_mem_file(path) {
            self.load_mem(file)
        } else if is_state_file(path) {
            self.load_state(file)
        } else {
            Err(InvalidFile.into())
        };

        res.map_err(|error| {
            WithPath { path: path.as_ref().into(), error }.into()
        })
    }

    pub fn display_mem_data<W, B>(
        &mut self,
        bounds: B,
        mut output: W,
        hex: bool,
    ) -> Fallible<()>
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

    pub fn display_mem_opcodes<W, B>(
        &mut self,
        bounds: B,
        mut output: W,
        hex: bool,
    ) -> Fallible<()>
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
                    needs_operand = info.operand;
                    write!(output, "  {}", info.mnemonic)?
                }
            }

            write!(output, "\n")?;
        }

        Ok(())
    }

    pub fn display_registers<W>(
        &mut self,
        mut output: W,
        hex: bool,
    ) -> Fallible<()>
    where
        W: Write,
    {
        let flag_n = self.ra >> 7;
        let flag_z = if self.ra == 0 { 1 } else { 0 };

        if hex {
            write!(output, "ra = {:02X}\n", self.ra)?;
            write!(output, "pc = {:02X}\n", self.pc)?;
            write!(output, "n  = {:02X}\n", flag_n)?;
            write!(output, "z  = {:02X}\n", flag_z)?;
        } else {
            write!(output, "ra = {:03}\n", self.ra)?;
            write!(output, "pc = {:03}\n", self.pc)?;
            write!(output, "n  = {:03}\n", flag_n)?;
            write!(output, "z  = {:03}\n", flag_z)?;
        }

        Ok(())
    }

    pub fn display_stats<W>(&mut self, mut output: W) -> Fallible<()>
    where
        W: Write,
    {
        write!(output, "cycles = {}\n", self.cycles)?;
        write!(output, "accesses = {}\n", self.accesses)?;

        Ok(())
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("neander::Machine")
            .field("ri", &self.ri)
            .field("pc", &self.pc)
            .field("ra", &self.ra)
            .field("rb", &self.rb)
            .field("rx", &self.rx)
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
            ra: 0,
            rb: 0,
            rx: 0,
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
            && self.ra == other.ra
            && self.cycling == other.cycling
            && self.cycles == other.cycles
            && self.accesses == other.accesses
            && &self.mem as &[u8] == &other.mem as &[u8]
    }
}

impl Eq for Machine {}
