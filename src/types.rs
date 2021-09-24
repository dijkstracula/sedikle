#[derive(Debug, PartialEq)]
pub enum Atom {
    Positive(usize),
    Negative(usize),
}

impl Atom {
    pub fn from_dimacs_token(parsed: i64) -> Atom {
        if parsed < 0 {
            Atom::Negative(-parsed as usize)
        } else {
            Atom::Positive(parsed as usize)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Variable {
    Var(Atom),
    Not(Atom),
    Unassigned(Atom),
}

impl Variable {
    pub fn from_dimacs_token(parsed: i64) -> Variable {
        Variable::Unassigned(Atom::from_dimacs_token(parsed))
    }
}

pub type Clause = Vec<Variable>;
