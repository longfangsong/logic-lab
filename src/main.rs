#![feature(box_syntax)]
#![feature(once_cell)]
#![feature(map_first_last)]
mod bdd;
mod formula;

use bdd::BDD;
use enum_dispatch::enum_dispatch;
use formula::and::AndOperand;
use formula::not::NotOperand;
use formula::or::OrOperand;
use formula::*;
use std::collections::{BTreeSet, HashMap};
#[enum_dispatch]
trait Evaluatable {
    fn eval(&self, ctx: &HashMap<String, bool>) -> bool;
}

impl<T> Evaluatable for Box<T>
where
    T: Evaluatable,
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
    let exp = expression::parse("false").unwrap().1;
    println!("{}", BDD::from_formula(&exp).reduce().dot());
}
