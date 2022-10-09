use std::collections::{BTreeSet, HashMap};

use enum_dispatch::enum_dispatch;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;

use super::atom::Atom;
use super::in_brackets::InBrackets;
use super::{atom, in_brackets};

use crate::{ContainVariable, Evaluatable};

#[enum_dispatch(Evaluatable, ContainVariable)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NotOperand {
    Atom,
    InBrackets,
    Not(Box<Not>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Not(pub(crate) NotOperand);

impl Evaluatable for Not {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool {
        !self.0.eval(ctx)
    }
}

impl ContainVariable for Not {
    fn variables(&self) -> BTreeSet<String> {
        self.0.variables()
    }
}

pub fn parse(code: &str) -> IResult<&str, Not> {
    preceded(
        tag("!"),
        alt((
            map(atom::parse, |x| Not(NotOperand::Atom(x))),
            map(in_brackets::parse, |x| Not(NotOperand::InBrackets(x))),
            map(parse, |x| Not(NotOperand::Not(Box::new(x)))),
        )),
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            parse("!x").unwrap(),
            ("", Not(NotOperand::Atom(Atom::Variable("x".to_string())),))
        );
        assert_eq!(
            parse("!!x").unwrap(),
            (
                "",
                Not(NotOperand::Not(box Not(NotOperand::Atom(Atom::Variable(
                    "x".to_string()
                )))))
            )
        );
    }

    #[test]
    fn test_eval() {
        let mut ctx = HashMap::new();
        ctx.insert("x".to_string(), true);
        let not_x = parse("!x").unwrap().1;
        let not_not_x = parse("!!x").unwrap().1;
        assert_eq!(not_x.eval(&ctx), false);
        assert_eq!(not_not_x.eval(&ctx), true);
        ctx.insert("x".to_string(), false);
        assert_eq!(not_x.eval(&ctx), true);
        assert_eq!(not_not_x.eval(&ctx), false);
    }

    #[test]
    fn test_variables() {
        let result = parse("!x").unwrap().1.variables();
        assert!(result.contains("x"));
        assert!(!result.contains("y"));
    }
}
