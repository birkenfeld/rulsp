use std::fs::File;
use std::io::prelude::*;

use env::{c_env, env_set, Env};
use data::{AtomVal, AtomType, AtomRet, c_int, c_nil, c_list, c_symbol, c_func};
use eval::eval_str;

fn safe_get(args: &[AtomVal], index: usize) -> AtomVal {
    args.get(index).cloned().unwrap_or_else(c_nil)
}

fn int_fold_op<F>(f: F, empty: i64, args: &[AtomVal]) -> AtomRet
    where F: Fn(i64, i64) -> i64
{
    let mut it = args.iter();
    match it.next() {
        None => Ok(c_int(empty)),
        Some(acc) => {
            let mut acc = acc.get_int()?;
            for arg in it {
                let arg = arg.get_int()?;
                acc = f(acc, arg);
            }
            Result::Ok(c_int(acc))
        }
    }
}

fn add(args: &[AtomVal]) -> AtomRet {
    int_fold_op(|acc, v| acc + v, 0, args)
}

fn sub(args: &[AtomVal]) -> AtomRet {
    int_fold_op(|acc, v| acc - v, 0, args)
}

fn mul(args: &[AtomVal]) -> AtomRet {
    int_fold_op(|acc, v| acc * v, 1, args)
}

fn div(args: &[AtomVal]) -> AtomRet {
    int_fold_op(|acc, v| acc / v, 1, args)
}

fn cons(args: &[AtomVal]) -> AtomRet {
    let mut list = vec![safe_get(args, 0)];
    list.extend(safe_get(args, 1).get_list()?.iter().cloned());
    Ok(c_list(list))
}

fn list(args: &[AtomVal]) -> AtomRet {
    Ok(c_list(args.to_vec()))
}

fn is_list(args: &[AtomVal]) -> AtomRet {
    match *safe_get(args, 0) {
        AtomType::List(_) => Ok(c_int(1)),
        _ => Ok(c_nil()),
    }
}

fn is_nil(args: &[AtomVal]) -> AtomRet {
    match *safe_get(args, 0) {
        AtomType::Nil => Ok(c_int(1)),
        _ => Ok(c_nil()),
    }
}


fn count(args: &[AtomVal]) -> AtomRet {
    Ok(c_int(safe_get(args, 0).get_list()?.len() as i64))
}


fn nth(args: &[AtomVal]) -> AtomRet {
    trace!("action=nth args={:?}", args);
    let n = safe_get(args, 1).get_int().unwrap_or(0);

    Ok(safe_get(safe_get(args, 0).get_list()?, n as usize))
}


fn rest(args: &[AtomVal]) -> AtomRet {
    match safe_get(args, 0).get_list() {
        Ok(seq) => {
            if seq.len() > 0 {
                Ok(c_list(seq[1..seq.len()].iter().cloned().collect()))
            } else {
                Ok(c_list(vec![]))
            }
        }
        _ => Ok(c_nil()),
    }
}

fn partialeq(args: &[AtomVal]) -> AtomRet {
    let mut output = c_int(1);
    for (i, arg) in args.iter().enumerate() {
        let next_arg = args.get(i + 1);
        if next_arg.is_some() {
            if next_arg.unwrap() != arg {
                output = c_nil();
            };
        }
    }

    Ok(output)
}


fn format_args(args: &[AtomVal], format: bool) -> String {
    args.iter()
        .map(|ref v| v.format(format))
        .collect::<Vec<_>>()
        .join(" ")
}

fn println(args: &[AtomVal]) -> AtomRet {
    println!("{}", format_args(&args, false));
    Ok(safe_get(args, 0))
}

fn print(args: &[AtomVal]) -> AtomRet {
    print!("{}", format_args(&args, false));
    Ok(safe_get(args, 0))
}

fn _println(args: &[AtomVal]) -> AtomRet {
    println!("{}", format_args(&args, true));
    Ok(safe_get(args, 0))
}

fn _print(args: &[AtomVal]) -> AtomRet {
    print!("{}", format_args(&args, true));
    Ok(safe_get(args, 0))
}


#[allow(unused_must_use)]
pub fn build() -> Env {
    let env = c_env(None);

    env_set(&env, &c_symbol("print"), c_func(print));
    env_set(&env, &c_symbol("println"), c_func(println));
    env_set(&env, &c_symbol("_print"), c_func(_print));
    env_set(&env, &c_symbol("_println"), c_func(_println));
    env_set(&env, &c_symbol("+"), c_func(add));
    env_set(&env, &c_symbol("-"), c_func(sub));
    env_set(&env, &c_symbol("*"), c_func(mul));
    env_set(&env, &c_symbol("/"), c_func(div));
    env_set(&env, &c_symbol("cons"), c_func(cons));
    env_set(&env, &c_symbol("list"), c_func(list));
    env_set(&env, &c_symbol("list?"), c_func(is_list));
    env_set(&env, &c_symbol("nil?"), c_func(is_nil));
    env_set(&env, &c_symbol("nth"), c_func(nth));
    env_set(&env, &c_symbol("rest"), c_func(rest));
    env_set(&env, &c_symbol("count"), c_func(count));

    // predicates
    env_set(&env, &c_symbol("="), c_func(partialeq));
    // env_set(&env, &c_symbol("="), c_func(partialeq));


    let mut f = File::open("src/core.clrs").expect("core.clrs has to be openable");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Couldn't read core.clrs");

    eval_str(s.as_str(), &env).expect("Problem loading core.clrs into ENV");

    env
}

#[cfg(test)]
mod tests {
    use super::add;
    use data::c_int;
    use test::Bencher;

    #[bench]
    fn bench_adding(b: &mut Bencher) {
        let args = vec![c_int(1), c_int(1)];

        b.iter(|| add(&args));
    }

}
