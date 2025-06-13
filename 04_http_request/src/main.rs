use reqwest;

// get
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let response = reqwest::get("https://jsonplaceholder.typicode.com/posts").await?;

    println!("{:#?}", response);

    println!("Status: {}", response.status());

    let body = response.text().await?; // Membaca body sebagai string
    println!("Body: {}", body);

    Ok(())
}

// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize)]
// struct MyData {
//     title: String,
//     body: String,
//     #[serde(rename = "userId")]
//     user_id: u32,
// }

// #[tokio::main]
// async fn main() -> Result<(), reqwest::Error> {
//     let new_post = MyData {
//         title: "foo".to_string(),
//         body: "bar".to_string(),
//         user_id: 1,
//     };

//     // Melakukan POST request
//     let client = reqwest::Client::new();
//     let response = client.post("https://jsonplaceholder.typicode.com/posts")
//         .json(&new_post)  // Mengirim data dalam format JSON
//         .send()
//         .await?;

//     // Menampilkan status dan body response
//     println!("Status: {}", response.status());
//     let body = response.text().await?;
//     println!("Body: {}", body);

//     Ok(())
// }
