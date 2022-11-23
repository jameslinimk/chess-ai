use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;
use macroquad::texture::{load_texture, FilterMode, Texture2D};
use maplit::hashmap;

lazy_static! {
    static ref ASSET_MAP: Mutex<HashMap<String, Texture2D>> = Mutex::new(hashmap!());
}

pub fn get_image(path: &str) -> Texture2D {
    match ASSET_MAP.lock().unwrap().get(path) {
        Some(texture) => texture.to_owned(),
        None => panic!("{}", format!("Path \"{}\" not loaded!", path)),
    }
}

pub fn get_image_owned(path: String) -> Texture2D {
    match ASSET_MAP.lock().unwrap().get(&path) {
        Some(texture) => texture.to_owned(),
        None => panic!("{}", format!("Path \"{}\" not loaded!", path)),
    }
}

pub async fn load_image(path: &'static str) -> Texture2D {
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

pub async fn load_image_owned(path: String) -> Texture2D {
    if ASSET_MAP.lock().unwrap().contains_key(&path) {
        return get_image_owned(path);
    }
    let resource = load_texture(&path).await.unwrap();
    resource.set_filter(FilterMode::Nearest);
    ASSET_MAP.lock().unwrap().insert(path, resource.to_owned());
    resource
}
