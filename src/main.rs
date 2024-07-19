use clap::*;
use kdam::{term::Colorizer, tqdm, Colour};
use std::io::IsTerminal;

mod pull;
use pull::pull;
mod process;
use process::process;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    pull: String,
}

fn main() {
    // A nice progress bar
    kdam::term::init(std::io::stderr().is_terminal());
    let pb = tqdm!(
        bar_format = format!(
            "{} {} {} [ {} < {} {} ]",
            "{percentage:3.0}%",
            "{animation}",
            "{count}/{total}".colorize("green"),
            "{elapsed}".colorize("yellow"),
            "{remaining}".colorize("blue"),
            "{rate:.1}it/s".colorize("red")
        ),
        colour = Colour::gradient(&["#5A56E0", "#EE6FF8"])
    );

    // basic app information
    // Parse the command line arguments using the generated Args struct
    let matches = Command::new("livermore")
        .about("fetching and processing stock data")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("pull")
                .short_flag('P')
                .long_flag("pull")
                .about("Pull data from screener csv")
                .arg(
                    Arg::new("path")
                        .help("/path/to/screener")
                        .action(ArgAction::Set)
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("process")
                .short_flag('p')
                .long_flag("process")
                .about("Process data for Livermore-analyze")
                .arg(
                    Arg::new("path")
                        .help("/path/to/data")
                        .action(ArgAction::Set)
                        .num_args(3),
                ),
        )
        .get_matches();
    match matches.subcommand() {
        Some(("pull", sync_matches)) => {
            let path: Vec<&str> = sync_matches
                .get_many::<String>("path")
                .expect("is present")
                .map(|s| s.as_str())
                .collect();
            pull(path[0], path[1], pb)
        }
        Some(("process", sync_matches)) => {
            let path: Vec<&str> = sync_matches
                .get_many::<String>("path")
                .expect("is present")
                .map(|s| s.as_str())
                .collect();
            process(path[0], path[1], path[2].parse::<usize>().unwrap(), pb);
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
}
