use structopt::StructOpt;
use quicli::prelude::*;

#[derive(StructOpt)]
#[structopt(name = "kin", about = "Easy, secure backups for your next of kin")]
enum CommandLineArgs {

    /// Start a backup project
    #[structopt(name = "init")]
    Init {

        /// The directory where you will assemble the things you need to back up
        #[structopt(parse(from_os_str))]
        directory: Option<std::path::PathBuf>
    },

    /// Compile a backup
    #[structopt(name = "compile")]
    Compile {

        /// The destination directory where you want to generate the backup
        #[structopt(name = "destination", parse(from_os_str))]
        dest_dir: std::path::PathBuf
    }
}

fn main() -> CliResult {

    let args = CommandLineArgs::from_args();
    let message = match args {
        CommandLineArgs::Init { directory } => {
            match directory {
                Some(dir) => String::from(dir.to_str().unwrap()),
                None => String::from("No directory specified.")
            }
        },

        CommandLineArgs::Compile { dest_dir } => String::from(dest_dir.to_str().unwrap())
    };

    println!("{}", message);
    Ok(())
}
