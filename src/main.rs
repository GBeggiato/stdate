use stdate::DateTime;

fn main() {
    let m = DateTime::now();
    match m {
        Some(d) => {println!("{}", d.str_full())},
        None => {println!("not a valid date")},
    }
}
