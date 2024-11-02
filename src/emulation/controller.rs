/// Struct that contains the state a controller is in
#[derive(Clone, Copy)]
pub struct Controller {
    pub up: bool,
    pub left: bool,
    pub right: bool,
    pub down: bool,
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            up: false,
            left: false,
            right: false,
            down: false,
            start: false,
            select: false,
            a: false,
            b: false,
        }
    }
}
