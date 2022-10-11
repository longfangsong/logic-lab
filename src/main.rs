#![feature(box_syntax)]
#![feature(once_cell)]
#![feature(map_first_last)]
mod binary_decision_diagram;

// It is strange to assert the result of `eval` directly with `assert!`
#[allow(clippy::bool_assert_comparison)]
mod formula;

use binary_decision_diagram::BinaryDecisionDiagram;
use enum_dispatch::enum_dispatch;
use formula::and::AndOperand;
use formula::not::NotOperand;
use formula::or::OrOperand;
use formula::*;
use std::collections::{BTreeSet, HashMap};
#[enum_dispatch]
trait Evaluable {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool;
}

impl<T> Evaluable for Box<T>
where
    T: Evaluable,
{
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool {
        Box::as_ref(self).eval(ctx)
    }
}

#[enum_dispatch]
trait ContainVariable {
    fn variables(&self) -> BTreeSet<String>;
}

impl<T> ContainVariable for Box<T>
where
    T: ContainVariable,
{
    fn variables(&self) -> BTreeSet<String> {
        Box::as_ref(self).variables()
    }
}

fn main() {
    let exp1 = expression::parse("a&b").unwrap().1;
    let exp2 = expression::parse("a&c").unwrap().1;
    let bdd1 = BinaryDecisionDiagram::from_formula(&exp1);
    let bdd2 = BinaryDecisionDiagram::from_formula(&exp2);
    let mut bdd3 = bdd1.apply(&bdd2, |a, b| a | b).reduce();
    bdd3.restrict("b", false);
    println!("{}", bdd3.dot());
}
