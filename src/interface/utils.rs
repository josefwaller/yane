use crate::{AppSettings, Nes};
use log::*;

pub fn save_new_savestate(nes: &Nes, settings: &mut AppSettings) {
    match nes.to_savestate() {
        Err(e) => error!("Unable to create quicksave: {}", e),
        Ok(data) => {
            let mut path = settings.savestate_dir.clone();
            let filename = chrono::Local::now().format("savestate-%Y-%m-%d-%H-%M-%S.bin");
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
