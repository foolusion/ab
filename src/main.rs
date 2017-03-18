extern crate sha1;
extern crate byteorder;
extern crate rand;

use byteorder::{ByteOrder, BigEndian};
use rand::Rng;

fn main() {
    println!("Hello, world!");
    let e = Experiment {
        name: "this".to_string(),
        namespace: "namespace".to_string(),
        params: vec![Param {
                         name: "p1".to_string(),
                         choices: Choices::Uniform(vec!["a".to_string(), "b".to_string()]),
                     }],
        segments: vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                       255],
    };
    let e2 = Experiment {
        name: "that".to_string(),
        namespace: "arstneio".to_string(),
        params: vec![Param {
                         name: "my-param".to_string(),
                         choices: Choices::Weighted(vec![("a".to_string(), 1.0), ("b".to_string(), 2.0)]),
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

fn gen_name(len: i32) -> String {
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

struct Experiment {
    name: String,
    namespace: String,
    params: Vec<Param>,
    segments: Vec<u8>,
}

struct Param {
    name: String,
    choices: Choices,
}

enum Choices {
    Weighted(Vec<(String, f64)>),
    Uniform(Vec<String>),
}

#[derive(Debug)]
struct Experience {
    name: String,
    namespace: String,
    params: Vec<ParamExperience>,
}

#[derive(Debug)]
// ParamExperience is a result from hashing the user and determining their experience.
struct ParamExperience {
    name: String,
    choice: String,
}

fn eval_test<'a, 'b>(exp: &'a Experiment,
                     user_id: &'a String)
                     -> Result<Experience, String> {
    let salt = "choices".to_string();
    let exp_hash = hash(&salt, &exp.namespace, &exp.name, &String::new(), user_id);

    match valid_segment(&exp.segments, exp_hash) {
        Some(e) => return Err(e),
        _ => (),
    }

    let mut params: Vec<ParamExperience> = Vec::new();
    for param in exp.params.iter() {
        let hash = hash(&salt, &exp.namespace, &exp.name, &param.name, user_id);
        params.push(ParamExperience {
            name: param.name.clone(),
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
        name: exp.name.clone(),
        namespace: exp.namespace.clone(),
        params: params,
    })
}

fn eval_weighted<'a>(choices: &Vec<(String, f64)>, hash: u64) -> Result<String, String> {
    let partitions: Vec<(String, f64)> = choices.iter()
        .scan(0f64, |accum, &(ref s, w)| {
            *accum += w;
            Some((s.clone(), *accum))
        })
        .collect();
    let x = get_uniform(0.0, partitions[partitions.len() - 1].1, hash);
    match partitions.iter().find(|&&(_, p)| {
        x < p
    }) {
        Some(&(ref s, _)) => Ok(s.clone()),
        None => Err("could not determine choice".to_string()),
    }
}

fn eval_uniform<'a>(choices: &Vec<String>, hash: u64) -> String {
    choices[(hash as usize) % choices.len()].clone()
}

fn get_uniform(min: f64, max: f64, hash: u64) -> f64 {
    const LONG_SCALE: f64 = 0xFF_FF_FF_FF_FF_FF_FF_FFu64 as f64;
    let zero_to_one = (hash as f64) / LONG_SCALE;
    min + (max - min) * zero_to_one
}

fn hash(salt: &String,
        namespace: &String,
        experiment_name: &String,
        param_name: &String,
        user_id: &String)
        -> u64 {
    let mut m = sha1::Sha1::new();
    let hash_string = format!("{}:{}:{}:{}:{}", salt, namespace, experiment_name, param_name, user_id);
    m.update(hash_string.as_bytes());
    let a = &m.digest().bytes()[0..16];
    BigEndian::read_u64(a)
}

// valid_segment if a segment is valid None will be returned
fn valid_segment<'a>(segments: &Vec<u8>, hash: u64) -> Option<String> {
    let pos: u64 = hash % ((segments.len() as u64) * 8);
    let byte: u8 = segments[(pos / 8) as usize];
    match 1 << (pos % 8) & byte {
        0 => {
            return Some("segment not activated".to_string())
        }
        _ => None,
    }
}