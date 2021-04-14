use anyhow::Result;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;

pub trait Emit {
    fn emit<W>(&self, f: &mut W, emit_ctx: &EmitCtx) -> Result<()>
    where
        W: fmt::Write;
}

#[derive(Debug)]
pub struct EmitCtx {
    pub portmap: Portmap,
}

impl EmitCtx {
    pub fn pin(&self, pin: usize) -> String {
        self.portmap.map(pin).map(str::to_owned).unwrap_or_else(|| format!("p{}", pin))
    }
}

#[derive(Debug)]
pub struct Portmap {
    map: BTreeMap<usize, String>,
}

impl Portmap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn map(&self, i: usize) -> Option<&str> {
        self.map.get(&i).map(String::as_str)
    }

    pub fn deser<R>(rdr: R) -> ron::Result<Self>
    where
        R: Read,
    {
        Ok(Self {
            map: ron::de::from_reader(rdr)?,
        })
    }
}
