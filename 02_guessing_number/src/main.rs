use rand::Rng;
use std::cmp::Ordering;
use std::io;

/// Membaca input dari pengguna dan mengonversinya ke i32.
/// Jika input tidak valid, mengembalikan `None`.
fn read_input() -> Option<i32> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Gagal membaca input");

    input.trim().parse().ok()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Permainan menebak angka dimulai!");
    println!("Masukkan Angka dari 1-100:");

    // Generate angka acak antara 1 dan 100
    let secret_number = rand::thread_rng().gen_range(1..=100);

    loop {
        // Membaca input dari pengguna
        let input = match read_input() {
            Some(num) => num,
            None => {
                println!("Input tidak valid, masukkan angka!");
                continue;
            }
        };

        // Memeriksa tebakan pengguna
        match input.cmp(&secret_number) {
            Ordering::Less => {
                if secret_number - input <= 10 {
                    println!("Terlalu kecil, tapi mendekati!");
                } else {
                    println!("Terlalu kecil!");
                }
            }
            Ordering::Greater => {
                if input - secret_number <= 10 {
                    println!("Terlalu besar, tapi mendekati!");
                } else {
                    println!("Terlalu besar!");
                }
            }
            Ordering::Equal => {
                println!("Selamat, tebakanmu benar!");
                break;
            }
        }
    }

    Ok(())
}
