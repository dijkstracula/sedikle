use core::fmt;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use crate::types::{self, Clause, Literal};


/* CNF input parsing routines.  Format taken from:
 * https://www.cs.utexas.edu/users/moore/acl2/manuals/current/manual/index-seo.php/SATLINK____DIMACS
 *
 * Only does a tiny amount of sanity-checking on the input files.
 *
 * The source iterator stuff was surely overkill but that was done in the name of practicing a bit
 * of type astronautics, and as it turns out, lazily reading a file line by line is the hardest
 * problem in Rust anyway.
 */

#[derive(Debug)]
pub enum DimacsError {
    UnexpectedEOF,
    Error(usize, String),
    IO(std::io::Error),
}

impl std::error::Error for DimacsError {}

impl fmt::Display for DimacsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            &Self::UnexpectedEOF => write!(f, "Unexpected EOF"),
            &Self::Error(line_num, msg) => write!(f, "Line {}: {}", line_num, msg),
            &Self::IO(inner) => write!(f, "{}", &inner),
        }
    }
}

impl From<std::io::Error> for DimacsError {
    fn from(e: std::io::Error) -> Self {
        DimacsError::IO(e)
    }
}

#[derive(Debug)]
struct ParserState {
    header_line: usize,
    vars_seen: Vec<bool>,
    expected_clauses: usize,
    expected_vars: usize,
}

impl ParserState {
    fn new(header_line: usize, expected_vars: usize, expected_clauses: usize) -> ParserState {
        ParserState {
            header_line: header_line,
            vars_seen: vec![false; expected_vars + 1],
            expected_vars: expected_vars,
            expected_clauses: expected_clauses,
        }
    }
}

fn tokenise<'a, T>(
    it: &mut impl Iterator<Item = &'a str>,
    line: usize,
    desc: &'a str,
) -> Result<T, DimacsError>
where
    T: FromStr,
{
    match it.next() {
        None => Err(DimacsError::UnexpectedEOF),
        Some(t) => t.parse::<T>().or_else(|_| {
            Err(DimacsError::Error(
                line,
                format!("Expected {}, got {}", desc, t),
            ))
        }),
    }
}

/// Extracts a CNF structure (a vector of disjunction clauses) given a Read.
pub fn parse_from<S: std::io::Read>(src: S) -> Result<Vec<types::Clause>, DimacsError> {
    let lines = BufReader::new(src).lines();

    let mut state: Option<ParserState> = None; /* Is None until we get to the header (the 'p' clause) */

    let mut clauses: Vec<types::Clause> = Vec::new();

    let mut curr_literals: Vec<Literal> = Vec::new();

    for (line_num, line) in lines.enumerate() {
        let line = line?;
        let mut tokens = line.split_ascii_whitespace().peekable();

        match tokens.peek() {
            None => {
                /* We better not be in the midst of parsing a clause. */
                if curr_literals.len() > 0 {
                    return Err(DimacsError::UnexpectedEOF);
                }
                break;
            }
            Some(&"c") => {
                /* Drop comments; if we're in the middle of a clause, error out. */
                if curr_literals.len() > 0 {
                    return Err(DimacsError::Error(
                        line_num,
                        "Unterminated clause before comment line".to_string(),
                    ));
                }
                continue; /* eat the line */
            }
            Some(&"p") => {
                tokens.next(); /* Eat the "p", previously peeked-at */

                match tokens.next() {
                    None => return Err(DimacsError::UnexpectedEOF),
                    Some("cnf") => (),
                    Some(s) => {
                        return Err(DimacsError::Error(
                            line_num,
                            format!("Expected \"cnf\" file format type, got {}", s),
                        ))
                    }
                }
                state = match state {
                    None => {
                        let num_vars: usize = tokenise(&mut tokens, line_num, "num_vars")?;
                        let num_clauses: usize = tokenise(&mut tokens, line_num, "num_clauses")?;
                        Some(ParserState::new(line_num, num_vars, num_clauses))
                    }
                    Some(s) => {
                        return Err(DimacsError::Error(
                            line_num,
                            format!("Header already defined on line {}", s.header_line),
                        ))
                    }
                }
            }
            Some(_) => {
                match &mut state {
                    None => {
                        return Err(DimacsError::Error(
                                line_num,
                                format!("Saw a clause before a DIMACS header")));
                    }
                    Some(s) => {
                        /* TODO: this is kinda gnarly */
                        while let Some(_) = tokens.peek() {
                            let v: i64 = tokenise(&mut tokens, line_num, "var")?;
                            if v == 0 {
                                clauses.push(Clause::from_variables(curr_literals));
                                curr_literals = Vec::new();
                            } else {
                                let lit = Literal::from_dimacs_token(v);
                                if lit.var() > s.expected_vars {
                                    return Err(DimacsError::Error(line_num, format!("Var {v} > {}", s.expected_vars)));
                                }
                                s.vars_seen[lit.var()] = true;
                                curr_literals.push(lit);
                            }
                        }
                    }
                }

            }
        }
    }

    if clauses.len() == 0 {
        // TODO: are empty lines permitted?
        return Err(DimacsError::Error(0, String::from("No clauses!")));
    }
    // TODO: confirm the number of clauses matches what the header claims, and
    // perhaps also that the only variables we see are on [1..num_vars] ?

    Ok(clauses)
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Write};

    use tempfile::tempfile;

    use crate::types::{Clause, Literal};

    use super::*;

    #[test]
    fn empty_file() {
        let text = "c  simple_v3_c2.cnf
c
";
        let mut file = tempfile().expect("cnf file creation");
        file.write_all(text.as_bytes()).expect("cnf file write");
        parse_from(file).expect_err("Should complain about an empty file");
    }

    #[test]
    fn clauseless_file() {
        let text = "c  simple_v3_c2.cnf
p cnf 0 0
";
        let mut file = tempfile().expect("cnf file creation");
        file.write_all(text.as_bytes()).expect("cnf file write");
        parse_from(file).expect_err("Should complain about no clauses");
    }

    #[test]
    fn basics_str() {
        let text = "c  simple_v3_c2.cnf
c
p cnf 3 2
1 -3 0
2 3 -1 0
";
        let cursor = Cursor::new(text);
        let clauses = parse_from(cursor).expect("Parsing should succeed");
        assert!(clauses.len() == 2);
        assert!(clauses[0] == Clause::from_variables(vec![
                Literal::Positive(1), 
                Literal::Negative(3)]));
        assert!(clauses[1] == Clause::from_variables(vec![
                Literal::Positive(2), 
                Literal::Positive(3),
                Literal::Negative(1), 
                ]));
    }

    #[test]
    fn single_line_str() {
        // The same test as basics_str, but verifies that 0 delimits clauses
        // and not newlines.
        let text = "c  simple_v3_c2.cnf
p cnf 3 2
1 -3 0 2 3 -1 0
";
        let cursor = Cursor::new(text);
        let clauses = parse_from(cursor).expect("Parsing should succeed");
        assert!(clauses.len() == 2);
        assert!(clauses[0] == Clause::from_variables(vec![
                Literal::Positive(1), 
                Literal::Negative(3)]));
        assert!(clauses[1] == Clause::from_variables(vec![
                Literal::Positive(2), 
                Literal::Positive(3),
                Literal::Negative(1), 
                ]));
    }
}
