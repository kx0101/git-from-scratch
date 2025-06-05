use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::Digest;
use sha1::Sha1;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    let hash = if write {
        let tmp = "temporary";
        let hash = write_blob(&file, File::create(tmp).context("create temporary file")?)
            .context("write blob object")?;

        fs::create_dir_all(format!(".git/objects/{}", &hash[..2]))
            .context("create .git/objects directory")?;
        fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("move blob file into .git/objects")?;

        hash
    } else {
        write_blob(&file, std::io::sink()).context("write blob object")?
    };

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    writeln!(stdout, "{hash}").context("write hash to stdout")?;

    Ok(())
}

fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
where
    W: Write,
{
    let stat = fs::metadata(file).context("get file metadata")?;
    let writer = ZlibEncoder::new(writer, Compression::default());
    let mut writer = HashWriter {
        writer,
        hasher: Sha1::new(),
    };

    write!(writer, "blob ")?;
    write!(writer, "{}\0", stat.len())?;

    let mut file = File::open(file)?;
    std::io::copy(&mut file, &mut writer).context("copy file contents to zlib encoder")?;

    let _ = writer.writer.finish()?;
    let hash = writer.hasher.finalize();
    Ok(hex::encode(hash))
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);

        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
