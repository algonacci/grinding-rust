use rand::Rng;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Permainan menebak angka dimulai!");
    println!("Masukkan Angka dari 1-100:");

    let secret_number = rand::thread_rng().gen_range(1..101);

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input: i32 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Input tidak valid, masukkan angka!");
                continue;
            }
        };

        if input < secret_number {
            if secret_number - input <= 10 {
                println!("Terlalu kecil, tapi mendekati!");
            } else {
                println!("Terlalu kecil!");
            }
        } else if input > secret_number {
            if input - secret_number <= 10 {
                println!("Terlalu besar, tapi mendekati!");
            } else {
                println!("Terlalu besar!");
            }
        } else {
            println!("Selamat, tebakanmu benar!");
            break;
        }
    }

    Ok(())
}
