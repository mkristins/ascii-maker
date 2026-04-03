use std::io::Cursor;
use axum::{routing::{Router, get, post}};
use axum::response::{Html, Json};
use axum::extract::{
    Multipart
};
use image::{DynamicImage, ImageReader};
use log::{info, error};
use serde::Serialize;

#[derive(Serialize)]
struct Response{
    str: String
}

async fn index() -> Html<String>{
    info!("Serve index page");
    Html(include_str!("../index.html").to_string())
}

fn image_work(img : DynamicImage, target_h : u32, target_w : u32, alphabet : String) -> Vec<char> {
    info!("Target image sizes: {} {} {}", target_h, target_w, alphabet);
    let resized_img = img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3);

    let bytes = resized_img.into_luma8();
    let raw_bytes = bytes.into_raw();
    info!("Size {}", raw_bytes.len());
    let block_size = 255 / alphabet.len() + 1;
    let mut buffer = vec!['?'; raw_bytes.len()];
    for (i, pixel) in raw_bytes.iter().enumerate() {
        let index = pixel / (block_size as u8);
        buffer[i] = alphabet.chars().nth(index as usize).unwrap_or('_');
    }
    buffer
}

async fn process_request(mut multipart: Multipart) -> Json<Response>{
    let mut target_h: Option<u32> = None;
    let mut target_w: Option<u32> = None;
    let mut image: Option<DynamicImage> = None;
    let mut alphabet: Option<String> = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        match name.as_str() {
            "targetH" => {
                let str = String::from_utf8_lossy(&data);
                target_h = Some(str.parse().unwrap());
                info!("Parsed: {}", str);
            },
            "targetW" => {
                let str = String::from_utf8_lossy(&data);
                target_w = Some(str.parse().unwrap());
                info!("Parsed: {}", str);
            },
            "alphabet" => {
                let str = String::from_utf8_lossy(&data);
                alphabet = Some(str.to_string());
                info!("Parsed: {}", str);
            },
            "image" => {
                println!("{} !", data.len());
                let img = match ImageReader::new(Cursor::new(data)).with_guessed_format() {
                    Ok(reader) =>
                        match reader.decode() {
                            Ok(decoded) => decoded,
                            Err(e) => {
                                error!("Error: {}", e);
                                return Json(Response { str: "Meower".to_string() });
                            }
                        }
                    ,
                    Err(e) => {
                        error!("Error: {}", e);
                        return Json(Response { str: "Error".to_string() })
                        // return Html("<h1> Error! </h1>".to_string());
                    }
                };
                image = Some(img);
            }
            _ => {
                println!("Invalid!");
            }
        }
    }
    if let(Some(h), Some(w), Some(alph), Some(img)) = (target_h, target_w, alphabet, image){
        let res = image_work(img, h, w, alph);
        let mut output = String::new();
        for (i, x) in res.iter().enumerate() {
            output.push(*x);
            if (i + 1) % (w as usize) == 0 {
                output.push_str("<br>");
            }
        }
        Json(Response { str: output })
    }
    else{
        Json(Response { str: "Missing fields".to_string() })
    }
}

#[tokio::main]
async fn main(){
    env_logger::init();

    let app = Router::new()
    .route("/", get(index))
    .route("/api/process", post(process_request));
    let address = "127.0.0.1:8080";

    info!("Starting the server at http://{}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
}