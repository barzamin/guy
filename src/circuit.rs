pub enum Term {
    Or(Vec<Term>),
    And(Vec<Term>),
    Not(Box<Term>),
    Buffer { inpt: Box<Term>, ctrl: Box<Term> },
    Signal(String),
}
