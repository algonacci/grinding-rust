use std::io;

fn main() -> io::Result<()> {
    println!("Masukkan temperatur:\n");

    let mut input_temperatur = String::new();
    io::stdin().read_line(&mut input_temperatur)?;

    println!("Masukkan kelembaban:\n");
    let mut input_kelembaban = String::new();
    io::stdin().read_line(&mut input_kelembaban)?;

    let temperatur: i32 = input_temperatur.trim().parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Input tidak valid, masukkan angka!",
        )
    })?;

    let kelembaban: i32 = input_kelembaban.trim().parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Input tidak valid, masukkan angka!",
        )
    })?;

    if temperatur > 25 && kelembaban > 70 {
        println!("Cuaca panas dan berawan");
    } else if temperatur > 25 && kelembaban <= 70 {
        println!("Cuaca panas");
    } else if temperatur <= 25 && kelembaban > 70 {
        println!("Cuaca berawan");
    } else {
        println!("Cuaca dingin");
    }

    Ok(())
}
