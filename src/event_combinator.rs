use crate::stick::Event;

pub struct EventCombinator {
    menu_left_pressed: bool,
    menu_right_pressed: bool,
}

pub enum SpecialEvent {
    Shutdown,
}

impl EventCombinator {
    pub fn new() -> Self {
        Self {
            menu_left_pressed: false,
            menu_right_pressed: false,
        }
    }

    pub fn add(&mut self, event: &Event) -> Option<SpecialEvent> {
        match event {
            Event::MenuL(pressed) => {
                self.menu_left_pressed = *pressed;
            }
            Event::MenuR(pressed) => {
                self.menu_right_pressed = *pressed;
            }
            _ => {}
        }
        if self.menu_left_pressed && self.menu_right_pressed {
            return Some(SpecialEvent::Shutdown);
        }
        None
    }
}
