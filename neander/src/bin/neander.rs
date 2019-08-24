use error::{Fallible, WithPath};
use std::{
    io,
    path::{Path, PathBuf},
    process,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "neander",
    about = "Multiplatform Simulator of Neander Hypothetical Machine"
)]
enum Command {
    /// Creates a new zeroed memory or a zeroed state
    #[structopt(name = "new")]
    New {
        #[structopt(short = "o", parse(from_os_str))]
        output: PathBuf,
    },

    /// Writes a byte into Program Counter register
    #[structopt(name = "write")]
    Write {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "o", parse(from_os_str))]
        output: Option<PathBuf>,
        #[structopt(short = "x")]
        hex: bool,
        #[structopt(short = "a")]
        addr: String,
        #[structopt(short = "d")]
        data: String,
    },

    /// Writes a byte into a given address
    #[structopt(name = "setpc")]
    SetPc {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "o", parse(from_os_str))]
        output: Option<PathBuf>,
        #[structopt(short = "x")]
        hex: bool,
        #[structopt(short = "d")]
        data: String,
    },

    /// Runs the code in a machine until HLT is found
    #[structopt(name = "run")]
    Run {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "o", parse(from_os_str))]
        output: Option<PathBuf>,
    },

    /// Runs only a few steps of the code in a machine
    #[structopt(name = "step")]
    Step {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "o", parse(from_os_str))]
        output: Option<PathBuf>,
        #[structopt(short = "n", default_value = "1")]
        steps: u64,
    },

    /// Shows a given range of memory data of a machine
    #[structopt(name = "data")]
    Data {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "x")]
        hex: bool,
        #[structopt(short = "s")]
        start: Option<String>,
        #[structopt(short = "e")]
        end: Option<String>,
    },

    /// Shows a range of memory data with mnemonics in a machine.
    #[structopt(name = "code")]
    Code {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "x")]
        hex: bool,
        #[structopt(short = "s")]
        start: Option<String>,
        #[structopt(short = "e")]
        end: Option<String>,
    },

    /// Shows register data of a machine (.state)
    #[structopt(name = "registers")]
    Regs {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "x")]
        hex: bool,
    },

    /// Shows statistics of code being run in a machine (.state)
    #[structopt(name = "stats")]
    Stats {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
    },
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        process::exit(-1);
    }
}

fn try_main() -> Fallible<()> {
    match Command::from_args() {
        Command::New { output } => subcommand_new(output),

        Command::Write { input, output, hex, addr, data } => {
            subcommand_write(input, output, hex, addr, data)
        },

        Command::SetPc { input, output, hex, data } => {
            subcommand_setpc(input, output, hex, data)
        },

        Command::Run { input, output } => subcommand_run(input, output),

        Command::Step { input, output, steps } => {
            subcommand_step(input, output, steps)
        },

        Command::Data { input, hex, start, end } => {
            subcommand_data(input, hex, start, end)
        },

        Command::Code { input, hex, start, end } => {
            subcommand_code(input, hex, start, end)
        },

        Command::Regs { input, hex } => subcommand_regs(input, hex),

        Command::Stats { input } => subcommand_stats(input),
    }
}

fn subcommand_new(output: PathBuf) -> Fallible<()> {
    let vm = neander::Machine::new();
    vm.save_at_path(&output)?;
    Ok(())
}

fn subcommand_write(
    input: PathBuf,
    output: Option<PathBuf>,
    hex: bool,
    addr: String,
    data: String,
) -> Fallible<()> {
    let addr = parse_dec_or_hex(&addr, hex)?;
    let data = parse_dec_or_hex(&data, hex)?;
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.write_raw(addr, data);
    vm.save_at_path(resolve_output(&input, &output))?;

    Ok(())
}

fn subcommand_setpc(
    input: PathBuf,
    output: Option<PathBuf>,
    hex: bool,
    data: String,
) -> Fallible<()> {
    let data = parse_dec_or_hex(&data, hex)?;
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.set_pc(data);
    vm.save_at_path(resolve_output(&input, &output))?;

    Ok(())
}

fn subcommand_run(input: PathBuf, output: Option<PathBuf>) -> Fallible<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.execute();
    vm.save_at_path(resolve_output(&input, &output))?;

    Ok(())
}

fn subcommand_step(
    input: PathBuf,
    output: Option<PathBuf>,
    steps: u64,
) -> Fallible<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    for _ in 0 .. steps {
        vm.cycle();
    }
    vm.save_at_path(resolve_output(&input, &output))?;

    Ok(())
}

fn subcommand_data(
    input: PathBuf,
    hex: bool,
    start: Option<String>,
    end: Option<String>,
) -> Fallible<()> {
    let start = start.map_or(Ok(128), |s| parse_dec_or_hex(&s, hex))?;
    let end = end.map_or(Ok(255), |s| parse_dec_or_hex(&s, hex))?;
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.display_mem_data(start ..= end, io::stdout(), hex)?;

    Ok(())
}

fn subcommand_code(
    input: PathBuf,
    hex: bool,
    start: Option<String>,
    end: Option<String>,
) -> Fallible<()> {
    let start = start.map_or(Ok(0), |s| parse_dec_or_hex(&s, hex))?;
    let end = end.map_or(Ok(127), |s| parse_dec_or_hex(&s, hex))?;
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.display_mem_opcodes(start ..= end, io::stdout(), hex)?;

    Ok(())
}

fn subcommand_regs(input: PathBuf, hex: bool) -> Fallible<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.display_registers(io::stdout(), hex)?;

    Ok(())
}

fn subcommand_stats(input: PathBuf) -> Fallible<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.display_stats(io::stdout())?;

    Ok(())
}

fn resolve_output<'args>(
    input: &'args Path,
    output_arg: &'args Option<PathBuf>,
) -> &'args Path {
    output_arg.as_ref().map_or(input, |buf| &**buf)
}

fn parse_dec_or_hex(num: &str, hex: bool) -> Fallible<u8> {
    u8::from_str_radix(num, if hex { 16 } else { 10 })
        .map_err(|e| WithPath { path: num.into(), error: e.into() }.into())
}
