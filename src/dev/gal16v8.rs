use anyhow::{anyhow, Result};
use std::fmt;

// use crate::circuit::{Signal, SignalKey, Circuit};

#[derive(Debug)]
pub struct OLMC {
    pub xor: bool,
    pub ac1: bool,
}

#[derive(Debug)]
pub struct Gal16V8 {
    pub fuses: Vec<bool>,

    pub syn: bool,
    pub ac0: bool,
    pub mode: Mode,

    pub signature: Vec<u8>,

    pub olmcs: Vec<OLMC>,
    pub ptd: Vec<Vec<bool>>, // macrocell # |-> enabled prod terms
}

fn to_u8(slice: &[bool]) -> u8 {
    slice.iter().fold(0, |acc, &b| (acc << 1) | (b as u8))
}

#[derive(Debug)]
pub enum Mode {
    /// Tristate or flop outs
    Registered,
    /// Tristate outs
    Complex,
    /// Combinatorial outs
    Simple,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColSignal {
    Pin { id: u32, n: bool },
    FlopOut { olmc: usize, n: bool },
}

impl fmt::Display for ColSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ColSignal::Pin { id, n } => {
                if n {
                    write!(f, "~")?;
                }
                write!(f, "p{}", id)?;
            }
            ColSignal::FlopOut { olmc, n } => {
                if n {
                    write!(f, "~")?;
                }
                write!(f, "q{}", olmc)?;
            }
        }

        Ok(())
    }
}

pub trait Reducible {
    /// Is the equation (fragment) trivially bottom?
    fn is_always_bot(&self) -> bool;
}

#[derive(Debug)]
pub struct ProdTerm(Vec<ColSignal>);

impl Reducible for ProdTerm {
    fn is_always_bot(&self) -> bool {
        for factor in &self.0 {
            if self.0.contains(&factor.inverted()) {
                return true
            }
        }

        false
    }
}

impl fmt::Display for ProdTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut factors = self.0.iter().peekable();
        while let Some(factor) = factors.next() {
            write!(f, "{}", factor)?;
            if factors.peek().is_some() {
                write!(f, " & ")?;
            }
        }

        Ok(())
    }
}

impl ColSignal {
    const fn flop(olmc: usize) -> ColSignal {
        Self::FlopOut { olmc, n: false }
    }

    const fn pin(id: u32) -> ColSignal {
        Self::Pin { id, n: false }
    }

    const fn inverted(self) -> ColSignal {
        use ColSignal::*;
        match self {
            Pin { id, n } => Pin { id, n: !n },
            FlopOut { olmc, n } => FlopOut { olmc, n: !n },
        }
    }
}

impl Gal16V8 {
    pub fn new(fuses: &[bool]) -> Result<Self> {
        assert_eq!(2194, fuses.len());

        let syn = fuses[2192];
        let ac0 = fuses[2193];

        let mode = match (syn, ac0) {
            (false, true) => Ok(Mode::Registered),
            (true, true) => Ok(Mode::Complex),
            (true, false) => Ok(Mode::Simple),
            _ => Err(anyhow!("invalid fuses")),
        }?;

        let olmcs = fuses[2048..=2055]
            .iter()
            .zip(fuses[2120..=2127].iter())
            .map(|(&xor, &ac1)| OLMC { xor, ac1 })
            .collect();

        let signature = fuses[2056..=2119]
            .chunks(8)
            .map(|octet| to_u8(octet))
            .collect();

        Ok(Self {
            fuses: fuses.to_vec(),
            syn: fuses[2192],
            ac0: fuses[2193],
            mode,
            olmcs,
            signature,
            ptd: fuses[2128..=2191].chunks(8).map(|x| x.to_vec()).collect(),
        })
    }

    pub fn olmc_feedback(&self, idx: usize) -> ColSignal {
        match self.mode {
            Mode::Registered => {
                if !self.olmcs[idx].ac1 {
                    // registered
                    ColSignal::flop(idx).inverted()
                } else {
                    // combinatorial
                    // olmc i has pin 19-i for i∈[0,7]
                    ColSignal::pin(19 - idx as u32)
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn col_signals(&self) -> Vec<ColSignal> {
        fn push_pair(sigs: &mut Vec<ColSignal>, sig: ColSignal) {
            sigs.push(sig);
            sigs.push(sig.inverted());
        }

        match self.mode {
            // this is what the first GAL I'm interested in from my gf's oscilloscope uses, so it's where i started.
            // there are many more fun GALs to re though.
            Mode::Registered => {
                // less err-prone way of constructing
                let mut sigs = Vec::new();
                for i in 0..8 {
                    // i ∈ [0, 7] is macrocell index
                    push_pair(&mut sigs, ColSignal::pin(i as u32 + 2));
                    push_pair(&mut sigs, self.olmc_feedback(i));
                }

                sigs
            }
            _ => unimplemented!(),
        }
    }

    pub fn and_term(&self, i: usize, cols: &[ColSignal]) -> ProdTerm {
        ProdTerm(
            self.and_term_fuses(i)
                .iter()
                .zip(cols.iter())
                .filter(|(&fuse, &_factor)| !fuse)
                .map(|(&_fuse, &factor)| factor)
                .collect(),
        )
    }

    pub fn and_term_fuses(&self, i: usize) -> &[bool] {
        assert!(i < 64);
        &self.fuses[i * 32..i * 32 + 32]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_to_u8() {
        assert_eq!(
            0x43,
            to_u8(&[false, true, false, false, false, false, true, true])
        );
    }
}
