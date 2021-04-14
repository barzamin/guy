use std::collections::BTreeMap;
use anyhow::Result;

pub trait Emit {
    fn emit(&self) -> Result<String>;
}

pub struct Portmap {
    map: BTreeMap<usize, String>,
}

impl Portmap {
    pub fn new() -> Self {
        Self { map : BTreeMap::new() }
    }

    pub fn map(&self, i: usize) -> Option<&str> {
        self.map.get(&i).map(String::as_str)
    }
}
