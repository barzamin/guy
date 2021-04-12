use anyhow::{anyhow, Result};
use std::fmt;

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
    pub ptd: Vec<bool>, // row # |-> enabled prod terms

    pub cols: Vec<ColSignal>,
    pub rows: Vec<ProdTerm>,
}

fn to_u8(slice: &[bool]) -> u8 {
    slice.iter().fold(0, |acc, &b| (acc << 1) | (b as u8))
}

#[derive(Debug, Clone, Copy)]
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

/// Reduce terms or LIR
pub trait Reducible {
    /// Is the equation (fragment) trivially bottom?
    fn is_always_bot(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ProdTerm(Vec<ColSignal>);

#[derive(Debug, Clone)]
pub struct SumTerm(Vec<ProdTerm>);

#[derive(Debug)]
pub enum OE {
    Const(bool),
    ProdTerm(ProdTerm),
    OEPin,
}

#[derive(Debug)]
pub enum OutSig {
    FlopOut(usize),
    SumTerm { term: SumTerm, xor: bool },
    Const(bool),
}

#[derive(Debug)]
pub struct OutBuffer {
    inverted: bool, // usu true
    oe: OE,
    sig: OutSig,
}

impl Reducible for ProdTerm {
    fn is_always_bot(&self) -> bool {
        for factor in &self.0 {
            if self.0.contains(&factor.inverted()) {
                return true;
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

impl fmt::Display for SumTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() == 0 {
            write!(f, "0")?;
        } else {
            let mut terms = self.0.iter().peekable();
            while let Some(term) = terms.next() {
                write!(f, "({})", term)?;
                if terms.peek().is_some() {
                    write!(f, " | ")?;
                }
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
        if fuses.len() != 2194 {
            return Err(anyhow!("wrong number of fuses"));
        }

        let syn = fuses[2192];
        let ac0 = fuses[2193];

        let mode = match (syn, ac0) {
            (false, true) => Ok(Mode::Registered),
            (true, true) => Ok(Mode::Complex),
            (true, false) => Ok(Mode::Simple),
            _ => Err(anyhow!("invalid fuses")),
        }?;

        let olmcs: Vec<_> = fuses[2048..=2055]
            .iter()
            .zip(fuses[2120..=2127].iter())
            .map(|(&xor, &ac1)| OLMC { xor, ac1 })
            .collect();

        let signature = fuses[2056..=2119]
            .chunks(8)
            .map(|octet| to_u8(octet))
            .collect();

        let cols = Self::col_signals(mode, &olmcs);
        let rows: Vec<_> = (0..64).map(|i| Self::and_term(&fuses, &cols, i)).collect();

        Ok(Self {
            fuses: fuses.to_vec(),
            syn: fuses[2192],
            ac0: fuses[2193],
            mode,
            olmcs,
            signature,
            ptd: fuses[2128..=2191].to_vec(),
            cols,
            rows,
        })
    }

    fn olmc_feedback(mode: Mode, olmcs: &[OLMC], idx: usize) -> ColSignal {
        match mode {
            Mode::Registered => {
                if !olmcs[idx].ac1 {
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

    fn col_signals(mode: Mode, olmcs: &[OLMC]) -> Vec<ColSignal> {
        fn push_pair(sigs: &mut Vec<ColSignal>, sig: ColSignal) {
            sigs.push(sig);
            sigs.push(sig.inverted());
        }

        match mode {
            // this is what the first GAL I'm interested in from my gf's oscilloscope uses, so it's where i started.
            // there are many more fun GALs to re though.
            Mode::Registered => {
                // less err-prone way of constructing
                let mut sigs = Vec::new();
                for i in 0..8 {
                    // i ∈ [0, 7] is macrocell index
                    push_pair(&mut sigs, ColSignal::pin(i as u32 + 2));
                    push_pair(&mut sigs, Self::olmc_feedback(mode, olmcs, i));
                }

                sigs
            }
            _ => unimplemented!(),
        }
    }

    pub fn out_buffer(&self, olmc_idx: usize) -> OutBuffer {
        let olmc = &self.olmcs[olmc_idx];
        match self.mode {
            Mode::Registered => {
                if !olmc.ac1 {
                    // registered
                    OutBuffer {
                        inverted: true,
                        oe: OE::OEPin,
                        sig: OutSig::FlopOut(olmc_idx),
                    }
                } else {
                    // combinatorial
                    OutBuffer {
                        inverted: true,
                        oe: OE::ProdTerm(self.rows[olmc_idx * 8].clone()),
                        sig: OutSig::SumTerm {
                            term: self.or_term(olmc_idx),
                            xor: olmc.xor,
                        },
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn or_term(&self, olmc_idx: usize) -> SumTerm {
        match self.mode {
            Mode::Registered => {
                let col_idxs = if !self.olmcs[olmc_idx].ac1 {
                    // registered
                    olmc_idx * 8..olmc_idx * 8 + 8
                } else {
                    // combinatorial
                    olmc_idx * 8 + 1..olmc_idx * 8 + 8
                };
                SumTerm(
                    col_idxs
                        .filter(|&i| self.ptd[i] && !self.rows[i].is_always_bot())
                        .map(|i| self.rows[i].clone())
                        .collect(),
                )
            }

            _ => unimplemented!(),
        }
    }

    fn and_term(fuses: &[bool], cols: &[ColSignal], i: usize) -> ProdTerm {
        ProdTerm(
            Self::and_term_fuses(fuses, i)
                .iter()
                .zip(cols.iter())
                .filter(|(&fuse, &_factor)| !fuse)
                .map(|(&_fuse, &factor)| factor)
                .collect(),
        )
    }

    pub fn and_term_fuses(fuses: &[bool], i: usize) -> &[bool] {
        assert!(i < 64);
        &fuses[i * 32..i * 32 + 32]
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
