mod lib;

fn main() -> lib::cmdline::CliResult {

    let args = lib::cmdline::parse();
    match args {
        lib::cmdline::CliArgs::Init(args) => lib::init::run(args),
        lib::cmdline::CliArgs::Compile(_) => Err(failure::err_msg("Not implemented yet"))
    }?;

    Ok(())
}
