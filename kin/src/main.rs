mod lib;
#[macro_use] extern crate serde_derive;

fn main() -> lib::CliResult {

    let args = lib::parse_cmdline();

    lib::libsodium_init()?;

    match args {
        lib::SubCommand::Init(args) => lib::init::run(&args),
        lib::SubCommand::Compile(args) => lib::compile::run(&args),
        lib::SubCommand::Decrypt(args) => lib::decrypt::run(&args)
    }?;

    Ok(())
}
