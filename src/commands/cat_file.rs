use crate::objects::{Kind, Object};
use anyhow::Context;

pub fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "-p is required for cat-file");

    let mut object = Object::read(object_hash).context("parse out blob object file")?;
    match object.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("copy .git/objects contents to stdout")?;

            anyhow::ensure!(
                n == object.expected_size,
                ".git/objects file was not the expected size. Expected {}, got {}",
                object.expected_size,
                n
            );
        }
        _ => anyhow::bail!("dont know how to print '{}' object", object.kind),
    }

    Ok(())
}
