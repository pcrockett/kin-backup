mod lib;
#[macro_use] extern crate serde_derive;

fn main() -> lib::cmdline::CliResult {

    let args = lib::cmdline::parse();

    sodiumoxide::init().expect("unable to initialize libsodium");

    match args {
        lib::cmdline::SubCommand::Init(args) => lib::init::run(args),
        lib::cmdline::SubCommand::Compile(_) => Err(failure::err_msg("Not implemented yet"))
    }?;

    Ok(())
}
