use std::{ rc::Rc, cell::RefCell, collections::HashSet };

use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Window;

pub struct Keyboard {
    keys_down: Rc<RefCell<HashSet<Key>>>
}

impl Keyboard {
    pub fn new(window: &Window) -> Self {
        let keys_down = Rc::new(RefCell::new(HashSet::new()));
        Self::add_key_down_listener(window, keys_down.clone());
        Self::add_key_up_listener(window, keys_down.clone());
        Self { keys_down }
    }

    pub fn is_down(&self, key: Key) -> bool {
        self.keys_down.as_ref().borrow_mut().contains(&key)
    }

    fn add_key_down_listener(window: &Window, keys_down: Rc<RefCell<HashSet<Key>>>) {
        // Set up the listener as a JS &Function
        let listener = move |e: web_sys::KeyboardEvent| {
            if let Some(key) = Key::from(&e.key()) {
                keys_down.as_ref().borrow_mut().insert(key); // This key was pressed down; add to the set.
            }
        };

        let listener = Closure::<dyn FnMut(_)>::new(listener);

        // "Register" the listener
        window.add_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref()).unwrap();
        listener.forget();
    }

    fn add_key_up_listener(window: &Window, keys_down: Rc<RefCell<HashSet<Key>>>) {
        // Set up the listener as a JS &Function
        let listener = move |e: web_sys::KeyboardEvent| {
            if let Some(key) = Key::from(&e.key()) {
                keys_down.as_ref().borrow_mut().remove(&key); // This key was released; remove from the set.
            }
        };

        let listener = Closure::<dyn FnMut(_)>::new(listener);

        // "Register" the listener
        window.add_event_listener_with_callback("keyup", listener.as_ref().unchecked_ref()).unwrap();
        listener.forget();
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Turbo,
}

impl Key {
    fn from(str: &str) -> Option<Key> {
        match str {
            "ArrowUp" => Some(Key::Up),
            "ArrowDown" => Some(Key::Down),
            "ArrowLeft" => Some(Key::Left),
            "ArrowRight" => Some(Key::Right),
            "Shift" => Some(Key::Turbo),
            _ => None
        }
    }
}