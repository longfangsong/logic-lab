use nom::{
    branch::alt, bytes::complete::tag, character::complete::alpha1, combinator::map, IResult,
};

use crate::{ContainVariable, Evaluable};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Atom {
    Variable(String),
    Const(bool),
}

impl Evaluable for Atom {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool {
        match self {
            Atom::Variable(x) => *ctx.get(x).unwrap(),
            Atom::Const(c) => *c,
        }
    }
}

impl ContainVariable for Atom {
    fn variables(&self) -> BTreeSet<String> {
        match self {
            Atom::Variable(x) => [x.clone()].into_iter().collect(),
            Atom::Const(_) => BTreeSet::new(),
        }
    }
}

pub fn parse(code: &str) -> IResult<&str, Atom> {
    alt((
        map(tag("0"), |_| Atom::Const(false)),
        map(tag("false"), |_| Atom::Const(false)),
        map(tag("1"), |_| Atom::Const(true)),
        map(tag("true"), |_| Atom::Const(true)),
        map(alpha1, |name: &str| Atom::Variable(name.to_string())),
    ))(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(parse("0").unwrap(), ("", Atom::Const(false)));
        assert_eq!(parse("false").unwrap(), ("", Atom::Const(false)));
        assert_eq!(parse("1").unwrap(), ("", Atom::Const(true)));
        assert_eq!(parse("true").unwrap(), ("", Atom::Const(true)));
        assert_eq!(parse("x").unwrap(), ("", Atom::Variable("x".to_string())));
        assert_eq!(parse("y").unwrap(), ("", Atom::Variable("y".to_string())));
    }

    #[test]
    fn test_eval() {
        let mut ctx = HashMap::new();
        ctx.insert("x".to_string(), true);
        ctx.insert("y".to_string(), false);
        let f = Atom::Const(false);
        assert_eq!(f.eval(&ctx), false);
        let t = Atom::Const(true);
        assert_eq!(t.eval(&ctx), true);
        let x = Atom::Variable("x".to_string());
        assert_eq!(x.eval(&ctx), true);
        let y = Atom::Variable("y".to_string());
        assert_eq!(y.eval(&ctx), false);
    }

    #[test]
    fn test_variables() {
        let result = parse("x").unwrap().1.variables();
        assert!(result.contains("x"));
        assert!(!result.contains("y"));
        let result = parse("1").unwrap().1.variables();
        assert!(!result.contains("x"));
        assert!(!result.contains("y"));
    }
}
