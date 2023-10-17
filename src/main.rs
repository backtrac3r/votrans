use axum::{extract::State, routing::post, Json, Router};
use hound::WavReader;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;
use vosk::{Model, Recognizer};
use ytd_rs::{Arg, YoutubeDL};

#[derive(Clone)]
struct AppData {
    temp_counter: Arc<Mutex<u32>>,
}

#[derive(serde::Deserialize)]
struct Ytdlp {
    url: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ffmpeg", post(ffmpeg_page))
        .route("/yt", post(yt_dlp))
        .route("/vosk", post(vosk_page))
        .route("/full", post(full_cycle))
        .with_state(AppData {
            temp_counter: Arc::new(Mutex::new(0)),
        });

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ffmpeg_page(path: Json<Ytdlp>) -> String {
    println!("got ffmpeg");
    println!("{:?}", std::env::current_dir().unwrap().to_str().unwrap());

    let input_path = format!("./downloads/{}", path.url);
    let name_without_ext = path.url.split('.').next().unwrap();
    let output_name = format!("./ffmpeg/{name_without_ext}.wav");

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

    format!("{output_name}.wav")
}

async fn yt_dlp(State(data): State<AppData>, url: Json<Ytdlp>) -> String {
    println!("got reqwest {}", url.url);
    let output_name = format!("temp{}", data.temp_counter.lock().await);
    *data.temp_counter.lock().await += 1;

    let args = vec![
        Arg::new("--quiet"),
        Arg::new_with_arg("--output", format!("{output_name}.%(ext)s").as_str()),
    ];
    // let link = "https://www.youtube.com/watch?v=uTO0KnDsVH0";
    let path = PathBuf::from("./downloads");
    let Ok(ytd) = YoutubeDL::new(&path, args, &url.url) else {
        return String::new();
    };

    // start download
    let Ok(download) = ytd.download() else {
        return String::new();
    };

    // print out the download path
    println!(
        "Your download: {},\n filename: {}",
        download.output_dir().to_string_lossy(),
        output_name
    );

    format!("{output_name}.webm")
}

async fn vosk_page(url: Json<Ytdlp>) -> String {
    vosk(url.url.clone())
}

fn vosk(wav_path: String) -> String {
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

async fn full_cycle(State(data): State<AppData>, url: Json<Ytdlp>) -> String {
    // YT_DLP
    println!("got reqwest {}", url.url);
    let output_name = format!("temp{}", data.temp_counter.lock().await);
    *data.temp_counter.lock().await += 1;

    let args = vec![
        Arg::new("--quiet"),
        Arg::new_with_arg("--output", format!("{output_name}.%(ext)s").as_str()),
    ];
    // let link = "https://www.youtube.com/watch?v=uTO0KnDsVH0";
    let path = PathBuf::from("./downloads");
    let Ok(ytd) = YoutubeDL::new(&path, args, &url.url) else {
        return String::new();
    };

    // start download
    let Ok(download) = ytd.download() else {
        return String::new();
    };

    // print out the download path
    println!(
        "Your download: {},\n filename: {}",
        download.output_dir().to_string_lossy(),
        output_name
    );

    // FFMPEG
    println!("got ffmpeg");
    println!("{:?}", std::env::current_dir().unwrap().to_str().unwrap());
    let input_path = format!("./downloads/{output_name}.webm");
    let name_without_ext = output_name.split('.').next().unwrap();
    let output_path_and_name = format!("./ffmpeg/{name_without_ext}.wav");

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
    println!("{output_path_and_name}");

    // VOSK
    vosk(output_path_and_name)
}
