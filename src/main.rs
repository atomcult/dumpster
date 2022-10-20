use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::{Path, PathBuf};

use clap::Parser;
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(required = true)]
    sources: Vec<PathBuf>,

    #[arg(required = true)]
    target: PathBuf,
}

// TODO: async?
// TODO: enable the ability to extract chunks that aren't contained in any of the sources
//       from the target (-x, --extract)
// TODO: designate output format (-F, --format)
// TODO: overwrite (--force)
// TODO: --quiet and --verbose

enum FileRead {
    Next,
    End,
}

impl FileRead {
    fn is_end(&self) -> bool {
        match self {
            Self::End => true,
            _ => false,
        }
    }
}

struct Source {
    buffer: BufReader<File>,
    chunks: Vec<(usize, usize)>,
    start: Option<usize>,
    byte: [u8; 1],
}


impl Source {
    pub fn new(path: &Path) -> Result<Self> {
        let f = File::open(&path).with_context(|| {
            format!("Could not open file: `{}`", &path.display())
        })?;
        let buffer = BufReader::new(f);

        Ok(Self {
            buffer,
            chunks: Vec::new(),
            start: None,
            byte: [0u8],
        })
    }

    fn byte(&self) -> u8 {
        self.byte[0]
    }

    fn rewind(&mut self) -> Result<()> {
        self.buffer.rewind()?;
        self.next()?;

        Ok(())
    }

    fn next(&mut self) -> Result<FileRead> {
        return if self.buffer.read(&mut self.byte)? == 0 {
            Ok(FileRead::End)
        } else {
            Ok(FileRead::Next)
        }
    }

    pub fn check(&mut self, byte: u8, offset: usize) -> Result<()> {
        if byte == self.byte() {
            if self.start.is_none() { self.start = Some(offset); }

            if self.next()?.is_end() {
                self.chunks.push((self.start.unwrap(), offset));
                self.rewind()?;
            }
        } else {
            self.start = None;
            self.rewind()?;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let f = File::open(&args.target)
        .with_context(|| format!("Could not open file: {}", &args.target.display()))?;
    let target = BufReader::new(f);

    let mut sources = Vec::new();
    for path in args.sources {
        let src = Source::new(&path).with_context(|| {
            format!("Could not open file: {}", &path.display())
        })?;
        sources.push(src);
    }

    for (offset, byte) in target.bytes().enumerate() {
        if let Ok(byte) = byte {
            for src in &mut sources {
                src.check(byte, offset)?;
            }
        } else { break }
    }

    for src in sources {
        for chunk in src.chunks {
            println!("{:?}", chunk);
        }
    }

    Ok(())
}
