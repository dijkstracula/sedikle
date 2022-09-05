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

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Positive(_) => true,
            Self::Negative(_) => false,
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

    pub fn eval(&self, assignments: &Vec<bool>) -> bool {
        self.0.iter()
            .fold(false, 
                |curr, v| curr || (v.as_bool() == assignments[v.var() - 1]))
    }
}

#[derive(Debug, PartialEq)]
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
        
        assert!(min_max.0 == 1); // Just for now.

        Conjunction {
            atom_domain: Range {start: min_max.0, end: min_max.1 + 1},
            disjunctions: disjunctions,
        }
    }
}

// A model is an enumeration of assignments to a formula.
#[derive(Clone, Debug, PartialEq)]
pub struct Model<'a> {
    pub cnf: &'a Conjunction,
    pub assignments: Vec<Option<bool>>
}

impl<'a> Model<'a> {
    // TODO: evaling a model should probably produce values for each literal, not the
    // final thing.
    pub fn eval(&self) -> bool {
        let unwrapped: Vec<bool> = self.assignments.iter()
            .map(|ob| ob.unwrap())
            .collect();

        self.cnf.disjunctions.iter()
            .fold(true, |curr, clause| curr && clause.eval(&unwrapped))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Result<'a> {
    Sat(&'a Model<'a>), 
    Unsat, // Core?
}