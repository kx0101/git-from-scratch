use crate::objects::{Kind, Object};
use anyhow::Context;
use std::{
    ffi::CStr,
    io::{BufRead, Read, Write},
};

pub fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let mut object = Object::read(tree_hash).context("parse out blob object file")?;
    match object.kind {
        Kind::Tree => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            let mut buf = Vec::new();
            let mut hash_buf = [0; 20];
            loop {
                buf.clear();

                let n = object
                    .reader
                    .read_until(b'\0', &mut buf)
                    .context("read tree object entry")?;
                if n == 0 {
                    break; // EOF
                }

                object
                    .reader
                    .read_exact(&mut hash_buf[..])
                    .context("read tree object entry hash")?;
                let mode_and_name = CStr::from_bytes_with_nul(&buf)
                    .context("tree object entry is not nul-terminated")?;

                // TODO: replace with split_once https://github.com/rust-lang/rust/issues/112811
                let mut bits = mode_and_name.to_bytes().splitn(2, |&b| b == b' ');
                let mode = bits.next().expect("tree object entry has no mode");
                let name = bits.next().ok_or_else(|| {
                    anyhow::anyhow!(
                        "tree object entry has no name after mode: {:?}",
                        mode_and_name
                    )
                })?;

                if name_only {
                    stdout.write_all(name).context("write name to stdout")?;
                } else {
                    let mode = std::str::from_utf8(mode).context("mode is always valid utf-8")?;
                    let hash = hex::encode(&hash_buf);
                    let object = Object::read(&hash)
                        .with_context(|| format!("read object for tree entry {hash}"))?;

                    write!(stdout, "{mode:0>6} {} {hash} ", object.kind)
                        .context("write tree entry hash to stdout")?;
                    stdout.write_all(name).context("write name to stdout")?;
                }

                writeln!(stdout).context("write newline to stdout")?;
            }
        }
        _ => anyhow::bail!("dont know how to ls '{}'", object.kind),
    }

    Ok(())
}
