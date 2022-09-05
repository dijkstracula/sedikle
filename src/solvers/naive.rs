/* A wildly-uninteresting solver implementation, maybe useful for test verification
 * and/or designing the solver interface. */

use std::ops::Range;

use crate::types::{Conjunction, Model, Result};

use super::Solver;

pub struct Naive<'a> {
    formula: &'a Conjunction,
    model: Model<'a>,
}

impl<'a> Naive<'a> {
    pub fn new(cnf: &'a Conjunction) -> Naive<'a> {
        Naive { 
            formula: cnf, 
            model: Model {
                cnf: &cnf,
                assignments: vec![None; cnf.atom_domain.end - cnf.atom_domain.start]
            } }
    }

    fn solve_ith(&mut self, vars: &mut Range<usize>) -> bool {
        match vars.next() {
            None => self.model.eval(),
            Some(i) => {
                self.model.assignments[i-1] = Some(true);
                match self.solve_ith(vars) {
                    true => true,
                    false => {
                        self.model.assignments[i-1] = Some(false);
                        self.solve_ith(vars)
                    }
                }
            }
        }
    }
}

impl<'a> Solver for Naive<'a> {
    fn solve(&mut self) -> Result {
        let mut range = self.formula.atom_domain.clone();
        match self.solve_ith(&mut range) {
            true => Result::Sat(&self.model),
            false => Result::Unsat,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{types::{Conjunction, Clause, Literal, Result, Model}, solvers::Solver};

    use super::Naive;

    #[test]
    fn test_trivial_sat() {
        let cnf = Conjunction{
            atom_domain: 1..2,
            disjunctions: vec![
                Clause::from_variables(vec![Literal::Positive(1)]),
                Clause::from_variables(vec![Literal::Positive(1)])],
        };

        let mut solver = Naive::new(&cnf);
        match solver.solve() {
            Result::Sat(model) => assert_eq!(model.eval(), true),
            _ => assert!(false),
        };
    }

    #[test]
    fn test_trivial_unsat() {
        let cnf = Conjunction{
            atom_domain: 1..2,
            disjunctions: vec![
                Clause::from_variables(vec![Literal::Positive(1)]),
                Clause::from_variables(vec![Literal::Negative(1)])],
        };

        let mut solver = Naive::new(&cnf);
        match solver.solve() {
            Result::Unsat => assert!(true),
            _ => assert!(false),
        };
    }
}