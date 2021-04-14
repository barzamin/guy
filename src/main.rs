use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::Result;
use colored::*;
use jedec::JEDECFile;
use structopt::StructOpt;

mod dev;
mod emit;

use dev::gal16v8::{Gal16V8, Reducible};
use dev::Device;
use emit::{Emit, EmitCtx, Portmap};

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, short, possible_values = &Device::variants(), case_insensitive = true)]
    device: Device,

    #[structopt(long, short, parse(from_os_str))]
    portmap: Option<PathBuf>,

    #[structopt(name = "FILE", parse(from_os_str))]
    input_path: PathBuf,
}

fn dump_fuses(dev: &Gal16V8) {
    for (i, row) in dev.fuses.grid().chunks(32).enumerate() {
        if i % 8 == 0 {
            let idx = i / 8;
            if i > 0 {
                println!();
            }
            println!(
                "-- olmc{}: xor={}, ac1={} --",
                idx,
                dev.fuses.xor(idx) as u8,
                dev.fuses.ac1(idx) as u8
            );
        }
        print!("{}: ", format!("{:>4}", i * 32).red());
        for (i, bit) in row.iter().enumerate() {
            match bit {
                false => print!("{}", "0".bright_magenta()),
                true => print!("{}", "1".dimmed().white()),
            }
            if i < 31 {
                if (i + 1) % 4 == 0 {
                    print!("  ");
                } else {
                    print!(" ");
                }
            }
        }
        println!();
    }
}

fn main() -> Result<()> {
    let opt = Opts::from_args();
    let dat = fs::read(opt.input_path)?;
    let portmap = match opt.portmap {
        Some(path) => Portmap::deser(File::open(path)?)?,
        None => Portmap::new(),
    };
    let jd = JEDECFile::from_bytes(&dat)?;

    match opt.device {
        Device::Gal16V8 => {
            let lesb = Gal16V8::new(&jd.f)?;
            println!("/*");
            println!(
                "mode: {:?} (syn={}, ac0={})",
                lesb.mode,
                lesb.fuses.syn() as u8,
                lesb.fuses.ac0() as u8
            );
            println!(
                "sig: {:?}",
                String::from_utf8(lesb.fuses.signature()).unwrap()
            );
            // dump_fuses(&lesb);
            println!("*/");

            let mut out = String::new();
            lesb.emit(&mut out, &EmitCtx { portmap })?;
            println!("{}", out);
        }
    }

    Ok(())
}
