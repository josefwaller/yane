use crate::{AppSettings, Nes};
use log::*;

pub fn save_new_savestate(nes: &Nes, settings: &mut AppSettings, game_name: &Option<String>) {
    match nes.to_savestate() {
        Err(e) => error!("Unable to create quicksave: {}", e),
        Ok(data) => {
            let mut path = settings.savestate_dir.clone();
            let game = match game_name {
                Some(n) => format!("{}_", n),
                None => String::new(),
            };
            let time = chrono::Local::now().format("%Y_%m_%d__%H_%M_%S");
            let filename = format!("savestate_{}_{}.yane.bin", game, time);
            path.push(filename.to_string());
            match std::fs::write(&path, data) {
                Ok(_) => {
                    debug!("Wrote savestate to {:?}", &path);
                    settings.quickload_file = Some(path);
                }
                Err(e) => error!("Unable to save savestate: {}", e),
            }
        }
    }
}
