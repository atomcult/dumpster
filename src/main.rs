use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use anyhow::{Context, Result};

mod src;

use src::Source;


// TODO: async?
// TODO: enable the ability to extract chunks that aren't contained in any of the sources
//       from the target (-x, --extract)
// TODO: designate output format (-F, --format)
// TODO: overwrite (--force)
// TODO: --quiet and --verbose
// TODO: Enable searching bit-wise instead of byte-wise?


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(required = true)]
    sources: Vec<PathBuf>,

    #[arg(required = true)]
    target: PathBuf,
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
