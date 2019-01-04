mod lib;
#[macro_use] extern crate serde_derive;

fn main() -> lib::cmdline::CliResult {

    let args = lib::cmdline::parse();

    lib::libsodium::init()?;

    match args {
        lib::cmdline::SubCommand::Init(args) => lib::init::run(&args),
        lib::cmdline::SubCommand::Compile(args) => lib::compile::run(&args)
    }?;

    Ok(())
}
