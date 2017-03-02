extern crate sha1;

fn main() {
    println!("Hello, world!");
    let e = Experiment {
        name: "this",
        namespace: "namespace",
        params: vec![Param {
                         name: "p1",
                         choices: vec!["1", "2"],
                         weights: vec![],
                     }],
    };
    eval_test(&e, "123");
}

struct Experiment<'a> {
    name: &'a str,
    namespace: &'a str,
    params: Vec<Param<'a>>,
}

struct Param<'a> {
    name: &'a str,
    choices: Vec<&'a str>,
    weights: Vec<f64>,
}

fn eval_test(exp: &Experiment, userid: &str) {
    for elem in exp.params.iter() {
        let mut m = sha1::Sha1::new();
        m.update(exp.namespace.as_bytes());
        m.update(exp.name.as_bytes());
        m.update(elem.name.as_bytes());
        m.update(userid.as_bytes());
        println!("{}", m.digest().to_string())
    }
}