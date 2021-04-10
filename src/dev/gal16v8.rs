use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct OLMC {
    pub xor: bool,
    pub ac1: bool,
}

#[derive(Debug)]
pub struct Gal16V8 {
    fuses: Vec<bool>,

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
    Registered,
    Complex,
    Simple,
}

impl Gal16V8 {
    pub fn new(fuses: &[bool]) -> Result<Self> {
        assert_eq!(2194, fuses.len());

        let syn = fuses[2192];
        let ac0 = fuses[2193];

        let mode = match (syn, ac0) {
            (false, true ) => Ok(Mode::Registered),
            (true , true ) => Ok(Mode::Complex),
            (true , false) => Ok(Mode::Simple),
            (false, false) => Err(anyhow!("invalid fuses")), // todo result
        }?;

        let olmcs = fuses[2048..=2055].iter()
                .zip(fuses[2120..=2127].iter())
                .map(|(&xor, &ac1)| OLMC { xor, ac1 }).collect();

        let signature = fuses[2056..=2119].chunks(8).map(|octet| to_u8(octet)).collect();

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
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_to_u8() {
        assert_eq!(0x43, to_u8(&[false, true, false, false, false, false, true, true]));
    }
}
