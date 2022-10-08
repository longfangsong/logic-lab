use std::collections::HashMap;
use std::ops;

use super::{and, atom, in_brackets, not, Evaluative};
use crate::formula::and::And;
use crate::formula::atom::Atom;
use crate::formula::in_brackets::InBrackets;
use crate::formula::not::Not;
use enum_dispatch::enum_dispatch;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::fold_many0;
use nom::sequence::preceded;
use nom::IResult;
#[enum_dispatch(Evaluative)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OrOperand {
    Atom,
    InBrackets,
    Not,
    And,
    Or(Box<Or>),
}

impl<T> ops::BitOr<T> for OrOperand
where
    T: Into<OrOperand>,
{
    type Output = Or;

    fn bitor(self, rhs: T) -> Self::Output {
        Or(self, rhs.into()).into()
    }
}

fn parse_higher_priority_operand(code: &str) -> IResult<&str, OrOperand> {
    alt((
        map(and::parse, OrOperand::And),
        map(not::parse, OrOperand::Not),
        map(in_brackets::parse, OrOperand::InBrackets),
        map(atom::parse, OrOperand::Atom),
    ))(code)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Or(pub(crate) OrOperand, pub(crate) OrOperand);

impl Evaluative for Or {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool {
        let Or(lhs, rhs) = self;
        lhs.eval(ctx) || rhs.eval(ctx)
    }
}

pub fn parse(code: &str) -> IResult<&str, Or> {
    let (rest, first) = parse_higher_priority_operand(code)?;
    let (rest, second) = preceded(tag("|"), parse_higher_priority_operand)(rest)?;
    fold_many0(
        preceded(tag("|"), parse_higher_priority_operand),
        move || Or(first.clone(), second.clone()),
        |acc, next| Or(OrOperand::Or(Box::new(acc)), next),
    )(rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            parse("x|y").unwrap(),
            (
                "",
                Or(
                    OrOperand::Atom(Atom::Variable("x".to_string())),
                    OrOperand::Atom(Atom::Variable("y".to_string()))
                )
            )
        );
        assert_eq!(
            parse("x|y&z").unwrap(),
            (
                "",
                Or(
                    OrOperand::Atom(Atom::Variable("x".to_string())),
                    OrOperand::And(and::And(
                        and::AndOperand::Atom(Atom::Variable("y".to_string())),
                        and::AndOperand::Atom(Atom::Variable("z".to_string()))
                    ))
                )
            )
        );
    }

    #[test]
    fn test_eval() {
        let mut ctx = HashMap::new();
        ctx.insert("x".to_string(), true);
        ctx.insert("y".to_string(), false);
        ctx.insert("z".to_string(), true);
        let x_or_y = parse("x|y").unwrap().1;
        let x_or_y_and_z = parse("x|y&z").unwrap().1;
        assert_eq!(x_or_y.eval(&ctx), true);
        assert_eq!(x_or_y_and_z.eval(&ctx), true);
        ctx.insert("x".to_string(), false);
        assert_eq!(x_or_y_and_z.eval(&ctx), false);
    }
}
