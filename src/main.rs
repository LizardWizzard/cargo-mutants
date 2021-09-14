// Copyright 2021 Martin Pool

use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use argh::FromArgs;
use similar::TextDiff;

mod mutate;
mod textedit;

use mutate::FileMutagen;

/// Rust mutation testing.
#[derive(FromArgs, PartialEq, Debug)]
struct TopArgs {
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    ApplyMutation(ApplyMutation),
    ListFiles(ListFiles),
    ListMutants(ListMutants),
}

/// List source files in a tree.
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list-files")]
struct ListFiles {
    /// rust crate directory to examine.
    #[argh(option)]
    dir: PathBuf,
}

/// List mutation scenarios that can be generated from a file.
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list-mutants")]
struct ListMutants {
    /// rust source file to examine.
    #[argh(option)]
    file: Option<PathBuf>,
}

/// Print mutated source to stdout.
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "apply")]
struct ApplyMutation {
    /// rust source file to read and mutate.
    #[argh(option)]
    file: PathBuf,

    /// mutation index number, from list-mutants.
    #[argh(option)]
    index: usize,

    /// show the diff between the original and mutated file.
    #[argh(switch)]
    diff: bool,
}

fn main() -> Result<()> {
    let args: TopArgs = argh::from_env();
    match args.command {
        Command::ListFiles(sub) => {
            // TODO: Check there's a Cargo.toml.
            for entry in walkdir::WalkDir::new(sub.dir.join("src"))
                .sort_by_file_name()
                .into_iter()
                .filter_map(|r| r.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .map_or(false, |p| p.eq_ignore_ascii_case("rs"))
                })
            {
                println!("{}", entry.path().display());
            }
        }
        Command::ListMutants(sub) => {
            let mutagen = FileMutagen::new(sub.file.expect("file must be specified"))?;
            for (i, mute) in mutagen.discover_mutation_sites().into_iter().enumerate() {
                println!("{:>8} {:<16?} {}", i, mute.op, mute.function_name());
            }
        }
        Command::ApplyMutation(sub) => {
            let mutagen = FileMutagen::new(sub.file)?;
            let mutation = mutagen
                .discover_mutation_sites()
                .into_iter()
                .nth(sub.index)
                .expect("index in range");
            let mutated_code = mutation.mutated_code(&mutagen);
            if sub.diff {
                let old_label = mutagen.path.to_string_lossy();
                let new_label = format!("{} {:?}", &old_label, &mutation);
                let text_diff = TextDiff::from_lines(&mutagen.code, &mutated_code);
                print!(
                    "{}",
                    text_diff
                        .unified_diff()
                        .context_radius(10)
                        .header(&old_label, &new_label)
                );
            } else {
                std::io::stdout().write_all(mutated_code.as_bytes())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    // use super::*;
}
