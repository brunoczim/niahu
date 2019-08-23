use crate::Machine;
use std::io;

pub fn app() -> clap::App<'static, 'static> {
    clap::App::new("Simulador Niahu de Neander")
        .version("0.1.0")
        .author("Bruno C.Z. <brunoczim@gmail.com>")
        .about("Simula a máquina hipotética Neander")
        .subcommand(
            clap::SubCommand::with_name("run")
                .about(
                    "Executa o código a partir de onde o registrador pc \
                     (program counter) está, até que um HLT seja atingido",
                )
                .arg(
                    clap::Arg::with_name("input")
                        .long("input")
                        .short("i")
                        .takes_value(true)
                        .value_name("FILE")
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("output")
                        .long("output")
                        .short("o")
                        .takes_value(true)
                        .value_name("FILE")
                        .required(true),
                ),
        )
}

pub fn run<M>() -> io::Result<()>
where
    M: Machine,
{
    let mut vm = M::default();

    let matches = app();
    Ok(())
}
