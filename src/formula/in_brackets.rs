use std::collections::BTreeSet;
use std::collections::HashMap;

use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::delimited;
use nom::IResult;

use super::expression;
use super::Expression;
use crate::{ContainVariable, Evaluatable};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InBrackets(pub Box<Expression>);

impl Evaluatable for InBrackets {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool {
        self.0.eval(ctx)
    }
}

impl ContainVariable for InBrackets {
    fn variables(&self) -> BTreeSet<String> {
        self.0.variables()
    }
}

pub fn parse(code: &str) -> IResult<&str, InBrackets> {
    map(
        delimited(tag("("), expression::parse, tag(")")),
        |expression| InBrackets(Box::new(expression)),
    )(code)
}

#[cfg(test)]
mod tests {
    use crate::formula::{And, Atom};

    use crate::Evaluatable;
    use std::collections::HashMap;

    use crate::formula::and::AndOperand;

    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            parse("(a&b)"),
            Ok((
                "",
                InBrackets(box Expression::And(And(
                    AndOperand::Atom(Atom::Variable("a".to_string())),
                    AndOperand::Atom(Atom::Variable("b".to_string()))
                )))
            ))
        );
    }

    #[test]
    fn test_eval() {
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), true);
        ctx.insert("b".to_string(), false);
        assert_eq!(parse("(a&b)").unwrap().1.eval(&ctx), false);
    }
}
