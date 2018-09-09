#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::*;
use std::io::BufRead;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct Calculator;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left),
            Operator::new(power, Right)
        ])
    };
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => pair.as_str().parse::<f64>().unwrap(),
            Rule::expr => eval(pair.into_inner()),
            _ => unreachable!(),
        },
        |lhs: f64, op: Pair<Rule>, rhs: f64| match op.as_rule() {
            Rule::add      => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide   => lhs / rhs,
            Rule::power    => lhs.powf(rhs),
            _ => unreachable!(),
        },
    )
}

fn main() {
    let stdin = std::io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let parse_result = Calculator::parse(Rule::calculation, &line);

        match parse_result {
            Ok(calc) => println!(" = {}", eval(calc)),
            Err(_) => println!(" Syntax error"),
        }
    }
}
