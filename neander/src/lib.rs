#[cfg(test)]
mod test;

use std::{fmt, io::{self, Read, Write}};

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

const HEADER: [u8; 4] = [0x03, 0x4E, 0x44, 0x52];

#[derive(Clone)]
pub struct Machine {
    ri: u8,
    pc: u8,
    ra: u8,
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
        self.ra = self.read(self.ri);
    }

    fn exec_sta(&mut self) {
        self.fetch();
        self.write(self.ri, self.ra);
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

    fn exec_jz(&mut self) {
        self.fetch();
        if self.ra == 0 {
            self.pc = self.ri;
        }
    }

    fn exec_jn(&mut self) {
        self.fetch();
        if self.ra & 0x80 != 0 {
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
        W: Write
    {
        output.write_all(&HEADER)?;

        for &byte in self.mem.iter() {
            output.write_all(&[byte, 0x00])?;
        }

        Ok(())
    }

    fn load_mem<R>(&mut self, mut input: R) -> io::Result<()>
    where
        R: Read
    {
        let mut header = [0; 4]; 

        input.read_exact(&mut header)?;
        
        if header != HEADER {
            make_error()?;
        }

        for byte in self.mem.iter_mut() {
            let mut buf = [0; 2];
            input.read_exact(&mut buf)?;
            *byte = buf[0];
        }

        Ok(())
    }

    fn save_state<W>(&self, mut output: W) -> io::Result<()>
    where
        W: Write
    {
        
        output.write_all(&HEADER)?;

        output.write_all(&[self.ri, self.pc, self.ra])?;
        output.write_all(&[if self.cycling { 1 } else { 0 }])?;
        output.write_all(&self.cycles.to_le_bytes())?;
        output.write_all(&self.accesses.to_le_bytes())?;

        for &byte in self.mem.iter() {
            output.write_all(&[byte, 0x00])?;
        }

        Ok(())
    }

    fn load_state<R>(&mut self, input: R) -> io::Result<()>
    where
        R: Read
    {
        unimplemented!()
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("neander::Machine")
            .field("ri", &self.ri)
            .field("pc", &self.pc)
            .field("ra", &self.ra)
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
            mem: [0; 256],
            cycling: false,
            cycles: 0,
            accesses: 0,
        }
    }
}
