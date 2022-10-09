use super::and::{self, And};
use super::atom::{self, Atom};
use super::in_brackets::{self, InBrackets};
use super::not::{self, Not};
use super::or::{self, Or};
use enum_dispatch::enum_dispatch;
use nom::branch::alt;
use nom::combinator::map;
use nom::IResult;

#[enum_dispatch(Evaluatable, ContainVariable)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Atom,
    InBrackets,
    Not,
    And,
    Or,
}

pub fn parse(code: &str) -> IResult<&str, Expression> {
    alt((
        map(or::parse, Expression::Or),
        map(and::parse, Expression::And),
        map(not::parse, Expression::Not),
        map(in_brackets::parse, Expression::InBrackets),
        map(atom::parse, Expression::Atom),
    ))(code)
}

#[cfg(test)]
mod tests {
    use crate::{ContainVariable, Evaluatable};

    use std::collections::HashMap;

    use crate::formula::{and::AndOperand, not::NotOperand, or::OrOperand};

    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            parse("!(a&b)|c"),
            Ok((
                "",
                Expression::Or(Or(
                    OrOperand::Not(Not(NotOperand::InBrackets(InBrackets(
                        box Expression::And(And(
                            AndOperand::Atom(Atom::Variable("a".to_string())),
                            AndOperand::Atom(Atom::Variable("b".to_string()))
                        ))
                    )))),
                    OrOperand::Atom(Atom::Variable("c".to_string()))
                ))
            ))
        );
    }

    #[test]
    fn test_eval() {
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), true);
        ctx.insert("b".to_string(), false);
        ctx.insert("c".to_string(), true);
        assert_eq!(parse("!(a&b)|c").unwrap().1.eval(&ctx), true);
    }

    #[test]
    fn test_variables() {
        let result = parse("!(a&b)|c").unwrap().1.variables();
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains("c"));
        assert!(!result.contains("d"));
    }
}
