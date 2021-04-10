use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use jedec::JEDECFile;
use structopt::StructOpt;

mod dev;
mod circuit;

use dev::gal16v8::Gal16V8;
use dev::Device;

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, short, possible_values = &Device::variants(), case_insensitive = true)]
    device: Device,

    #[structopt(name = "FILE", parse(from_os_str))]
    input_path: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opts::from_args();
    let dat = fs::read(opt.input_path)?;
    let jd = JEDECFile::from_bytes(&dat)?;

    match opt.device {
        Device::Gal16V8 => {
            let lesbian = Gal16V8::new(&jd.f)?;
            // println!("{:#?}", lesbian);
            println!("mode: {:?}", lesbian.mode);
            println!("syn: {:?}, ac0: {:?}", lesbian.syn, lesbian.ac0);
            println!("OLMCs: {:?}", lesbian.olmcs);
            println!("sig: {:?}", String::from_utf8(lesbian.signature));
            println!("ptd: {:?}", lesbian.ptd);
        }
    }

    Ok(())
}
