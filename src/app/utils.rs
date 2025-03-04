use crate::{app::Config, core::Nes};
use log::*;

/// Save a new savestate.
///
/// Serialize the NES, and then save it to a file containing `game_name` and the Unix timestamp at the save time.
pub fn save_new_savestate(nes: &Nes, config: &mut Config, game_name: &Option<String>) {
    match nes.to_savestate() {
        Err(e) => error!("Unable to create quicksave: {}", e),
        Ok(data) => {
            let mut path = config.savestate_dir.clone();
            let game = match game_name {
                Some(n) => format!("{}_", n),
                None => String::new(),
            };
            let time = chrono::Local::now().format("%Y_%m_%d__%H_%M_%S");
            let filename = format!("savestate_{}_{}.yane.bin", game, time);
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
