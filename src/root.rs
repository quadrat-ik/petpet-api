use {
    actix_web::{
        http::header::ContentType,
        middleware::Logger,
        web,
        App,
        Error,
        HttpResponse,
        HttpServer
    },
    base64::prelude::*,
    colored::Colorize,
    env_logger::Builder,
    image::{DynamicImage, RgbaImage},
    reqwest::get,
    serde::Serialize,
    std::{
        collections::HashMap,
        io::{Cursor, Write as _},
        sync::Arc,
        time::{Duration, Instant}
    },
    tokio::sync::RwLock
};

#[derive(Serialize)]
struct ImageJson
{
    id: String,
    image_data: String
}

type Cache = Arc<RwLock<HashMap<String, (Instant, String)>>>;

async fn load_img(
    image_bytes: &[u8],
    format: image::ImageFormat
) -> image::ImageResult<DynamicImage>
{
    let cursor = Cursor::new(image_bytes);
    image::load(cursor, format)
}

async fn fetch_image(id: &str) -> Result<RgbaImage, Box<dyn std::error::Error>>
{
    let response = get(&format!("https://avatar.cdev.shop/{id}"))
        .await?
        .bytes()
        .await?;

    Ok(
        DynamicImage::ImageRgb8(load_img(&response, image::ImageFormat::Png).await?.into())
            .to_rgba8()
    )
}

async fn convert_image_to_gif(
    img: RgbaImage,
    filter: petpet::FilterType
) -> Result<Vec<u8>, Box<dyn std::error::Error>>
{
    let gif_data = petpet::generate(img, filter, None)?;
    let mut output = Vec::new();
    petpet::encode_gif(gif_data, &mut output, 30)?;
    Ok(output)
}

fn filter_type_to_string(filter: petpet::FilterType) -> &'static str
{
    match filter {
        petpet::FilterType::CatmullRom => "Cubic: Catmull-Rom",
        petpet::FilterType::Nearest => "Nearest Neighbor",
        _ => "other"
    }
}

async fn handle_image_request(
    id: web::Path<String>,
    cache: web::Data<Cache>,
    query: web::Query<HashMap<String, String>>
) -> Result<HttpResponse, Error>
{
    let replace_id = id.into_inner().replace(".gif", "");

    let parsed_id = match replace_id.parse::<i64>() {
        Ok(_) => replace_id,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid ID format"))
    };

    let mode = query.get("mode").map(|s| s.as_str()).unwrap_or("gif");
    let force_update = query.get("upd").map_or(false, |s| s == "true");
    let speed = query.get("speed").map(|s| s.as_str()).unwrap_or("no");

    let filter = match speed {
        "no" | "false" | "nein" | "not" => petpet::FilterType::CatmullRom,
        _ => petpet::FilterType::Nearest
    };

    let filter_str = filter_type_to_string(filter);

    let cache_key = format!("{}:{}", parsed_id, filter_str);

    let cache_read = cache.read().await;

    if !force_update {
        if let Some((timestamp, cached_data)) = cache_read.get(&cache_key) {
            if timestamp.elapsed() < Duration::from_secs(3600) {
                match mode {
                    "json" => {
                        let json_response = ImageJson {
                            id: parsed_id.to_string(),
                            image_data: cached_data.clone()
                        };
                        return Ok(HttpResponse::Ok().json(json_response));
                    },
                    "base64" => {
                        return Ok(HttpResponse::Ok().body(cached_data.clone()));
                    },
                    _ => {
                        let gif_data = BASE64_STANDARD.decode(cached_data).unwrap();
                        return Ok(HttpResponse::Ok().body(gif_data));
                    }
                }
            }
        }
    }

    drop(cache_read);

    let img = match fetch_image(&parsed_id).await {
        Ok(img) => img,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e)))
    };

    let gif_data = match convert_image_to_gif(img, filter).await {
        Ok(data) => data,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e)))
    };

    let base64_data = BASE64_STANDARD.encode(&gif_data);
    let mut cache_write = cache.write().await;
    cache_write.insert(cache_key, (Instant::now(), base64_data.clone()));

    match mode {
        "json" => {
            let json_response = ImageJson {
                id: parsed_id.to_string(),
                image_data: base64_data
            };
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .insert_header(("Cache-Control", "max-age=3600"))
                .json(json_response))
        },
        "base64" => {
            Ok(HttpResponse::Ok()
                .insert_header(actix_web::http::header::CacheControl(vec![
                    actix_web::http::header::CacheDirective::MaxAge(3600),
                ]))
                .insert_header(ContentType::plaintext())
                .body(format!("data:image/png;base64,{}", base64_data)))
        },
        _ => {
            Ok(HttpResponse::Ok()
                .insert_header(actix_web::http::header::CacheControl(vec![
                    actix_web::http::header::CacheDirective::MaxAge(3600),
                ]))
                .append_header(("Content-Type", "image/gif"))
                .body(gif_data))
        },
    }
}

pub fn logging()
{
    let mut binding = Builder::from_default_env();
    let builder = binding.format(|buf, record| {
        let ts = buf.timestamp();

        let level_color = match record.level() {
            log::Level::Error => "ERROR".red().bold(),
            log::Level::Warn => "WARN".yellow().bold(),
            log::Level::Info => "INFO".blue().bold(),
            log::Level::Debug => "DEBUG".green().bold(),
            log::Level::Trace => "TRACE".purple().bold()
        };

        writeln!(
            buf,
            "{} [{}] - {}",
            ts,
            level_color,
            record.args().to_string().bold().truecolor(159, 146, 104)
        )
    });

    #[cfg(not(debug_assertions))]
    {
        builder.filter_level(log::LevelFilter::Info).init();
    }

    #[cfg(debug_assertions)]
    {
        if std::env::var("DEV_LOGGING").unwrap_or_else(|_| String::from("DEV_INFO")) == "DEV_DEBUG"
        {
            builder.filter_level(log::LevelFilter::Debug).init();
        }
        else {
            builder.filter_level(log::LevelFilter::Info).init();
        }
    }
}

pub fn get_binds() -> (String, u16)
{
    let ip: String = std::env::var("BIND_IP").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("BIND_PORT")
        .unwrap_or_else(|_| "6969".to_string())
        .parse::<u16>()
        .unwrap_or(6969);

    (ip, port)
}

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let _ = dotenv::dotenv().ok();
    logging();
    let (ip, port) = get_binds();
    // -----------------------
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(
                Arc::new(RwLock::new(HashMap::<String, (Instant, String)>::new())).clone()
            ))
            .wrap(Logger::new(
                "%a %r status=%s size_in_bytes=%b serve_time=%Ts"
            ))
            .route("/{id}", web::get().to(handle_image_request))
            .route("/{id}.gif", web::get().to(handle_image_request))
            .route(
                "/",
                web::to(|| async { HttpResponse::Ok().body("Hi! Use: /:id") })
            )
    })
    .bind((ip, port))?
    .run()
    .await
}
