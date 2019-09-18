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
/// Opcode of STA _addr_
pub const STA: u8 = 0x10;
/// Opcode of LDA _addr_
pub const LDA: u8 = 0x20;
/// Opcode of ADD _addr_
pub const ADD: u8 = 0x30;
/// Opcode of OR _addr_
pub const OR: u8 = 0x40;
/// Opcode of AND _addr_
pub const AND: u8 = 0x50;
/// Opcode of NOT
pub const NOT: u8 = 0x60;
/// Opcode of SUB _addr_
pub const SUB: u8 = 0x70;
/// Opcode of JMP _addr_
pub const JMP: u8 = 0x80;
/// Opcode of JN _addr_
pub const JN: u8 = 0x90;
/// Opcode of JP _addr_
pub const JP: u8 = 0x94;
/// Opcode of JV _addr_
pub const JV: u8 = 0x98;
/// Opcode of JNV _addr_
pub const JNV: u8 = 0x9C;
/// Opcode of JZ _addr_
pub const JZ: u8 = 0xA0;
/// Opcode of JNZ _addr_
pub const JNZ: u8 = 0xA4;
/// Opcode of JC _addr_
pub const JC: u8 = 0xB0;
/// Opcode of JNC _addr_
pub const JNC: u8 = 0xB4;
/// Opcode of JB _addr_
pub const JB: u8 = 0xB8;
/// Opcode of JNB _addr_
pub const JNB: u8 = 0xBC;
/// Opcode of SHR
pub const SHR: u8 = 0xE0;
/// Opcode of SHL
pub const SHL: u8 = 0xE1;
/// Opcode of ROR
pub const ROR: u8 = 0xE2;
/// Opcode of ROL
pub const ROL: u8 = 0xE3;
/// Opcode of HLT
pub const HLT: u8 = 0xF0;

const MEM_HEADER: [u8; 4] = [0x03, 0x41, 0x48, 0x4D];
const STATE_HEADER: [u8; 4] = [0x04, 0x41, 0x48, 0x4D];

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
    pub operand: bool,
}

impl InstrInfo {
    pub fn new(opcode: u8) -> Option<Self> {
        match (opcode & 0xF0, opcode & 0xFC, opcode) {
            (NOP, ..) => Some(Self { mnemonic: "NOP", operand: false }),
            (STA, ..) => Some(Self { mnemonic: "STA", operand: true }),
            (LDA, ..) => Some(Self { mnemonic: "LDA", operand: true }),
            (ADD, ..) => Some(Self { mnemonic: "ADD", operand: true }),
            (OR, ..) => Some(Self { mnemonic: "OR", operand: true }),
            (AND, ..) => Some(Self { mnemonic: "AND", operand: true }),
            (NOT, ..) => Some(Self { mnemonic: "NOT", operand: false }),
            (SUB, ..) => Some(Self { mnemonic: "SUB", operand: true }),
            (_, JMP, _) => Some(Self { mnemonic: "JMP", operand: true }),
            (_, JN, _) => Some(Self { mnemonic: "JN", operand: true }),
            (_, JP, _) => Some(Self { mnemonic: "JP", operand: true }),
            (_, JV, _) => Some(Self { mnemonic: "JV", operand: true }),
            (_, JNV, _) => Some(Self { mnemonic: "JNV", operand: true }),
            (_, JZ, _) => Some(Self { mnemonic: "JZ", operand: true }),
            (_, JNZ, _) => Some(Self { mnemonic: "JNZ", operand: true }),
            (_, JC, _) => Some(Self { mnemonic: "JC", operand: true }),
            (_, JNC, _) => Some(Self { mnemonic: "JNC", operand: true }),
            (_, JB, _) => Some(Self { mnemonic: "JB", operand: true }),
            (_, JNB, _) => Some(Self { mnemonic: "JNB", operand: true }),
            (_, _, SHR) => Some(Self { mnemonic: "SHR", operand: false }),
            (_, _, SHL) => Some(Self { mnemonic: "SHL", operand: false }),
            (_, _, ROR) => Some(Self { mnemonic: "ROR", operand: false }),
            (_, _, ROL) => Some(Self { mnemonic: "ROL", operand: false }),
            (HLT, ..) => Some(Self { mnemonic: "HLT", operand: false }),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Machine {
    ri: u8,
    pc: u8,
    ac: u8,
    overflow: bool,
    carry: bool,
    borrow: bool,
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
        match (self.ri & 0xF0, self.ri & 0xFC, self.ri) {
            (NOP, ..) => self.exec_nop(),
            (STA, ..) => self.exec_sta(),
            (LDA, ..) => self.exec_lda(),
            (ADD, ..) => self.exec_add(),
            (OR, ..) => self.exec_or(),
            (AND, ..) => self.exec_and(),
            (NOT, ..) => self.exec_not(),
            (SUB, ..) => self.exec_sub(),
            (_, JMP, _) => self.exec_jmp(),
            (_, JN, _) => self.exec_jn(),
            (_, JP, _) => self.exec_jp(),
            (_, JV, _) => self.exec_jv(),
            (_, JNV, _) => self.exec_jnv(),
            (_, JZ, _) => self.exec_jz(),
            (_, JNZ, _) => self.exec_jnz(),
            (_, JC, _) => self.exec_jc(),
            (_, JNC, _) => self.exec_jnc(),
            (_, JB, _) => self.exec_jb(),
            (_, JNB, _) => self.exec_jnb(),
            (_, _, SHR) => self.exec_shr(),
            (_, _, SHL) => self.exec_shl(),
            (_, _, ROR) => self.exec_ror(),
            (_, _, ROL) => self.exec_rol(),
            (HLT, ..) => self.exec_hlt(),
            _ => (),
        }
    }

    fn exec_nop(&mut self) {}

    fn exec_sta(&mut self) {
        self.fetch();
        self.write(self.ri, self.ac);
    }

    fn exec_lda(&mut self) {
        self.fetch();
        self.ac = self.read(self.ri);
    }

    fn exec_add(&mut self) {
        self.fetch();
        let operand = self.read(self.ri);
        let (result, carry) = self.ac.overflowing_add(operand);
        let (_, overflow) = (self.ac as i8).overflowing_add(operand as i8);
        self.ac = result;
        self.overflow = overflow;
        self.carry = carry;
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

    fn exec_sub(&mut self) {
        self.fetch();
        let operand = self.read(self.ri);
        let (result, borrow) = self.ac.overflowing_sub(operand);
        let (_, overflow) = (self.ac as i8).overflowing_sub(operand as i8);
        self.ac = result;
        self.overflow = overflow;
        self.borrow = borrow;
    }

    fn exec_jmp(&mut self) {
        self.fetch();
        self.pc = self.ri;
    }

    fn exec_jn(&mut self) {
        self.fetch();
        if self.ac & 0x80 != 0 {
            self.pc = self.ri;
        }
    }

    fn exec_jp(&mut self) {
        self.fetch();
        if self.ac & 0x80 == 0 {
            self.pc = self.ri;
        }
    }

    fn exec_jv(&mut self) {
        self.fetch();
        if self.overflow {
            self.pc = self.ri;
        }
    }

    fn exec_jnv(&mut self) {
        self.fetch();
        if !self.overflow {
            self.pc = self.ri;
        }
    }

    fn exec_jz(&mut self) {
        self.fetch();
        if self.ac == 0 {
            self.pc = self.ri;
        }
    }

    fn exec_jnz(&mut self) {
        self.fetch();
        if self.ac != 0 {
            self.pc = self.ri;
        }
    }

    fn exec_jc(&mut self) {
        self.fetch();
        if self.carry {
            self.pc = self.ri;
        }
    }

    fn exec_jnc(&mut self) {
        self.fetch();
        if !self.carry {
            self.pc = self.ri;
        }
    }

    fn exec_jb(&mut self) {
        self.fetch();
        if self.borrow {
            self.pc = self.ri;
        }
    }

    fn exec_jnb(&mut self) {
        self.fetch();
        if !self.borrow {
            self.pc = self.ri;
        }
    }

    fn exec_shr(&mut self) {
        self.carry = self.ac & 0x1 != 0;
        self.ac >>= 1;
    }

    fn exec_shl(&mut self) {
        self.carry = self.ac & 0x80 != 0;
        self.ac <<= 1;
    }

    fn exec_ror(&mut self) {
        let prev_carry = if self.carry { 0x80 } else { 0x0 };
        self.carry = self.ac & 0x1 != 0;
        self.ac >>= 1;
        self.ac |= prev_carry;
    }

    fn exec_rol(&mut self) {
        let prev_carry = if self.carry { 0x1 } else { 0x0 };
        self.carry = self.ac & 0x80 != 0;
        self.ac <<= 1;
        self.ac |= prev_carry;
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

        output.write_all(&[self.ri, self.pc, self.ac])?;
        output.write_all(&[if self.carry { 1 } else { 0 }])?;
        output.write_all(&[if self.borrow { 1 } else { 0 }])?;
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
        self.ac = buf[6];
        self.carry = buf[7] != 0;
        input.read_exact(&mut buf[.. 2])?;
        self.borrow = buf[0] != 0;
        self.cycling = buf[1] != 0;

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
        let flag_n = self.ac >> 7;
        let flag_z = if self.ac == 0 { 1 } else { 0 };
        let flag_v = if self.overflow { 1 } else { 0 };
        let flag_c = if self.carry { 1 } else { 0 };
        let flag_b = if self.borrow { 1 } else { 0 };

        if hex {
            write!(output, "ac = {:02X}\n", self.ac)?;
            write!(output, "pc = {:02X}\n", self.pc)?;
            write!(output, "n  = {:02X}\n", flag_n)?;
            write!(output, "z  = {:02X}\n", flag_z)?;
            write!(output, "v  = {:02X}\n", flag_v)?;
            write!(output, "c  = {:02X}\n", flag_c)?;
            write!(output, "b  = {:02X}\n", flag_b)?;
        } else {
            write!(output, "ac = {:03}\n", self.ac)?;
            write!(output, "pc = {:03}\n", self.pc)?;
            write!(output, "n  = {:03}\n", flag_n)?;
            write!(output, "z  = {:03}\n", flag_z)?;
            write!(output, "v  = {:02X}\n", flag_v)?;
            write!(output, "c  = {:02X}\n", flag_c)?;
            write!(output, "b  = {:02X}\n", flag_b)?;
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
            overflow: false,
            carry: false,
            borrow: false,
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
            && self.overflow == other.overflow
            && self.carry == other.carry
            && self.cycling == other.cycling
            && self.cycles == other.cycles
            && self.accesses == other.accesses
            && &self.mem as &[u8] == &other.mem as &[u8]
    }
}

impl Eq for Machine {}
