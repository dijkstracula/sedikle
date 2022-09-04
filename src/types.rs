use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Literal {
    Positive(usize), //  x_i
    Negative(usize), // !x_i
}

impl Literal {
    pub fn from_dimacs_token(parsed: i64) -> Literal {
        if parsed < 0 {
            Literal::Negative(-parsed as usize)
        } else {
            Literal::Positive(parsed as usize)
        }
    }

    pub fn var(&self) -> usize {
        match self {
            Self::Positive(v) => *v,
            Self::Negative(v) => *v,
        }
    }
}

/* 
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
*/

// A Clause is a disjunction of literals.
#[derive(Debug, PartialEq)]
pub struct Clause(Vec<Literal>);

impl Clause {
    pub fn from_variables(vars: Vec<Literal>) -> Clause {
        // TODO: avoid copying??
        Clause(vars)
    }
}

pub struct Conjunction {
    pub disjunctions: Vec<Clause>,
    pub atom_domain: Range<usize>,
}

impl Conjunction {
    fn new(disjunctions: Vec<Clause>) -> Conjunction {
        let min_max = 
        disjunctions.iter()
            .flat_map(|c| &c.0)
            .map(|l| l.var())
            .fold((usize::MAX, usize::MIN), |curr, i| (curr.0.min(i), curr.1.max(i)));
        
        assert!(min_max.0 == 0); // Just for now.


        Conjunction {
            atom_domain: Range {start: min_max.0, end: min_max.1 + 1},
            disjunctions: disjunctions,
        }
    }
}

// A model is an enumeration of assignments to a formula.
#[derive(Clone, Debug)]
pub struct Model {
    pub atom_domain: Range<usize>,
    pub assignments: Vec<bool>,
}

#[derive(Clone, Debug)]
pub enum Result {
    Sat(Model), 
    Unsat, // Core?
}