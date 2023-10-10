use std::sync::Mutex;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

// #[get("/")]
// async fn hello() -> impl Responder {
//     HttpResponse::Ok().body("Hello world!")
// }

// #[post("/echo")]
// async fn echo(req_body: String) -> impl Responder {
//     HttpResponse::Ok().body(req_body)
// }

// #[get("/json")]
// async fn json() -> impl Responder {
//     HttpResponse::Ok().json(Ytdlp {
//         url: "https://youtu.be/dQw4w9WgXcQ".to_string(),
//     })
// }

// #[get("/hi")]
// async fn hi() -> impl Responder {
//     HttpResponse::Ok().body("hi")
// }

#[post("/ffmpeg")]
async fn ffmpeg_page(path: web::Json<Ytdlp>) -> impl Responder {
    use std::process;

    println!("got ffmpeg");
    println!("{:?}", std::env::current_dir().unwrap().to_str().unwrap());
    let input_path = format!("./downloads/{}", path.url);
    let name_without_ext = path.url.split('.').into_iter().next().unwrap();
    let output_name = format!("./ffmpeg/{}.wav", name_without_ext);

    let mut ffmpeg = process::Command::new("ffmpeg")
        .args(vec![
            "-y",
            "-i",
            &input_path,
            "-map",
            "0:a",
            "-ac",
            "1",
            &output_name,
        ])
        .spawn()
        .unwrap();
    ffmpeg.wait().unwrap();
    println!("done ffmpeg");

    // println!("{}", String::from_utf8(ffmpeg.stdout).unwrap());

    HttpResponse::Ok().body(format!("{}.wav", output_name))
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Ytdlp {
    url: String,
}

#[post("/yt")]
async fn yt_dlp(url: web::Json<Ytdlp>, data: web::Data<AppData>) -> impl Responder {
    use std::path::PathBuf;
    use ytd_rs::{Arg, YoutubeDL};

    println!("got reqwest {}", url.url);
    let output_name = format!("temp{}", data.temp_counter.lock().unwrap().to_string());
    *data.temp_counter.lock().unwrap() += 1;

    let args = vec![
        Arg::new("--quiet"),
        Arg::new_with_arg("--output", format!("{}.%(ext)s", output_name).as_str()),
    ];
    // let link = "https://www.youtube.com/watch?v=uTO0KnDsVH0";
    let path = PathBuf::from("./downloads");
    let ytd = match YoutubeDL::new(&path, args, &url.url) {
        Ok(res) => res,
        Err(_) => return HttpResponse::Conflict().body(()),
    };

    // start download
    let download = match ytd.download() {
        Ok(res) => res,
        Err(_) => return HttpResponse::Conflict().body(()),
    };

    // print out the download path
    println!(
        "Your download: {},\n filename: {}",
        download.output_dir().to_string_lossy(),
        output_name
    );
    HttpResponse::Ok().body(format!("{}.webm", output_name))
}

#[post("/vosk")]
async fn vosk_page(url: web::Json<Ytdlp>) -> impl Responder {
    let res = vosk(url.url.clone());
    HttpResponse::Ok().body(res)
}

fn vosk(wav_path: String) -> String {
    use hound::WavReader;
    use vosk::{Model, Recognizer};

    // let model_path = "/home/zbykovd/Downloads/vosk-model-small-ru-0.22";
    let model_path = "/home/zbykovd/Downloads/vosk-model-small-en-us-0.15/";
    // let model_path = "/home/zbykovd/Downloads/vosk-model-en-us-0.22/";

    // let wav_path = "/home/zbykovd/projects/job/votrans/ffmpeg/temp0.wav";
    // let wav_path = "/home/zbykovd/projects/job/vosk-rs/file.wav";

    let mut reader = WavReader::open(wav_path).expect("Could not create the WAV reader");
    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<i16>>>()
        .expect("Could not read WAV file");

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer = Recognizer::new(&model, reader.spec().sample_rate as f32)
        .expect("Could not create the recognizer");

    recognizer.set_max_alternatives(0);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    // for sample in samples.chunks(1000).skip(500) {
    //     recognizer.accept_waveform(sample);
    //     println!("{:#?}", recognizer.partial_result());
    // }

    recognizer.accept_waveform(&samples);

    println!("{:#?}", recognizer.final_result().single().unwrap().text);
    println!("vosk done");
    recognizer.final_result().single().unwrap().text.to_string()
}

#[post("/full")]
async fn full_cycle(url: web::Json<Ytdlp>, data: web::Data<AppData>) -> impl Responder {
    // YT_DLP
    use std::path::PathBuf;
    use ytd_rs::{Arg, YoutubeDL};

    println!("got reqwest {}", url.url);
    let output_name = format!("temp{}", data.temp_counter.lock().unwrap().to_string());
    *data.temp_counter.lock().unwrap() += 1;

    let args = vec![
        Arg::new("--quiet"),
        Arg::new_with_arg("--output", format!("{}.%(ext)s", output_name).as_str()),
    ];
    // let link = "https://www.youtube.com/watch?v=uTO0KnDsVH0";
    let path = PathBuf::from("./downloads");
    let ytd = match YoutubeDL::new(&path, args, &url.url) {
        Ok(res) => res,
        Err(_) => return HttpResponse::Conflict().body(()),
    };

    // start download
    let download = match ytd.download() {
        Ok(res) => res,
        Err(_) => return HttpResponse::Conflict().body(()),
    };

    // print out the download path
    println!(
        "Your download: {},\n filename: {}",
        download.output_dir().to_string_lossy(),
        output_name
    );

    // FFMPEG
    use std::process;

    println!("got ffmpeg");
    println!("{:?}", std::env::current_dir().unwrap().to_str().unwrap());
    let input_path = format!("./downloads/{}.webm", output_name);
    let name_without_ext = output_name.split('.').into_iter().next().unwrap();
    let output_path_and_name = format!("./ffmpeg/{}.wav", name_without_ext);

    let mut ffmpeg = process::Command::new("ffmpeg")
        .args(vec![
            "-y",
            "-i",
            &input_path,
            "-map",
            "0:a",
            "-ac",
            "1",
            &output_path_and_name,
        ])
        .spawn()
        .unwrap();
    ffmpeg.wait().unwrap();
    println!("done ffmpeg");
    println!("{}", output_path_and_name);

    // VOSK
    let res = vosk(output_path_and_name);

    HttpResponse::Ok().body(res)
}

// async fn manual_hello() -> impl Responder {
//     HttpResponse::Ok().body("Hey there!")
// }

struct AppData {
    temp_counter: Mutex<u32>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppData {
                temp_counter: Mutex::new(0),
            }))
            .service(yt_dlp)
            .service(ffmpeg_page)
            .service(vosk_page)
            .service(full_cycle)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
