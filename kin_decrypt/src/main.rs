use kin_core as lib;

fn main() -> lib::CliResult {

    lib::libsodium_init()?;

    let exe_path = std::env::current_exe()?; // Intentionally not getting the exe path from the first arg

    let decrypt_args = lib::DecryptArgs {
        backup_dir: Some(exe_path.parent().unwrap().to_path_buf()),
        destination: None // Will prompt the user for a destination
    };

    lib::decrypt::run(&decrypt_args)?;

    Ok(())
}
