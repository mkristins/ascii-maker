use std::io::Cursor;
use axum::{response::Redirect, routing::{Router, get, post}};
use axum::response::{Html};
use axum::extract::{
    Multipart
};
use image::{DynamicImage, GenericImageView, ImageReader, save_buffer};
use log::{info, error};

async fn index() -> Html<String>{
    info!("Serve index page");
    Html(include_str!("../hello.html").to_string())
}

fn image_work(img : DynamicImage, target_h : u32, target_w : u32, alphabet : String) -> Vec<char> {
    info!("Target image sizes: {} {} {}", target_h, target_w, alphabet);
    let resized_img = img.resize(target_h, target_w, image::imageops::FilterType::Lanczos3);
    let (width, height) = resized_img.dimensions();

    let bytes = resized_img.into_luma8();
    let raw_bytes = bytes.into_raw();
    let _ = save_buffer("web_path.png", &raw_bytes, width, height, image::ExtendedColorType::L8);
    info!("Size {}", raw_bytes.len());
    let block_size = 255 / alphabet.len() + 1;
    let mut buffer = vec!['?'; raw_bytes.len()];
    for (i, pixel) in raw_bytes.iter().enumerate() {
        let index = pixel / (block_size as u8);
        buffer[i] = match alphabet.chars().nth(index as usize) {
            Some(x) => x,
            None => '_'
        };
        print!("{}", buffer[i]);
        if (i + 1) % (width as usize) == 0 {
            print!("\n");
        }
    }
    return buffer;
}

async fn upload(mut multipart: Multipart) -> Html<String>{
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
                                return Html("<h1> Error! </h1>".to_string());
                            }
                        }
                    ,
                    Err(e) => {
                        error!("Error: {}", e);
                        return Html("<h1> Error! </h1>".to_string());
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
        Html(format!("<div><h1> Success! </h1> <pre>{:} </pre></div>", output))
    }
    else{
        Html("<h1> Missing fields! </h1>".to_string())
    }
    
}

#[tokio::main]
async fn main(){
    env_logger::init();

    let app = Router::new()
    .route("/", get(index))
    .route("/upload", post(upload));
    let address = "127.0.0.1:8080";

    info!("Starting the server at http://{}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
}