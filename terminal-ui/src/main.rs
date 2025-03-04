use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Weak,
    },
    thread,
    time::Duration,
};

mod app;
use app::render::Host;
use app::tasks::Task;

fn main() -> io::Result<()> {
    let mut host = Host::new();
    let mut terminal = ratatui::init();
    let app_result = host.run(&mut terminal);
    ratatui::restore();

    app_result
}

impl Task for Host {
    fn background_task(tx: Sender<app::state::Event>, cancel: Weak<AtomicBool>) {
        let mut progress = 0_f64;
        let increment = 0.01_f64;

        while !cancel.upgrade().unwrap().load(Ordering::Relaxed) && progress < 1_f64 {
            thread::sleep(Duration::from_millis(500));
            progress += increment;
            progress = progress.min(1_f64);
            tx.send(app::state::Event::BackgroundTask(progress))
                .unwrap();
        }
    }
}
