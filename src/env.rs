use super::data::{AtomVal, AtomType, c_nil};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use fnv::FnvHashMap;

#[derive(PartialEq)]
pub struct EnvType {
    parent: Option<Env>,
    data: FnvHashMap<Rc<String>, AtomVal>,
}

pub type Env = Rc<RefCell<EnvType>>;


impl fmt::Display for EnvType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str: Vec<String> = vec![];

        for (ref key, ref value) in self.data.iter() {
            str.push(format!("{} {}", key, value));
        }

        str.sort();

        write!(f, "{{{}}}", str.join(" "))
    }
}

impl fmt::Debug for EnvType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str: Vec<String> = vec![];

        for (ref key, ref value) in self.data.iter() {
            str.push(format!("{} {}", key, value.format(true)));
        }

        str.sort();

        write!(f, "Env {{ data: {{{}}} }}", str.join(" "))
    }
}


pub fn c_env(env: Option<Env>) -> Env {
    Rc::new(RefCell::new(EnvType {
        parent: env,
        data: FnvHashMap::default(),
    }))
}

fn env_find_inner(env: &Env, key: &Rc<String>) -> Option<(Env, AtomVal)> {
    let env_borrow = env.borrow();
    match env_borrow.data.get(key) {
        Some(value) => Some((env.clone(), value.clone())),
        None => {
            if let Some(ref parent) = env_borrow.parent {
                env_find_inner(parent, key)
            } else {
                None
            }
        }
    }
}

fn env_find(env: &Env, key: &AtomVal) -> Option<(Env, AtomVal)> {
    match **key {
        AtomType::Symbol(ref str) => env_find_inner(env, str),
        _ => None,
    }
}

pub fn env_set(env: &Env, key: &AtomVal, value: AtomVal) {
    match **key {
        AtomType::Symbol(ref str) => {
            env.borrow_mut().data.insert(str.clone(), value);
        }
        _ => unreachable!(),
    }
}

pub fn env_get(env: &Env, key: &AtomVal) -> Option<AtomVal> {
    env_find(env, key).map(|(_, value)| value)
}

pub fn env_bind(env: &Env, params: &[AtomVal], args: &[AtomVal]) {
    for (index, param) in params.iter().enumerate() {
        env_set(env, param, args.get(index).cloned().unwrap_or_else(c_nil));
    }
}

#[allow(unused_must_use)]
#[cfg(test)]
mod tests {
    use super::{c_env, env_set, env_get};
    use data::{c_symbol, c_int};

    #[test]
    fn test_c_env() {
        let env = c_env(None);

        assert_eq!(format!("{}", *env.borrow()), "{}");
    }

    #[test]
    fn test_set() {
        let env = c_env(None);
        env_set(&env, &c_symbol("Test"), c_int(10));
        env_set(&env, &c_symbol("Gra"), c_int(5));

        assert_eq!(format!("{}", *env.borrow()), "{Gra 5 Test 10}");
    }

    #[test]
    fn test_get() {
        let env = c_env(None);
        let key = c_symbol("Test");
        env_set(&env, &key, c_int(10));

        let child = c_env(Some(env));
        env_set(&child, &c_symbol("TestChild"), c_int(20));


        let grandchild = c_env(Some(child));

        assert_eq!(format!("{}", env_get(&grandchild, &key).unwrap()), "10");
        assert_eq!(format!("{}",
                           env_get(&grandchild, &c_symbol("TestChild")).unwrap()),
                   "20");
    }

    #[test]
    fn test_get_missing_value() {
        let env = c_env(None);

        assert!(env_get(&env, &c_symbol("Missing")).is_none());
    }
}
