use anyhow::Context;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::objects::Kind;
use crate::objects::Object;

pub fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir = fs::read_dir(path).context("read directory")?;

    let mut entries = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("bad directory entry in {}", path.display()))?;
        entries.push(entry);
    }

    entries.sort_unstable_by(|a, b| {
        let mut afn = a.file_name().into_encoded_bytes();
        let mut bfn = b.file_name().into_encoded_bytes();

        // When we append 0xFF (byte 255) to strings, the shorter string will reach 0xFF sooner,
        // and since 255 is the highest possible byte,
        // that makes the shorter string sort after the longer one in a cmp()
        afn.push(0xff);
        bfn.push(0xff);

        afn.cmp(&bfn)
    });

    let mut tree_object = Vec::new();
    for entry in entries {
        let file_name = entry.file_name();
        if file_name == ".git" || file_name == "target" {
            continue;
        }

        let meta = entry
            .metadata()
            .context("get metadata for directory entry")?;

        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if (meta.permissions().mode() & 0o111) != 0 {
            "100755" // has at least one executable bit set
        } else {
            "100644"
        };

        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                continue; // skip empty directories
            };

            hash
        } else {
            let tmp = "temporary";
            let hash = Object::blob_from_file(&path)
                .context("open blob input file")?
                .write(File::create(tmp).context("create temporary file")?)
                .context("write file into blob")?;
            let hash_hex = hex::encode(hash);

            fs::create_dir_all(format!(".git/objects/{}", &hash_hex[..2]))
                .context("create subdir .git/objects directory")?;
            fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
            )
            .context("move blob file into .git/objects")?;

            hash
        };

        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }

    if tree_object.is_empty() {
        return Ok(None);
    }

    Ok(Some(
        Object {
            kind: Kind::Tree,
            expected_size: tree_object.len() as u64,
            reader: Cursor::new(tree_object),
        }
        .write_to_objects()
        .context("write tree object")?,
    ))
}

pub fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(Path::new(".")).context("write tree for current directory")?
    else {
        anyhow::bail!("asked to write empty tree, but this is not allowed");
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
