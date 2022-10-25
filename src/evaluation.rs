use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use crate::adventure::{Comparison, Record};

#[derive(PartialEq, Debug)]
pub enum EvaluationError {
    DivisionByZero,
    NotANumber(String),
    InvalidDieExpression(String),
    MissingDicePoolEvaluator(String),
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::DivisionByZero => write!(f, "Division by 0 error"),
            EvaluationError::NotANumber(n) => write!(f, "{} is not a number", n),
            EvaluationError::InvalidDieExpression(n) => write!(
                f,
                "{} is not a valid dice expression, use something like 1d6",
                n
            ),
            EvaluationError::MissingDicePoolEvaluator(n) => write!(
                f,
                "{} is not a valid dice pool expression, use something like 4d6p4",
                n
            ),
        }
    }
}
/// Evaluates expression into a number, taking care of randomness and record evaluation
///
/// # Errors
/// If the expression can't be evaluated or contains undefined records or calculations then an error will be returned instead.
pub fn evaluate_expression(
    exp: &str,
    records: &HashMap<String, Record>,
    rand: &mut Random,
) -> Result<i32, EvaluationError> {
    let tokens: Vec<&str> = exp.split_inclusive(&['+', '-', '*', '/'][..]).collect();
    // this function evaluates name of a record into its value, it defaults to 0 on records not found
    // Although, record not found should probably result in an error instead of 0
    let eval_rec = |x: &str| {
        let expected = x.replace("[", "").replace("]", "");
        if let Some(v) = records.get(&expected) {
            return v.value_as_string();
        }
        return "0".to_string();
    };
    // This closure turns a die expression into evaluated form <i32> based on parameters
    let mut eval_die = |x: &str, typ: char, pool: Option<char>| {
        // gathering slit marks with optional pool mark
        let mut cut = vec![typ];
        if let Some(p) = pool {
            cut.push(p);
        }
        let r: Vec<i32> = match x
            .split(&cut[..])
            .map(|x| {
                if let Ok(ok) = x.parse() {
                    Ok(ok)
                } else {
                    Err(EvaluationError::NotANumber(x.to_string()))
                }
            })
            .collect()
        {
            Ok(x) => x,
            Err(x) => return Err(x),
        };

        // need the result to be 2 or 3 parts, any other is an error
        if pool == None {
            if r.len() != 2 {
                return Err(EvaluationError::InvalidDieExpression(x.to_string()));
            }
        } else {
            if r.len() != 3 {
                return Err(EvaluationError::MissingDicePoolEvaluator(x.to_string()));
            }
        }

        // matching types to dice rolls
        match typ {
            'd' => match pool {
                None => Ok(rand.die(r[0], r[1])),
                Some('p') => Ok(rand.pool(r[0], r[1], r[2])),
                Some('q') => Ok(rand.pool_reverse(r[0], r[1], r[2])),
                _ => unreachable!(),
            },
            'x' => Ok(rand.die_explode(r[0], r[1])),
            _ => unreachable!(),
        }
    };

    let mut eval_exp = |x: &str| {
        let ev1;
        let ev2;

        if x.contains('d') {
            ev1 = 'd';
        } else if x.contains('x') {
            ev1 = 'x';
        } else {
            // if there is no main random number generation keyword then we treat this as a constant value
            if let Ok(v) = x.parse() {
                return Ok(v);
            } else {
                return Err(EvaluationError::NotANumber(x.to_string()));
            }
        }
        if x.contains('p') {
            ev2 = Some('p');
        } else if x.contains('q') {
            ev2 = Some('q');
        } else {
            ev2 = None;
        }

        eval_die(x, ev1, ev2)
    };

    let mut ops = Vec::new();

    // going through all the tokens and converting them into operations
    for tok in tokens {
        if tok == "-" {
            // special handing of -1 situations, like in expression 1d20*-1
            ops.push((-1, '*', 2));
            continue;
        }
        let mut exp: String;
        let op;
        let op_priority;
        // First we asses what operation the token needs to perform
        // If the token doesn't have an operation, then it's assumed it's the last token
        match tok.chars().last().unwrap() {
            '+' => {
                op = '+';
                op_priority = 1;
            }
            '-' => {
                op = '-';
                op_priority = 1
            }
            '*' => {
                op = '*';
                op_priority = 2
            }
            '/' => {
                op = '/';
                op_priority = 2
            }
            _ => {
                op = ' ';
                op_priority = 0
            }
        }
        // next we need to get rid of the operation from the string so it can be evaluated
        exp = tok.to_string();
        if op_priority > 0 {
            exp.replace_range(tok.len() - 1..tok.len(), "");
            exp = exp.trim().to_string();
        }

        // test if the token has a record name in it, and turn it into a number
        while exp.contains('[') {
            let start = exp.find(|x| x == '[').unwrap();
            let end = exp.find(|x| x == ']').unwrap();
            let val = &exp[start..=end];
            let ev = eval_rec(val);
            exp.replace_range(start..=end, &ev);
        }

        // If we have l or h keywords in the die roll expression, that measn we have to choose lower or higher of the expression
        if exp.contains(&['l', 'h'][..]) {
            // splitting the roll expression into individual rolls
            let mut split_exp: VecDeque<String> = exp
                .split_inclusive(&['l', 'h'][..])
                .map(|x| x.to_string())
                .collect();
            loop {
                // we take first and next expression to evaluate together
                let mut this = split_exp.pop_front().unwrap();
                let mut next = split_exp.pop_front().unwrap();
                // last char of the expression is the evaluation type
                let hi_or_lo = this.chars().last().unwrap();
                let hi_or_lo_next = next.chars().last().unwrap();
                let this_value;
                let next_value;

                // dropping the last char since it's not part of the die roll itself
                this.replace_range(this.len() - 1..this.len(), "");
                if split_exp.len() > 0 {
                    next.replace_range(next.len() - 1..next.len(), "");
                }
                // evaluating expressions into their values
                match eval_exp(&this) {
                    Ok(v) => this_value = v,
                    Err(e) => return Err(e),
                }
                match eval_exp(&next) {
                    Ok(v) => next_value = v,
                    Err(e) => return Err(e),
                }
                // now we obtain the final result
                let res = match hi_or_lo {
                    'l' => i32::min(next_value, this_value),
                    'h' => i32::max(next_value, this_value),
                    _ => unreachable!(),
                };
                // if it's not the last expression in our chain, then we reinsert the expression back to be evaluated with the next one
                if split_exp.len() > 0 {
                    split_exp.insert(0, format!("{res}{hi_or_lo_next}"));
                } else {
                    // if it is the last one then we push it to operations and end the loop
                    ops.push((res, op, op_priority));
                    break;
                }
            }
        } else {
            match eval_exp(&exp) {
                Ok(v) => ops.push((v, op, op_priority)),
                Err(e) => return Err(e),
            }
        }
    }

    // going through operations, evaluating one at a time until only one remains or we encounter error
    let mut i = 0;
    loop {
        // if it's the only operation remaining, then we have the result
        if ops.len() == 1 {
            return Ok(ops[0].0);
        }
        // if we reached the end of the operations, we wrap around and start from the beginning
        if i == ops.len() - 1 {
            i = 0;
        }
        // peeking at operations to see if we can calculate something
        let l = &ops[i];
        let r = &ops[i + 1];
        if l.2 >= r.2 {
            // priorities don't collide, so we calculate
            let r = match l.1 {
                '+' => (l.0 + r.0, r.1, r.2),
                '-' => (l.0 - r.0, r.1, r.2),
                '*' => (l.0 * r.0, r.1, r.2),
                '/' => {
                    if r.0 == 0 {
                        return Err(EvaluationError::DivisionByZero);
                    }
                    (l.0 / r.0, r.1, r.2)
                }
                _ => unreachable!(),
            };
            ops.remove(i + 1);
            ops.remove(i);
            ops.insert(i, r);
        } else {
            // there's mismatch in priorities, so we skip to the next operation
            i += 1;
        }
    }
}
/// Evaluates two expressions and compares them to each other.
///
/// If any of the expressions can't be evaluated, error is returned
pub fn evaluate_and_compare(
    lhe: &str,
    rhe: &str,
    comp: &Comparison,
    records: &HashMap<String, Record>,
    rand: &mut Random,
) -> Result<bool, EvaluationError> {
    let l;
    let r;
    match evaluate_expression(lhe, records, rand) {
        Ok(v) => l = v,
        Err(e) => return Err(e),
    }
    match evaluate_expression(rhe, records, rand) {
        Ok(v) => r = v,
        Err(e) => return Err(e),
    }
    return Ok(comp.compare(l, r));
}
pub struct Random {
    generator: StdRng,
}
impl Random {
    pub fn new(seed: u64) -> Self {
        Self {
            generator: StdRng::seed_from_u64(seed),
        }
    }
    pub fn die(&mut self, amount: i32, sides: i32) -> i32 {
        assert!(amount > 0);
        assert!(sides > 0);
        let min = amount;
        let max = amount * sides;
        self.generator.gen_range(min..=max)
    }
    pub fn pool(&mut self, amount: i32, sides: i32, threshold: i32) -> i32 {
        assert!(sides > 0);
        assert!(amount > 0);
        assert!(threshold > 0);
        let mut res = 0;
        for _ in 0..amount {
            if self.die(1, sides) >= threshold {
                res += 1;
            }
        }
        res
    }
    pub fn pool_reverse(&mut self, amount: i32, sides: i32, threshold: i32) -> i32 {
        assert!(sides > 0);
        assert!(amount > 0);
        assert!(threshold > 0);

        let mut res = 0;
        for _ in 0..amount {
            if self.die(1, sides) <= threshold {
                res += 1;
            }
        }
        res
    }
    pub fn die_explode(&mut self, amount: i32, sides: i32) -> i32 {
        assert!(amount > 0);
        assert!(sides > 0);

        let mut counter = 0;
        for _ in 0..amount {
            loop {
                let r = self.die(1, sides);
                counter += r;
                if r != sides {
                    break;
                }
            }
        }
        counter
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::adventure::{Comparison, Record};

    use super::{evaluate_and_compare, evaluate_expression, Random};

    #[test]
    fn evex_dice_regular() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let mut records = HashMap::<String, Record>::new();
        records.insert("strength".to_string(), {
            let mut r = Record::parse_from_string("strength".to_string()).unwrap();
            r.value = 4;
            r
        });
        assert_eq!(
            test.die(1, 4),
            evaluate_expression("1d4", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_record_dice() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let mut records = HashMap::<String, Record>::new();
        records.insert("strength".to_string(), {
            let mut r = Record::parse_from_string("strength".to_string()).unwrap();
            r.value = 4;
            r
        });

        assert_eq!(
            test.die(4, 6),
            evaluate_expression("[strength]d6", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_record_sides() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let mut records = HashMap::<String, Record>::new();
        records.insert("strength".to_string(), {
            let mut r = Record::parse_from_string("strength".to_string()).unwrap();
            r.value = 4;
            r
        });

        assert_eq!(
            test.die(6, 4),
            evaluate_expression("6d[strength]", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_pool() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            test.pool(2, 6, 4),
            evaluate_expression("2d6p4", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_pool_reverse() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            test.pool_reverse(2, 6, 4),
            evaluate_expression("2d6q4", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_exploding() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            test.die_explode(2, 6),
            evaluate_expression("2x6", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_adddition() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            test.die(1, 10) + 5,
            evaluate_expression("1d10+5", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dicedivision() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            test.die(2, 4) / 2,
            evaluate_expression("2d4/2", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_multiplication() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            test.die(1, 4) * test.die(1, 4),
            evaluate_expression("1d4*1d4", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_take_less() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            i32::min(test.die(1, 20), test.die(1, 20)),
            evaluate_expression("1d20l1d20", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_take_greater() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            i32::max(test.die(1, 20), test.die(1, 20)),
            evaluate_expression("1d20h1d20", &records, &mut rand).unwrap()
        );
    }
    #[test]
    fn evaluate_dice_long() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            evaluate_expression("1d20+5*2/3-1", &records, &mut rand).unwrap(),
            test.die(1, 20) + 5 * 2 / 3 - 1
        );
    }
    #[test]
    fn evaluate_dice_negative() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);

        let records = HashMap::<String, Record>::new();
        assert_eq!(
            evaluate_expression("1d20*-1", &records, &mut rand).unwrap(),
            test.die(1, 20) * -1
        );
    }

    #[test]
    fn deterministic_random() {
        let mut r = Random::new(1234567890);
        let mut l = Random::new(1234567890);
        assert_eq!(r.die(1, 20), l.die(1, 20));
        assert_eq!(r.die_explode(1, 6), l.die_explode(1, 6));
        assert_eq!(r.pool(4, 10, 6), l.pool(4, 10, 6));
    }
    #[test]
    fn random_die_in_range() {
        let mut r = Random::new(1234567890);
        for _ in 0..100 {
            let v = r.die(1, 20);
            assert!(v >= 1 && v <= 20);
        }
    }
    #[test]
    fn random_die_pool_in_range() {
        let mut r = Random::new(1234567890);
        for _ in 0..100 {
            let v = r.pool(4, 6, 4);
            assert!(v >= 0 && v <= 6);
        }
    }

    #[test]
    fn evaluate_compare() {
        let mut rand = Random::new(69420);
        let mut test = Random::new(69420);
        let records = HashMap::<String, Record>::new();

        for _ in 0..10 {
            let c = evaluate_and_compare("1d20", "1d10", &Comparison::Less, &records, &mut rand)
                .unwrap();
            assert_eq!(c, test.die(1, 20) < test.die(1, 10));
        }
    }
}
