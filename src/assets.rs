//! Helper functions for storing and loading `Texture2D`s and `Sound`s in sync code

use std::sync::Mutex;

use lazy_static::lazy_static;
use macroquad::audio::{load_sound, load_sound_from_bytes, Sound};
use macroquad::prelude::ImageFormat;
use macroquad::texture::{load_texture, FilterMode, Texture2D};
use rustc_hash::FxHashMap;

use crate::hashmap;

lazy_static! {
    static ref ASSET_MAP: Mutex<FxHashMap<String, Texture2D>> = Mutex::new(hashmap! {});
    static ref AUDIO_MAP: Mutex<FxHashMap<String, Sound>> = Mutex::new(hashmap! {});
}

/// Get an previously loaded image from the asset map
pub fn get_image(path: &str) -> Texture2D {
    match ASSET_MAP.lock().unwrap().get(path) {
        Some(texture) => texture.to_owned(),
        None => panic!("{}", format!("Path \"{}\" not loaded!", path)),
    }
}

/// Load an image path into the asset map
pub async fn load_image(path: &str) -> Texture2D {
    if ASSET_MAP.lock().unwrap().contains_key(path) {
        return get_image(path);
    }
    let resource = load_texture(path).await.unwrap();
    resource.set_filter(FilterMode::Nearest);
    ASSET_MAP
        .lock()
        .unwrap()
        .insert(path.to_owned(), resource.to_owned());
    resource
}

/// Load image bytes into the asset map
pub async fn load_image_from_bytes(path: &str, bytes: &[u8]) -> Texture2D {
    if ASSET_MAP.lock().unwrap().contains_key(path) {
        return get_image(path);
    }
    let resource = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    resource.set_filter(FilterMode::Nearest);
    ASSET_MAP
        .lock()
        .unwrap()
        .insert(path.to_owned(), resource.to_owned());
    resource
}

/// Get an previously loaded audio file from the audio map
pub fn get_audio(path: &str) -> Sound {
    match AUDIO_MAP.lock().unwrap().get(path) {
        Some(texture) => texture.to_owned(),
        None => panic!("{}", format!("Path \"{}\" not loaded!", path)),
    }
}

/// Load an audio file into the audio map
pub async fn load_audio(path: &str) -> Sound {
    if AUDIO_MAP.lock().unwrap().contains_key(path) {
        return get_audio(path);
    }
    let resource = load_sound(path).await.unwrap();
    AUDIO_MAP
        .lock()
        .unwrap()
        .insert(path.to_owned(), resource.to_owned());
    resource
}

/// Load audio bytes into the audio map
pub async fn load_audio_from_bytes(path: &str, bytes: &[u8]) -> Sound {
    if AUDIO_MAP.lock().unwrap().contains_key(path) {
        return get_audio(path);
    }
    let resource = load_sound_from_bytes(bytes).await.unwrap();
    AUDIO_MAP
        .lock()
        .unwrap()
        .insert(path.to_owned(), resource.to_owned());
    resource
}
