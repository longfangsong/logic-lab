use enum_dispatch::enum_dispatch;
use std::collections::HashMap;

pub(in crate::formula) mod and;
pub(in crate::formula) mod atom;
pub(in crate::formula) mod expression;
pub(in crate::formula) mod in_brackets;
pub(in crate::formula) mod not;
pub(in crate::formula) mod or;

pub use and::And;
pub use atom::Atom;
pub use expression::Expression;
pub use in_brackets::InBrackets;
pub use not::Not;
pub use or::Or;

pub use expression::parse;

use and::AndOperand;
use not::NotOperand;
use or::OrOperand;

#[enum_dispatch]
trait Evaluative {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool;
}

impl<T> Evaluative for Box<T>
where
    T: Evaluative,
{
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool {
        Box::as_ref(&self).eval(ctx)
    }
}
