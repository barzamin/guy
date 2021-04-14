use std::fmt::{self, Write};
use anyhow::Result;
use indenter::indented;


use crate::emit::Emit;
use super::{ElaboratedOLMC, Xor, ColSignal, ProdTerm, SumTerm, Gal16V8};

impl fmt::Display for ElaboratedOLMC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ElaboratedOLMC::*;
        match self {
            Registered { idx, d } => {
                if let Some(val) = d.trivially_const() {
                    writeln!(
                        f,
                        "assign {} = oe ? {} : 1'bz;",
                        format!("p{}", self.outpin()),
                        format!("1'b{}", val as u8)
                    )?;
                } else {
                    writeln!(f, "always @(posedge clk)")?;
                    writeln!(f, "  {} <= {};", format!("q{}", idx), d)?;
                    writeln!(
                        f,
                        "assign {} = oe ? {} : 1'bz;",
                        format!("p{}", self.outpin()),
                        format!("q{}", idx)
                    )?;
                }
            }
            Complex { idx, d, oe } => {
                writeln!(
                    f,
                    "assign {} = ({}) ? {} : 1'bz;",
                    format!("p{}", self.outpin()),
                    oe,
                    d
                )?;
            }
        }

        Ok(())
    }
}


impl<T> fmt::Display for Xor<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.xor {
            write!(f, "~")?;
        }

        write!(f, "{}", self.sig)?;

        Ok(())
    }
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


impl fmt::Display for ProdTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encl = self.0.len() > 1;
        let mut factors = self.0.iter().peekable();
        if encl {
            write!(f, "(")?;
        }
        while let Some(factor) = factors.next() {
            write!(f, "{}", factor)?;
            if factors.peek().is_some() {
                write!(f, " & ")?;
            }
        }
        if encl {
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl fmt::Display for SumTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() == 0 {
            write!(f, "1'b0")?;
        } else {
            let mut terms = self.0.iter().peekable();
            if self.0.len() > 1 {
                write!(f, "(")?;
            }
            while let Some(term) = terms.next() {
                write!(f, "{}", term)?;
                if terms.peek().is_some() {
                    write!(f, " | ")?;
                }
            }
            if self.0.len() > 1 {
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

impl Emit for Gal16V8 {
    fn emit(&self) -> Result<String> {
        let mut out = String::new();
        let mut f = indented(&mut out).with_str("");
        writeln!(f, "module GAL16V8 (")?;
        // writeln!(f, "  {}", )?;
        writeln!(f, ");\n")?;
        f = f.with_str("  ");
        for (i, e) in self.elaboration.iter().enumerate() {
            writeln!(f, "/* OLMC {} */", i)?;
            if let ElaboratedOLMC::Registered { idx, d } = e {
                if d.trivially_const().is_none() {
                    writeln!(f, "reg q{};", idx)?;
                }
            }
            writeln!(f, "{}", e)?;
        }

        f = f.with_str("");
        writeln!(f, "endmodule")?;

        Ok(out)
    }
}
