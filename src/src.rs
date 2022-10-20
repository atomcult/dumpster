use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};

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

pub struct Source {
    buffer: BufReader<File>,
    pub chunks: Vec<(usize, usize)>,
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
