mod cmdline;

fn main() -> cmdline::CliResult {

    let args = cmdline::parse();
    let message = match args {
        cmdline::CliArgs::Init { directory } => {
            match directory {
                Some(dir) => String::from(dir.to_str().unwrap()),
                None => String::from("No directory specified.")
            }
        },

        cmdline::CliArgs::Compile { dest_dir } => String::from(dest_dir.to_str().unwrap())
    };

    println!("{}", message);
    Ok(())
}
