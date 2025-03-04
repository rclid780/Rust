use std::sync::{atomic::AtomicBool, mpsc::Sender, Weak};

use crate::app::state::Event;

pub trait Task {
    fn background_task(tx: Sender<Event>, cancelation_token: Weak<AtomicBool>);
}