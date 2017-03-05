extern crate sha1;
extern crate byteorder;

use byteorder::ReadBytesExt;

fn main() {
    println!("Hello, world!");
    let e = Experiment {
        name: "this",
        namespace: "namespace",
        params: vec![Param {
                         name: "p1",
                         choices: vec![Choice {
                                           name: "1",
                                           weight: 0.0,
                                       },
                                       Choice {
                                           name: "2",
                                           weight: 0.0,
                                       }],
                     }],
    };
    println!("{:?}", eval_test(&e, "13").unwrap());
}

struct Experiment<'a> {
    name: &'a str,
    namespace: &'a str,
    params: Vec<Param<'a>>,
}

struct Param<'a> {
    name: &'a str,
    choices: Vec<Choice<'a>>,
}

struct Choice<'a> {
    name: &'a str,
    weight: f64,
}

#[derive(Debug)]
struct Experience<'a> {
    name: &'a str,
    namespace: &'a str,
    params: Vec<ParamExperience<'a>>,
}

#[derive(Debug)]
struct ParamExperience<'a> {
    name: &'a str,
    choice: &'a str,
}

fn eval_test<'a>(exp: &'a Experiment, userid: &str) -> Result<Experience<'a>, std::io::Error> {
    let mut params: Vec<ParamExperience> = Vec::new();
    for elem in exp.params.iter() {
        let mut m = sha1::Sha1::new();
        m.update(exp.namespace.as_bytes());
        m.update(b":");
        m.update(exp.name.as_bytes());
        m.update(b":");
        m.update(elem.name.as_bytes());
        m.update(b":");
        m.update(userid.as_bytes());
        let mut cur = std::io::Cursor::new(m.digest().bytes());
        let hash: u64 = match cur.read_u64::<byteorder::BigEndian>() {
            Ok(i) => i,
            Err(err) => return Err(err),
        };

        params.push(ParamExperience {
            name: elem.name,
            choice: elem.choices[(hash % (elem.choices.len() as u64)) as usize].name,
        })
    }
    Ok(Experience {
        name: exp.name,
        namespace: exp.namespace,
        params: params,
    })
}