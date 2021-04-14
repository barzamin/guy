use anyhow::{anyhow, Result};
use std::io;

mod emit;

#[derive(Debug)]
pub struct Xor<T> {
    pub sig: T,
    pub xor: bool,
}

impl Xor<SumTerm> {
    pub fn trivially_const(&self) -> Option<bool> {
        if self.sig.is_always_bot() {
            Some(self.xor)
        } else {
            None
        }
    }
}

type OLMCIdx = usize;

#[derive(Debug)]
pub enum OLMCType {
    Reg,
    CombFeedback,
    Comb,
    CombInput,
}

impl OLMCType {
    pub fn feedback(&self, idx: OLMCIdx) -> ColSignal {
        match *self {
            Self::Reg => ColSignal::flop(idx).inverted(),
            Self::CombFeedback => ColSignal::pin(19 - idx as u32), // olmc i has pin 19-i for i∈[0,7]
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum ElaboratedOLMC {
    Registered {
        idx: usize,
        d: Xor<SumTerm>,
    },
    Complex {
        idx: usize,
        d: Xor<SumTerm>,
        oe: ProdTerm,
    },
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

#[derive(Debug, Clone)]
pub struct ProdTerm(pub Vec<ColSignal>);

#[derive(Debug, Clone)]
pub struct SumTerm(pub Vec<ProdTerm>);

#[derive(Debug)]
pub struct Fuses {
    inner: Vec<bool>,
}

impl ElaboratedOLMC {
    fn outpin(&self) -> usize {
        use ElaboratedOLMC::*;
        19 - match self {
            Registered { idx, .. } => idx,
            Complex { idx, .. } => idx,
        }
    }
}


impl Fuses {
    pub fn new(inner: Vec<bool>) -> Result<Self> {
        if inner.len() != 2194 {
            return Err(anyhow!("wrong number of fuses"));
        }

        Ok(Self { inner })
    }

    pub fn syn(&self) -> bool {
        self.inner[2192]
    }

    pub fn ac0(&self) -> bool {
        self.inner[2193]
    }

    pub fn signature(&self) -> Vec<u8> {
        self.inner[2056..=2119]
            .chunks(8)
            .map(|octet| to_u8(octet))
            .collect()
    }

    pub fn xor(&self, idx: usize) -> bool {
        assert!(idx < 8);
        self.inner[2048 + idx]
    }

    pub fn ac1(&self, idx: usize) -> bool {
        assert!(idx < 8);
        self.inner[2120 + idx]
    }

    pub fn ptd(&self, idx: usize) -> bool {
        assert!(idx < 64);
        self.inner[2128 + idx]
    }

    pub fn mode(&self) -> Result<Mode> {
        match (self.syn(), self.ac0()) {
            (false, true) => Ok(Mode::Registered),
            (true, true) => Ok(Mode::Complex),
            (true, false) => Ok(Mode::Simple),
            _ => Err(anyhow!("invalid fuses")),
        }
    }

    pub fn grid(&self) -> &[bool] {
        &self.inner[0..=2047]
    }

    pub fn olmc_type(&self, idx: usize, mode: Mode) -> OLMCType {
        match mode {
            Mode::Registered => {
                if self.ac1(idx) {
                    OLMCType::CombFeedback
                } else {
                    OLMCType::Reg
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn and_term_fuses(&self, i: usize) -> &[bool] {
        assert!(i < 64);
        &self.inner[i * 32..i * 32 + 32]
    }

    fn and_term(&self, cols: &[ColSignal], i: usize) -> ProdTerm {
        ProdTerm(
            self.and_term_fuses(i)
                .iter()
                .zip(cols.iter())
                .filter(|(&fuse, &_factor)| !fuse)
                .map(|(&_fuse, &factor)| factor)
                .collect(),
        )
    }
}

#[derive(Debug)]
pub struct Gal16V8 {
    pub fuses: Fuses,
    pub mode: Mode,
    pub elaboration: Vec<ElaboratedOLMC>,
    // pub cols: Vec<ColSignal>,
    // pub rows: Vec<ProdTerm>,
}

fn to_u8(slice: &[bool]) -> u8 {
    slice.iter().fold(0, |acc, &b| (acc << 1) | (b as u8))
}

/// Reduce terms or LIR
pub trait Reducible {
    /// Is the equation (fragment) trivially bottom?
    fn is_always_bot(&self) -> bool;
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

impl Reducible for SumTerm {
    fn is_always_bot(&self) -> bool {
        self.0.is_empty()
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
        let fuses = Fuses::new(fuses.to_vec())?;
        let mode = fuses.mode()?;

        let cols = Self::col_signals(mode, &fuses);
        let elaboration = Self::elaborate(mode, &fuses, &cols);

        Ok(Self {
            fuses,
            mode,
            elaboration,
        })
    }

    fn elaborate(mode: Mode, fuses: &Fuses, cols: &[ColSignal]) -> Vec<ElaboratedOLMC> {
        (0..8)
            .map(|i| {
                let ty = fuses.olmc_type(i, mode);

                match ty {
                    OLMCType::Reg => ElaboratedOLMC::Registered {
                        idx: i,
                        d: Xor {
                            sig: Self::or_term(fuses, cols, i, ty),
                            xor: fuses.xor(i),
                        },
                    },
                    OLMCType::CombFeedback => ElaboratedOLMC::Complex {
                        idx: i,
                        d: Xor {
                            sig: Self::or_term(fuses, cols, i, ty),
                            xor: fuses.xor(i),
                        },
                        oe: fuses.and_term(cols, i*8),
                    },
                    _ => unimplemented!(),
                }
            })
            .collect()
    }

    fn col_signals(mode: Mode, fuses: &Fuses) -> Vec<ColSignal> {
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
                    push_pair(&mut sigs, fuses.olmc_type(i, mode).feedback(i));
                }

                sigs
            }
            _ => unimplemented!(),
        }
    }

    fn or_term(fuses: &Fuses, cols: &[ColSignal], olmc_idx: OLMCIdx, olmc_ty: OLMCType) -> SumTerm {
        let col_idxs = match olmc_ty {
            OLMCType::Reg => olmc_idx * 8..olmc_idx * 8 + 8,
            OLMCType::CombFeedback => olmc_idx * 8 + 1..olmc_idx * 8 + 8,
            _ => unimplemented!(),
        };
        SumTerm(
            col_idxs
                .filter_map(|i| {
                    let t = fuses.and_term(cols, i);
                    if fuses.ptd(i) && !t.is_always_bot() {
                        Some(t)
                    } else {
                        None
                    }
                })
                .collect(),
        )
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
