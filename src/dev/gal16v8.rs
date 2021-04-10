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
    pub olmcs: Vec<OLMC>,
    pub signature: Vec<u8>,
}

fn to_u8(slice: &[bool]) -> u8 {
    slice.iter().fold(0, |acc, &b| (acc << 1) | (b as u8))
}

impl Gal16V8 {
    pub fn new(fuses: &[bool]) -> Self {
        assert_eq!(2194, fuses.len());

        let olmcs = fuses[2048..=2055].iter()
                .zip(fuses[2120..=2127].iter())
                .map(|(&xor, &ac1)| OLMC { xor, ac1 }).collect();

        let signature = fuses[2056..=2119].chunks(8).map(|octet| to_u8(octet)).collect();

        Self {
            fuses: fuses.to_vec(),
            syn: fuses[2192],
            ac0: fuses[2193],
            olmcs,
            signature,
        }
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
