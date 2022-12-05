//! Helper functions for storing and loading `Texture2D`s and `Sound`s in sync code

use std::sync::Mutex;

use lazy_static::lazy_static;
use macroquad::audio::{load_sound, Sound};
use macroquad::texture::{load_texture, FilterMode, Texture2D};
use rustc_hash::FxHashMap;

use crate::hashmap;

lazy_static! {
    static ref ASSET_MAP: Mutex<FxHashMap<String, Texture2D>> = Mutex::new(hashmap! {});
    static ref AUDIO_MAP: Mutex<FxHashMap<String, Sound>> = Mutex::new(hashmap! {});
}

pub fn get_image(path: &str) -> Texture2D {
    match ASSET_MAP.lock().unwrap().get(path) {
        Some(texture) => texture.to_owned(),
        None => panic!("{}", format!("Path \"{}\" not loaded!", path)),
    }
}

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

pub fn get_audio(path: &str) -> Sound {
    match AUDIO_MAP.lock().unwrap().get(path) {
        Some(texture) => texture.to_owned(),
        None => panic!("{}", format!("Path \"{}\" not loaded!", path)),
    }
}

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
