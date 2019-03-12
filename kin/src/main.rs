use kin_core;
use kin_core::{ CliResult, SubCommand };
mod compile;

fn main() -> CliResult {

    kin_core::libsodium_init()?;

    // TODO: Kill this and replace with new stand-alone decrypt executable
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

            let decrypt_args = kin_core::DecryptArgs {
                backup_dir: Some(exe_path.parent().unwrap().to_path_buf()),
                destination: None // Will prompt the user for a destination
            };

            kin_core::decrypt::run(&decrypt_args)?;

            return Ok(());
        }
    }

    let args = kin_core::parse_cmdline();

    match args {
        SubCommand::Init(args) => kin_core::init::run(&args),
        SubCommand::Compile(args) => compile::run(&args),
        SubCommand::Decrypt(args) => kin_core::decrypt::run(&args)
    }?;

    Ok(())
}
