use super::cmdline::CompileArgs;
use super::fsutil;

pub fn run(args: CompileArgs) -> Result<(), failure::Error> {

    fsutil::ensure_empty_dir(&args.dest_dir)?;

    let dest_dir = args.dest_dir.to_str().unwrap();
    Err(failure::err_msg(format!("Not implemented yet, but the destination is {}", dest_dir)))
}
