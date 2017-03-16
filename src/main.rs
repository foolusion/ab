extern crate sha1;
extern crate byteorder;
extern crate rand;

use byteorder::{ByteOrder, BigEndian};
use rand::Rng;

fn main() {
    println!("Hello, world!");
    let e = Experiment {
        name: "this",
        namespace: "namespace",
        params: vec![Param {
                         name: "p1",
                         choices: Choices::Uniform(vec!["a", "b"]),
                     }],
        segments: vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                       255],
    };
    let e2 = Experiment {
        name: "that",
        namespace: "arstneio",
        params: vec![Param {
                         name: "my-param",
                         choices: Choices::Weighted(vec![("a", 1.0), ("b", 2.0)]),
                     }],
        segments: vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                       255],
    };

    let mut scores = std::collections::HashMap::new();
    for i in 0..60000 {
        let userid = gen_name(10);
        let exp1 = match eval_test(&e2, &userid) {
            Ok(exp) => exp,
            Err(e) => {println!("{}", e); return},
        };
        for p in exp1.params.iter() {
            let key = format!("{}.{}", p.name, p.choice);
            let count = scores.entry(key).or_insert(0);
            *count += 1;
        }
        let exp2 = match eval_test(&e, &userid) {
            Ok(exp) => exp,
            Err(e) => { println!("{}", e); return},
        };
        for p in exp2.params.iter() {
            let key = format!("{}.{}", p.name, p.choice);
            let count = scores.entry(key).or_insert(0);
            *count += 1;
        }
    }
    println!("{:?}", scores)
}

fn gen_name<'a>(len: i32) -> String {
    let alpha = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
    let mut rng = rand::thread_rng();
    let mut out: String = String::new();
    for i in 0..len {
        match alpha.bytes().nth(rng.gen::<usize>()%alpha.len()) {
            Some(c) => out.push(c as char),
            None => return "".to_string(),
        }
    };
    return out
}

struct Experiment<'a> {
    name: &'a str,
    namespace: &'a str,
    params: Vec<Param<'a>>,
    segments: Vec<u8>,
}

struct Param<'a> {
    name: &'a str,
    choices: Choices<'a>,
}

enum Choices<'a> {
    Weighted(Vec<(&'a str, f64)>),
    Uniform(Vec<(&'a str)>),
}

#[derive(Debug)]
struct Experience<'a> {
    name: &'a str,
    namespace: &'a str,
    params: Vec<ParamExperience<'a>>,
}

#[derive(Debug)]
// ParamExperience is a result from hashing the user and determining their experience.
struct ParamExperience<'a> {
    name: &'a str,
    choice: &'a str,
}

fn eval_test<'a, 'b>(exp: &'a Experiment,
                     user_id: &'b str)
                     -> Result<Experience<'a>, &'a str> {
    let exp_hash = hash("choices", exp.namespace, exp.name, "", user_id);

    match valid_segment(&exp.segments, exp_hash) {
        Some(e) => return Err(e),
        _ => (),
    }

    let mut params: Vec<ParamExperience> = Vec::new();
    for param in exp.params.iter() {
        let hash = hash("choices", exp.namespace, exp.name, param.name, user_id);
        params.push(ParamExperience {
            name: param.name,
            choice: match param.choices {
                Choices::Weighted(ref w) => match eval_weighted(w, hash){
                    Ok(s) => s,
                    Err(e) => return Err(e),
                },
                Choices::Uniform(ref u) => eval_uniform(u, hash),
            },
        })
    }
    Ok(Experience {
        name: exp.name,
        namespace: exp.namespace,
        params: params,
    })
}

fn eval_weighted<'a>(choices: &Vec<(&'a str, f64)>, hash: u64) -> Result<&'a str, &'a str> {
    let partitions: Vec<(&str, f64)> = choices.iter()
        .scan(0f64, |accum, &(s, w)| {
            *accum += w;
            Some((s, *accum))
        })
        .collect();
    let x = get_uniform(0.0, partitions[partitions.len() - 1].1, hash);
    match partitions.iter().find(|&&(_, p)| {
        x < p
    }) {
        Some(&(s, _)) => Ok(s),
        None => Err("could not determine choice"),
    }
}

fn eval_uniform<'a>(choices: &Vec<&'a str>, hash: u64) -> &'a str {
    choices[(hash as usize) % choices.len()]
}

fn get_uniform(min: f64, max: f64, hash: u64) -> f64 {
    const LONG_SCALE: f64 = 0xFF_FF_FF_FF_FF_FF_FF_FFu64 as f64;
    let zero_to_one = (hash as f64) / LONG_SCALE;
    min + (max - min) * zero_to_one
}

fn hash(salt: &str,
        namespace: &str,
        experiment_name: &str,
        param_name: &str,
        user_id: &str)
        -> u64 {
    let mut m = sha1::Sha1::new();
    let hash_string = format!("{}:{}:{}:{}:{}", salt, namespace, experiment_name, param_name, user_id);
    m.update(hash_string.as_bytes());
    let a = &m.digest().bytes()[0..16];
    BigEndian::read_u64(a)
}

// valid_segment if a segment is valid None will be returned
fn valid_segment<'a>(segments: &Vec<u8>, hash: u64) -> Option<&str> {
    let pos: u64 = hash % ((segments.len() as u64) * 8);
    let byte: u8 = segments[(pos / 8) as usize];
    match 1 << (pos % 8) & byte {
        0 => {
            return Some("segment not activated")
        }
        _ => None,
    }
}