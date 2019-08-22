use std::io::{self, Read, Write};

pub trait Machine: Default {
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
}
