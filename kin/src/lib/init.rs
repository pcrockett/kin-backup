use super::cmdline;

pub fn run(args: cmdline::InitArgs) -> Result<(), failure::Error> {

    match args.directory {
        Some(dir) => println!("Initializing {}...", dir.to_str().unwrap()),
        None => return Err(failure::err_msg("TODO: Figure out current directory"))
    }

    Ok(())
}
