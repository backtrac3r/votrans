use axum::http::StatusCode;
use hound::WavReader;
use vosk::{Model, Recognizer};

use crate::error::AppErr;

#[derive(serde::Deserialize)]
pub struct Ytdlp {
    pub url: String,
}

#[allow(clippy::cast_precision_loss)]
pub fn vosk_wav(wav_path: String) -> Result<String, AppErr> {
    // let model_path = "/home/zbykovd/Downloads/vosk-model-small-ru-0.22";
    let model_path = "./model/vosk-model-small-ru-0.22";
    // let model_path = "/home/zbykovd/Downloads/vosk-model-en-us-0.22/";

    // let wav_path = "/home/zbykovd/projects/job/votrans/ffmpeg/temp0.wav";
    // let wav_path = "/home/zbykovd/projects/job/vosk-rs/file.wav";

    dbg!();
    let mut reader = WavReader::open(wav_path).map_err(|e| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not create the WAV reader: {e}"),
        )
    })?;
    dbg!();

    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<i16>>>()
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not read WAV file: {e}"),
            )
        })?;

    dbg!();
    let model = Model::new(model_path).ok_or_else(|| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not create the model",
        )
    })?;

    dbg!();
    let mut recognizer =
        Recognizer::new(&model, reader.spec().sample_rate as f32).ok_or_else(|| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not create the recognizer",
            )
        })?;
    dbg!();

    recognizer.set_max_alternatives(0);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    dbg!();
    // for sample in samples.chunks(1000).skip(500) {
    //     recognizer.accept_waveform(sample);
    //     println!("{:#?}", recognizer.partial_result());
    // }

    dbg!();
    recognizer.accept_waveform(&samples);

    dbg!();
    let res = recognizer.final_result().single().ok_or_else(|| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not create the recognizer",
        )
    })?;

    dbg!();
    println!("vosk done");

    Ok(res.text.to_string())
}
