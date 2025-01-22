use std::io;

fn main() {
    println!("Masukkan angka: ");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Gagal membaca input");

    let finish: i32 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Gagal membaca input");
            return;
        }
    };

    for num in 1..=finish {
        if num % 3 == 0 && num % 5 == 0 {
            println!("Fizz Buzz");
        } else if num % 3 == 0 {
            println!("Fizz");
        } else if num % 5 == 0 {
            println!("Buzz");
        } else {
            println!("{}", num);
        }
    }
}