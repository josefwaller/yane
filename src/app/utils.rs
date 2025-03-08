use crate::{app::Config, core::Nes};
use log::*;

/// Perform a quick save
///
/// Serialize the NES, and then save it to a file containing [Config::game_name] and the Unix timestamp at the save time.
/// Override [Config::quickload_file] to this new file's path if successful.
pub fn quicksave(nes: &Nes, config: &mut Config) {
    match nes.to_savestate() {
        Err(e) => error!("Unable to create quicksave: {}", e),
        Ok(data) => {
            let mut path = config.savestate_dir.clone();
            let game = match &config.game_name {
                Some(n) => format!("{}_", n),
                None => String::new(),
            };
            let time = chrono::Local::now().format("%Y_%m_%d__%H_%M_%S");
            let filename = format!("savestate_{}{}.yane.bin", game, time);
            path.push(&filename);
            match std::fs::write(&path, data) {
                Ok(_) => {
                    debug!("Wrote savestate to {:?}", &path);
                    config.quickload_file = Some(path);
                }
                Err(e) => error!("Unable to save savestate: {}", e),
            }
        }
    }
}
/// Perform a quick load
///
/// Load the savestate at [Config::quickload_file], parse the [Nes] from the bytes, and return the [Nes].
pub fn quickload(config: &mut Config) -> Option<Nes> {
    match &config.quickload_file {
        Some(f) => match std::fs::read(f) {
            Ok(data) => match postcard::from_bytes(&data) {
                Ok(n) => {
                    info!("Loaded quicksave at {:?}", f);
                    return Some(n);
                }
                Err(e) => error!("Unable to deserialize save state {:?}: {}", &f, e),
            },
            Err(e) => error!("Unable to read save state {:?}: {}", &f, e),
        },
        None => info!("No save state to quickload from"),
    }
    None
}
