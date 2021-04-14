use anyhow::Result;
use indenter::indented;
use std::fmt::{self, Write};

use super::{ColSignal, ElaboratedOLMC, Gal16V8, ProdTerm, SumTerm, Xor};
use crate::emit::{Emit, EmitCtx};

impl Emit for ElaboratedOLMC {
    fn emit<W>(&self, f: &mut W, ctx: &EmitCtx) -> Result<()>
    where
        W: fmt::Write,
    {
        use ElaboratedOLMC::*;
        match self {
            Registered { idx, d } => {
                if let Some(val) = d.trivially_const() {
                    writeln!(
                        f,
                        "assign {} = ~oe ? {} : 1'bz;",
                        ctx.pin(self.outpin()),
                        format!("1'b{}", !val as u8)
                    )?;
                } else {
                    writeln!(f, "always @(posedge clk)")?;
                    write!(f, "  {} <= ", format!("q{}", idx))?;
                    d.emit(f, ctx)?;
                    writeln!(f, ";")?;
                    writeln!(
                        f,
                        "assign {} = ~oe ? ~{} : 1'bz;",
                        ctx.pin(self.outpin()),
                        format!("q{}", idx)
                    )?;
                }
            }
            Complex { idx, d, oe } => {
                write!(f, "assign {} = (", ctx.pin(self.outpin()))?;
                oe.emit(f, &ctx)?;
                write!(f, ") ? ")?;
                d.emit(f, &ctx)?;
                writeln!(f, " : 1'bz;")?;
            }
        }
        writeln!(f)?;

        Ok(())
    }
}

impl<T> Emit for Xor<T>
where
    T: Emit,
{
    fn emit<W>(&self, f: &mut W, ctx: &EmitCtx) -> Result<()>
    where
        W: fmt::Write,
    {
        if self.xor {
            write!(f, "~")?;
        }

        self.sig.emit(f, ctx)?;

        Ok(())
    }
}

impl Emit for ColSignal {
    fn emit<W>(&self, f: &mut W, ctx: &EmitCtx) -> Result<()>
    where
        W: fmt::Write,
    {
        match *self {
            ColSignal::Pin { id, n } => {
                if n {
                    write!(f, "~")?;
                }
                write!(f, "{}", ctx.pin(id))?;
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

impl Emit for ProdTerm {
    fn emit<W>(&self, f: &mut W, ctx: &EmitCtx) -> Result<()>
    where
        W: fmt::Write,
    {
        let encl = self.0.len() > 1;
        let mut factors = self.0.iter().peekable();
        if encl {
            write!(f, "(")?;
        }
        while let Some(factor) = factors.next() {
            factor.emit(f, ctx)?;
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

impl Emit for SumTerm {
    fn emit<W>(&self, f: &mut W, ctx: &EmitCtx) -> Result<()>
    where
        W: fmt::Write,
    {
        if self.0.len() == 0 {
            write!(f, "1'b0")?;
        } else {
            let mut terms = self.0.iter().peekable();
            if self.0.len() > 1 {
                write!(f, "(")?;
            }
            while let Some(term) = terms.next() {
                term.emit(f, ctx)?;
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
    fn emit<W>(&self, f: &mut W, ctx: &EmitCtx) -> Result<()>
    where
        W: fmt::Write,
    {
        let mut f = indented(f).with_str("");
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
            e.emit(&mut f, ctx)?;
        }

        f = f.with_str("");
        writeln!(f, "endmodule")?;

        Ok(())
    }
}
