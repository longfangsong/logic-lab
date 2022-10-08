use enum_dispatch::enum_dispatch;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::fold_many0;
use nom::sequence::preceded;
use nom::IResult;
use std::collections::HashMap;
use std::ops;

use super::{atom, in_brackets, not, Evaluative};
use crate::formula::atom::Atom;
use crate::formula::in_brackets::InBrackets;
use crate::formula::not::Not;

#[enum_dispatch(Evaluative)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AndOperand {
    Atom,
    InBrackets,
    Not,
    And(Box<And>),
}

impl<T> ops::BitAnd<T> for AndOperand
where
    T: Into<AndOperand>,
{
    type Output = And;

    fn bitand(self, rhs: T) -> Self::Output {
        And(self, rhs.into()).into()
    }
}

fn parse_higher_priority_operand(code: &str) -> IResult<&str, AndOperand> {
    alt((
        map(not::parse, AndOperand::Not),
        map(in_brackets::parse, AndOperand::InBrackets),
        map(atom::parse, AndOperand::Atom),
    ))(code)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct And(pub(crate) AndOperand, pub(crate) AndOperand);

impl Evaluative for And {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool {
        let And(lhs, rhs) = self;
        lhs.eval(ctx) && rhs.eval(ctx)
    }
}

pub fn parse(code: &str) -> IResult<&str, And> {
    let (rest, first) = parse_higher_priority_operand(code)?;
    let (rest, second) = preceded(tag("&"), parse_higher_priority_operand)(rest)?;
    fold_many0(
        preceded(tag("&"), parse_higher_priority_operand),
        move || And(first.clone(), second.clone()),
        |acc, next| And(AndOperand::And(Box::new(acc)), next),
    )(rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            parse("x&y").unwrap(),
            (
                "",
                And(
                    AndOperand::Atom(Atom::Variable("x".to_string())),
                    AndOperand::Atom(Atom::Variable("y".to_string()))
                )
            )
        );
        assert_eq!(
            parse("x&!y").unwrap(),
            (
                "",
                And(
                    AndOperand::Atom(Atom::Variable("x".to_string())),
                    AndOperand::Not(not::Not(not::NotOperand::Atom(Atom::Variable(
                        "y".to_string()
                    ))))
                )
            )
        );
        assert_eq!(
            parse("x&y&z").unwrap(),
            (
                "",
                And(
                    AndOperand::And(box And(
                        AndOperand::Atom(Atom::Variable("x".to_string())),
                        AndOperand::Atom(Atom::Variable("y".to_string()))
                    )),
                    AndOperand::Atom(Atom::Variable("z".to_string()))
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
        let x_and_y = parse("x&y").unwrap().1;
        let x_and_not_y = parse("x&!y").unwrap().1;
        let x_and_y_and_z = parse("x&y&z").unwrap().1;
        assert_eq!(x_and_y.eval(&ctx), false);
        assert_eq!(x_and_not_y.eval(&ctx), true);
        assert_eq!(x_and_y_and_z.eval(&ctx), false);
    }
}
