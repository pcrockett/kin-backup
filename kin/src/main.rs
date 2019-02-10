mod lib;
#[macro_use] extern crate serde_derive;

fn main() -> lib::CliResult {

    lib::libsodium_init()?;

    if std::env::args().len() <= 1 {
        // Not run with any args; the first arg in the array is just the
        // executable path

        let exe_path = std::env::current_exe()?; // Intentionally not getting the exe path from the first arg
        let exe_name = exe_path.file_name().unwrap()
            .to_str().unwrap();

        if exe_name == "decrypt" {

            // This executable is being run from a previously-generated backup
            // package. It is being run by a backup holder that shouldn't be
            // bothered with pesky command line syntax.... It's ok, we already
            // know what the command line args _should_ be.

            let decrypt_args = lib::DecryptArgs {
                backup_dir: Some(exe_path.parent().unwrap().to_path_buf()),
                destination: None // Will prompt the user for a destination
            };

            lib::decrypt::run(&decrypt_args)?;

            return Ok(());
        }
    }

    let args = lib::parse_cmdline();

    match args {
        lib::SubCommand::Init(args) => lib::init::run(&args),
        lib::SubCommand::Compile(args) => lib::compile::run(&args),
        lib::SubCommand::Decrypt(args) => lib::decrypt::run(&args)
    }?;

    Ok(())
}
