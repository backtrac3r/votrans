use crate::error::AppErr;
use axum::http::StatusCode;
use hound::WavReader;
use std::{fs::read_dir, result::Result};
use tokio::process::Command;
use vosk::{Model, Recognizer};

#[allow(clippy::cast_precision_loss)]
pub fn vosk_wav(wav_path: String, model_path: &str) -> Result<String, AppErr> {
    let mut reader = WavReader::open(wav_path).map_err(|e| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not create the WAV reader: {e}"),
        )
    })?;

    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<i16>>>()
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not read WAV file: {e}"),
            )
        })?;

    let model = Model::new(model_path).ok_or_else(|| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not create the model",
        )
    })?;

    let mut recognizer =
        Recognizer::new(&model, reader.spec().sample_rate as f32).ok_or_else(|| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not create the recognizer",
            )
        })?;

    recognizer.set_max_alternatives(0);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    // for sample in samples.chunks(1000).skip(500) {
    //     recognizer.accept_waveform(sample);
    //     println!("{:#?}", recognizer.partial_result());
    // }

    recognizer.accept_waveform(&samples);

    let res = recognizer.final_result().single().ok_or_else(|| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not create the recognizer",
        )
    })?;

    Ok(res.text.to_string())
}

pub fn ext_by_name(path: &str, file_name: &str) -> Result<String, AppErr> {
    let dir = read_dir(path).map_err(|e| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("error while read dir: {e}"),
        )
    })?;

    let dir = dir.filter_map(Result::ok);

    for file in dir {
        let p = file.path().to_string_lossy().into_owned();
        println!("{p}");
        if p.contains(file_name) {
            return Ok(file
                .path()
                .extension()
                .ok_or_else(|| {
                    AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, "error while read ext")
                })?
                .to_string_lossy()
                .into_owned());
        }
    }

    Err(AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, "no file"))
}

pub async fn convert_to_wav(
    ffmpeg_input_file_path: &str,
    ffmpeg_output_file_path: &str,
) -> Result<(), AppErr> {
    Command::new("ffmpeg")
        .args(vec![
            "-y",
            "-i",
            &ffmpeg_input_file_path,
            "-ac",
            "1",
            &ffmpeg_output_file_path,
        ])
        .status()
        .await
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("err while spawn ffmpeg task: {e}"),
            )
        })?;

    Command::new("rm")
        .arg(ffmpeg_input_file_path)
        .status()
        .await
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("err while spawn ffmpeg task: {e}"),
            )
        })?;

    Ok(())
}
