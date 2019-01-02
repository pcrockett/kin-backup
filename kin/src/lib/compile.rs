use super::cmdline::CompileArgs;

pub fn run(args: CompileArgs) -> Result<(), failure::Error> {

    let dest_dir = args.dest_dir.to_str().unwrap();
    Err(failure::err_msg(format!("Not implemented yet, but the destination is {}", dest_dir)))
}
