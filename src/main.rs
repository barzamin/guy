use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use colored::*;
use jedec::JEDECFile;
use structopt::StructOpt;

mod dev;

use dev::gal16v8::{Gal16V8, OLMC, Reducible};
use dev::Device;

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, short, possible_values = &Device::variants(), case_insensitive = true)]
    device: Device,

    #[structopt(name = "FILE", parse(from_os_str))]
    input_path: PathBuf,
}

fn dump_fuses(olmcs: &[OLMC], fuses: &[bool]) {
    for (i, row) in fuses[0..=2047].chunks(32).enumerate() {
        if i % 8 == 0 {
            let idx = i / 8;
            if i > 0 {
                println!();
            }
            println!(
                "-- olmc{}: xor={}, ac1={} --",
                idx, olmcs[idx].xor as u8, olmcs[idx].ac1 as u8
            );
        }
        print!("{}: ", format!("{:>4}", i * 32).red());
        for (i, bit) in row.iter().enumerate() {
            match bit {
                true => print!("{}", "1".bright_magenta()),
                false => print!("{}", "0".dimmed().white()),
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
    let jd = JEDECFile::from_bytes(&dat)?;

    match opt.device {
        Device::Gal16V8 => {
            let lesb = Gal16V8::new(&jd.f)?;
            println!("mode: {:?}", lesb.mode);
            println!("syn: {:?}, ac0: {:?}", lesb.syn, lesb.ac0);
            println!("OLMCs: {:?}", lesb.olmcs);
            println!("sig: {:?}", &lesb.signature);
            println!("ptd: {:?}", lesb.ptd);

            dump_fuses(&lesb.olmcs, &lesb.fuses);
            println!();
            let cols = lesb.col_signals();
            println!("{} signals: {:?}", "column".blue(), cols);
            println!();
            for i in 0..64 {
                let term = lesb.and_term(i, &cols);
                if !term.is_always_bot() {
                    println!("{:>2}/{:>4}. {}", i, i*32, term);
                }
            }
        }
    }

    Ok(())
}
