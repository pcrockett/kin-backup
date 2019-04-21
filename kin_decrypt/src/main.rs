use kin_core;
use kin_core::DecryptArgs;
use nfd;
use nfd::Response::{Cancel, Okay, OkayMultiple};
use std::path::PathBuf;

fn main() -> kin_core::CliResult {
    kin_core::libsodium_init()?;

    let exe_path = std::env::current_exe()?; // Intentionally not getting the exe path from the first arg
    let private_dir = exe_path.parent().unwrap();
    let package_dir = match private_dir.parent() {
        Some(dir) => Some(dir.to_path_buf()),
        None => None,
    };

    let save_dialog_result = nfd::dialog_save()
        .filter("zip")
        .open()
        .expect("save file dialog error");

    let dest_path = match save_dialog_result {
        Okay(path) => Some(PathBuf::from(path)),
        OkayMultiple(_) => panic!("multiple files selected."),
        Cancel => None,
    };

    if dest_path.is_none() {
        println!("no destination file path selected.");
        return Ok(());
    }

    let decrypt_args = DecryptArgs {
        backup_dir: package_dir,
        destination: dest_path,
    };

    kin_core::decrypt::run(&decrypt_args)?;

    Ok(())
}
