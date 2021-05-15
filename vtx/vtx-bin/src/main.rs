mod wav;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Convert `vtx` file to wav format
    ConvertWav(wav::ConvertWav),
}

impl Command {
    pub fn execute(self) -> Result<(), anyhow::Error> {
        match self {
            Command::ConvertWav(cmd) => cmd.execute(),
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::from_args();
    args.cmd.execute()
}
