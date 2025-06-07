use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::{env, fs};

pub(crate) mod commands;
pub(crate) mod objects;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        #[clap(short = 'm')]
        message: String,

        #[clap(short = 'p')]
        parent_hash: Option<String>,

        tree_hash: String,
    },
    Commit {
        #[clap(short = 'm')]
        message: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir_all(".git").unwrap();
            fs::create_dir_all(".git/objects").unwrap();
            fs::create_dir_all(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();

            println!(
                "Initialized empty Git repository in {}",
                env::current_dir().unwrap().display()
            );
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, &object_hash)
            .context("cat-file command failed")?,
        Command::HashObject { write, file } => commands::hash_object::invoke(write, &file)?,
        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash)?,
        Command::WriteTree {} => commands::write_tree::invoke()?,
        Command::CommitTree {
            message,
            tree_hash,
            parent_hash,
        } => commands::commit_tree::invoke(message, tree_hash, parent_hash)
            .context("commit-tree command failed")?,
        Command::Commit { message } => {
            let head_ref = std::fs::read_to_string(".git/HEAD").context("read .git/HEAD")?;
            let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
                anyhow::bail!("refusing to commit on detached HEAD)");
            };

            let head_ref = head_ref.trim();
            let parent_hash =
                fs::read_to_string(format!(".git/{head_ref}")).context("resolve HEAD ref")?;
            let parent_hash = parent_hash.trim();

            let Some(tree_hash) = commands::write_tree::write_tree_for(Path::new("."))
                .context("write tree for current directory")?
            else {
                eprintln!("nothing to commit, no files changed");
                return Ok(());
            };

            let commit_hash = commands::commit_tree::write_commit(
                &message,
                &hex::encode(tree_hash),
                Some(parent_hash),
            )
            .context("write commit")?;

            let commit_hash = hex::encode(commit_hash);

            fs::write(format!(".git/{}", head_ref), &commit_hash)
                .with_context(|| format!("update head reference target {}", head_ref))?;

            println!("Head is now at {}", &commit_hash);
        }
    }

    Ok(())
}
