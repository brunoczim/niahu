use std::io::{self, Read, Write};

pub trait Machine: Default {
    type Word;

    fn step(&mut self);

    fn execute(&mut self);

    fn save_mem<W>(&self, output: W) -> io::Result<()>
    where
        W: Write;

    fn load_mem<R>(&mut self, input: R) -> io::Result<()>
    where
        R: Read;

    fn save_state<W>(&self, output: W) -> io::Result<()>
    where
        W: Write;

    fn load_state<R>(&mut self, input: R) -> io::Result<()>
    where
        R: Read;

    fn display_mem_data<W, B>(
        &mut self,
        bounds: B,
        output: W,
        hex: bool,
    ) -> io::Result<()>
    where
        B: IntoIterator<Item = Self::Word>,
        W: Write;

    fn display_mem_opcodes<W, B>(
        &mut self,
        bounds: B,
        output: W,
        hex: bool,
    ) -> io::Result<()>
    where
        B: IntoIterator<Item = Self::Word>,
        W: Write;

    fn display_registers<W>(&mut self, output: W, hex: bool) -> io::Result<()>
    where
        W: Write;

    fn write_raw(&mut self, addr: Self::Word, byte: u8);
}
