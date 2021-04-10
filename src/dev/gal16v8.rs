#[derive(Debug)]
pub struct OLMC {
    xor: bool,
    ac1: bool,
}

#[derive(Debug)]
pub struct Gal16V8 {
    fuses: Vec<bool>,
    syn: bool,
    ac0: bool,
    olmcs: Vec<OLMC>,
}

impl Gal16V8 {
    pub fn new(fuses: &[bool]) -> Self {
        assert_eq!(2194, fuses.len());

        Self {
            fuses: fuses.to_vec(),
            syn: fuses[2192],
            ac0: fuses[2193],
            olmcs: fuses[2048..=2055].iter()
                .zip(fuses[2120..=2127].iter())
                .map(|(&xor, &ac1)| OLMC { xor, ac1 }).collect(),
        }
    }
}
