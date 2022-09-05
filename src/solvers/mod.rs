pub mod naive;

use crate::types::Result;

pub trait Solver {
    fn solve(&mut self) -> Result;
}
