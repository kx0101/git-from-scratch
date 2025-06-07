use anyhow::Context;
use std::fmt::Write;
use std::io::Cursor;

use crate::objects::Kind;
use crate::objects::Object;

pub fn write_commit(
    message: &str,
    tree_hash: &str,
    parent_hash: Option<&str>,
) -> anyhow::Result<[u8; 20]> {
    let mut commit = String::new();
    writeln!(commit, "tree {}", tree_hash)?;
    if let Some(parent_hash) = parent_hash {
        writeln!(commit, "parent {}", parent_hash)?;
    }

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("Failed to get current time")?;

    writeln!(
        commit,
        "author kx0101 <liakos.koulaxis@yahoo.com> {} +0000",
        time.as_secs(),
    )?;
    writeln!(
        commit,
        "committer kx0101 <liakos.koulaxis@yahoo.com> {} +0000",
        time.as_secs()
    )?;
    writeln!(commit, "")?;
    writeln!(commit, "{message}")?;

    Object {
        kind: Kind::Commit,
        expected_size: commit.len() as u64,
        reader: Cursor::new(commit),
    }
    .write_to_objects()
    .context("Failed to write commit object")
}

pub fn invoke(
    message: String,
    tree_hash: String,
    parent_hash: Option<String>,
) -> anyhow::Result<()> {
    let hash = write_commit(&message, &tree_hash, parent_hash.as_deref())
        .context("Failed to write commit")?;

    println!("{}", hex::encode(hash));

    Ok(())
}
