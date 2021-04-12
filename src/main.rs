use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use colored::*;
use jedec::JEDECFile;
use structopt::StructOpt;

mod dev;

use dev::gal16v8::{Gal16V8, Reducible, OLMC};
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
    let jd = JEDECFile::from_bytes(&dat)?;

    match opt.device {
        Device::Gal16V8 => {
            let lesb = Gal16V8::new(&jd.f)?;
            println!(
                "mode: {:?} (syn={}, ac0={})",
                lesb.mode, lesb.syn as u8, lesb.ac0 as u8
            );
            println!("OLMCs:");
            for (i, olmc) in lesb.olmcs.iter().enumerate() {
                println!(" [{}] - {:?}", i, olmc);
            }
            println!("sig: {:?}", std::str::from_utf8(&lesb.signature).unwrap());
            println!("ptd: {:?}", lesb.ptd);

            dump_fuses(&lesb.olmcs, &lesb.fuses);
            println!();
            let cols = lesb.col_signals();
            println!("{} signals: {:?}", "column".blue(), cols);
            println!();
            let rows: Vec<_> = (0..64).map(|i| lesb.and_term(i, &cols)).collect();
            for (i, term) in rows.iter().enumerate() {
                if !term.is_always_bot() {
                    println!("{:>2}/{:>4}. {}", i, i * 32, term);
                }
            }

            for i in 0..8 {
                println!("{}", lesb.or_term(i, &rows));
            }
        }
    }

    Ok(())
}
