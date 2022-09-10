use rand::prelude::*;

#[derive(Clone, Debug)]
struct Foo {
    x: i32,
}
fn main() {
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("stdin failed :(");
    let num: i32 = line.parse().expect("Failed to parse");
    println!("{num}");
}
