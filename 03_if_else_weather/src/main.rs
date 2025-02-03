use std::io::{self, Write};

// Simplified error type
#[derive(Debug)]
enum WeatherError {
    IoError,
    ParseError,
}

impl From<io::Error> for WeatherError {
    fn from(_: io::Error) -> Self {
        WeatherError::IoError
    }
}

// Struct untuk menyimpan data cuaca
struct WeatherData {
    temperature: i32,
    humidity: i32,
}

impl WeatherData {
    fn new(temperature: i32, humidity: i32) -> Self {
        Self {
            temperature,
            humidity,
        }
    }

    fn get_weather_condition(&self) -> &str {
        match (self.temperature > 25, self.humidity > 70) {
            (true, true) => "Cuaca panas dan berawan",
            (true, false) => "Cuaca panas",
            (false, true) => "Cuaca berawan",
            (false, false) => "Cuaca dingin",
        }
    }
}

fn read_input(prompt: &str) -> Result<i32, WeatherError> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    input
        .trim()
        .parse()
        .map_err(|_| WeatherError::ParseError)
}

fn main() -> Result<(), WeatherError> {
    let temperature = read_input("Masukkan temperatur: ")?;
    let humidity = read_input("Masukkan kelembaban: ")?;

    let weather = WeatherData::new(temperature, humidity);
    println!("{}", weather.get_weather_condition());

    Ok(())
}