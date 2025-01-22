use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Masukkan angka:");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let finish: i32 = input.trim().parse().map_err(|_| "Input tidak valid, masukkan angka!")?;

    for num in 1..=finish {
        match (num % 3, num % 5) {
            (0, 0) => println!("Fizz Buzz"),
            (0, _) => println!("Fizz"),
            (_, 0) => println!("Buzz"),
            _ => println!("{}", num),
        }
    }

    Ok(())
}
