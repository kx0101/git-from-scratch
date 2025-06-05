use anyhow::Context;
use core::fmt;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub struct Object<R> {
    pub kind: Kind,
    pub expected_size: u64,
    pub reader: R,
}

impl Object<()> {
    pub fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let f = File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("open in .git/objects")?;
        let z = ZlibDecoder::new(f);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();

        z.read_until(b'\0', &mut buf)
            .context("read header from .git/objects")?;

        let header = CStr::from_bytes_with_nul(&buf)
            .expect("know there is exactly one nul and it's in the end");
        let header = header
            .to_str()
            .context(".git/objects header isn't valid UTF-8")?;
        let Some((kind, size)) = header.split_once(' ') else {
            anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
        };

        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!(".git/objects file header had unknown type: '{kind}'"),
        };

        let size = size
            .parse::<u64>()
            .context(".git/objects file header has invalid size: {size}")?;

        // this wont error if the decompressed file is too long, but will at least not
        // spam stdout and be vulnerable to a zipbomb
        let z = z.take(size);
        Ok(Object {
            kind,
            expected_size: size,
            reader: z,
        })
    }
}
