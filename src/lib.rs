#![feature(box_syntax)]
#![feature(once_cell)]
#![feature(map_first_last)]
mod binary_decision_diagram;

// It is strange to assert the result of `eval` directly with `assert!`
#[allow(clippy::bool_assert_comparison)]
mod formula;

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

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
