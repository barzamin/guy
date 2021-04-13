use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use colored::*;
use jedec::JEDECFile;
use structopt::StructOpt;

mod dev;

use dev::gal16v8::{Gal16V8, Reducible};
use dev::Device;

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, short, possible_values = &Device::variants(), case_insensitive = true)]
    device: Device,

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
    let jd = JEDECFile::from_bytes(&dat)?;

    match opt.device {
        Device::Gal16V8 => {
            let lesb = Gal16V8::new(&jd.f)?;
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

            dump_fuses(&lesb);

            println!("reg q1, q2, q3, q4, q5, q6, q7;\n");

            for (i, e) in lesb.elaboration.iter().enumerate() {
                println!("/* OLMC {} */", i);
                println!("{}", e);
            }
            // println!();
            // println!("{} signals", "column".blue());
            // print!("{{");
            // for (j, col) in lesb.cols.iter().enumerate() {
            //     print!("{} => {},", j, format!("{}", col));
            // }
            // println!("}}");
            // println!();
            // for (i, term) in lesb.rows.iter().enumerate() {
            //     if !term.is_always_bot() {
            //         println!(
            //             "{}/{}. {}",
            //             format!("{:>2}", i).red(),
            //             format!("{:>4}", i*32).red(),
            //             term
            //         );
            //     }
            // }

            // for i in 0..8 {
            //     println!("{}. ┌ {}", i, lesb.or_term(i));
            //     println!("   └ {:?}", lesb.out_buffer(i));
            // }
        }
    }

    Ok(())
}
