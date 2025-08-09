use stdate::DateTime;

fn main() {
    let n = DateTime::now().expect("now is a valid date!");
    println!("{}", n.str_full());
}
