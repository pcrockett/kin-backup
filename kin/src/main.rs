mod lib;

fn main() -> lib::cmdline::CliResult {

    let args = lib::cmdline::parse();
    match args {
        lib::cmdline::SubCommand::Init(args) => lib::init::run(args),
        lib::cmdline::SubCommand::Compile(_) => Err(failure::err_msg("Not implemented yet"))
    }?;

    Ok(())
}
