use crate::error::AppErr;
use axum::http::StatusCode;
use hound::WavReader;
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

pub fn file_ext_from_url(url: &str) -> Result<String, AppErr> {
    let ext = if url.contains("youtu.be") || url.contains("youtube.com") {
        String::from("opus")
    } else if url.contains("vk.com") {
        String::from("m4a")
    } else {
        return Err(AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "err while get file ext from url",
        ));
    };

    Ok(ext)
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
