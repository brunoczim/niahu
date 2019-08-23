use std::{io, path::PathBuf, process};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "neander",
    about = "Simulador multiplataforma da máquina hipotética Neander"
)]
enum Command {
    #[structopt(name = "new")]
    New {
        #[structopt(short = "o", parse(from_os_str))]
        output: PathBuf,
    },

    #[structopt(name = "run")]
    Run {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "o", parse(from_os_str))]
        output: PathBuf,
    },

    #[structopt(name = "run")]
    Step {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "o", parse(from_os_str))]
        output: PathBuf,
        #[structopt(short = "n", default_value = "1")]
        steps: u64,
    },

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

    #[structopt(name = "registers")]
    Regs {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "x")]
        hex: bool,
    },

    #[structopt(name = "stats")]
    Stats {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
    },

    #[structopt(name = "write")]
    Write {
        #[structopt(short = "i", parse(from_os_str))]
        input: PathBuf,
        #[structopt(short = "o", parse(from_os_str))]
        output: PathBuf,
        #[structopt(short = "x")]
        hex: bool,
        #[structopt(short = "a")]
        addr: String,
        #[structopt(short = "d")]
        data: String,
    },
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        process::exit(-1);
    }
}

fn try_main() -> io::Result<()> {
    match Command::from_args() {
        Command::New { output } => subcommand_new(output),

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

        Command::Write { input, output, hex, addr, data } => {
            subcommand_write(input, output, hex, addr, data)
        },
    }
}

fn subcommand_new(output: PathBuf) -> io::Result<()> {
    let vm = neander::Machine::new();
    vm.save_at_path(&output)?;
    Ok(())
}

fn subcommand_run(input: PathBuf, output: PathBuf) -> io::Result<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.execute();
    vm.save_at_path(&output)?;

    Ok(())
}

fn subcommand_step(
    input: PathBuf,
    output: PathBuf,
    steps: u64,
) -> io::Result<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    for _ in 0 .. steps {
        vm.cycle();
    }
    vm.save_at_path(&output)?;

    Ok(())
}

fn subcommand_data(
    input: PathBuf,
    hex: bool,
    start: Option<String>,
    end: Option<String>,
) -> io::Result<()> {
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
) -> io::Result<()> {
    let start = start.map_or(Ok(0), |s| parse_dec_or_hex(&s, hex))?;
    let end = end.map_or(Ok(127), |s| parse_dec_or_hex(&s, hex))?;
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.display_mem_opcodes(start ..= end, io::stdout(), hex)?;

    Ok(())
}

fn subcommand_regs(input: PathBuf, hex: bool) -> io::Result<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.display_registers(io::stdout(), hex)?;

    Ok(())
}

fn subcommand_stats(input: PathBuf) -> io::Result<()> {
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.display_stats(io::stdout())?;

    Ok(())
}

fn subcommand_write(
    input: PathBuf,
    output: PathBuf,
    hex: bool,
    addr: String,
    data: String,
) -> io::Result<()> {
    let addr = parse_dec_or_hex(&addr, hex)?;
    let data = parse_dec_or_hex(&data, hex)?;
    let mut vm = neander::Machine::new();

    vm.load_from_path(&input)?;
    vm.write_raw(addr, data);
    vm.save_at_path(&output)?;

    Ok(())
}

fn parse_dec_or_hex(num: &str, hex: bool) -> io::Result<u8> {
    u8::from_str_radix(num, if hex { 16 } else { 10 })
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
}
