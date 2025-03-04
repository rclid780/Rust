use crate::app::state;
use crate::app::tasks::Task;

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{
        Alignment,
        Constraint::{self, Length, Min},
        Layout, Rect,
    },
    prelude::Stylize,
    style::{Color, Style},
    symbols::{self, border},
    text::Line,
    widgets::{Block, Gauge, Padding, Paragraph, Tabs, Widget},
    DefaultTerminal, Frame,
};
use std::{
    io, sync::{
        atomic::{AtomicBool, Ordering}, mpsc::{channel, Receiver, Sender}, Arc
    }, thread, time::Duration, vec
};
use strum::IntoEnumIterator;

pub struct Host {
    state: state::HostState,
    tab: state::SelectedTab,
    background_progress: f64,
    cancelation: Arc<AtomicBool>,
    tx: Sender<state::Event>,
    rx: Receiver<state::Event>,
}

impl Host {
    pub fn new() -> Self{
        let (tx, rx) = channel::<state::Event>();
        Host{
            state: state::HostState::Running,
            tab: state::SelectedTab::Tab1,
            background_progress: 0_f64,
            cancelation: Arc::new(AtomicBool::new(false)),
            tx,
            rx,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let input_tx = self.tx.clone();
        thread::spawn(move || {
            Host::handle_key_input(input_tx);
        });

        while self.state != state::HostState::Completed {
            match self.rx.recv().unwrap() {
                state::Event::KeyInput(key_event) => match self.state {
                    state::HostState::Completed => {}
                    state::HostState::Running => self.handle_key_event(key_event)?,
                    state::HostState::ShuttingDown => self.handle_should_exit(key_event)?,
                },
                state::Event::BackgroundTask(progress) => self.background_progress = progress,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_should_exit(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        match key_event.kind {
            KeyEventKind::Press => match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    while Arc::weak_count(&self.cancelation) > 0 {
                        if self.cancelation.load(Ordering::Relaxed) == false {
                            self.cancelation.store(true, Ordering::Relaxed);
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    self.state = state::HostState::Completed
                }

                KeyCode::Char('n') | KeyCode::Char('N') => self.state = state::HostState::Running,
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        match key_event.kind {
            KeyEventKind::Press => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.state = state::HostState::ShuttingDown
                }

                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.cancelation.store(true, Ordering::Relaxed);
                }

                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if self.cancelation.load(Ordering::Relaxed) == true {
                        self.cancelation.store(false, Ordering::Relaxed);
                    }

                    let (background_tx, cancellation_token) =
                        (self.tx.clone(), Arc::downgrade(&self.cancelation));

                    thread::spawn(move || {
                        Host::background_task(background_tx, cancellation_token);
                    });
                }

                KeyCode::Right => {
                    let cur = self.tab as usize;
                    let next = cur.saturating_add(1);
                    self.tab = state::SelectedTab::from_repr(next)
                        .unwrap_or(state::SelectedTab::from_repr(cur).unwrap());
                }

                KeyCode::Left => {
                    let cur = self.tab as usize;
                    let prev = cur.saturating_sub(1);
                    self.tab = state::SelectedTab::from_repr(prev)
                        .unwrap_or(state::SelectedTab::from_repr(cur).unwrap());
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_key_input(tx: Sender<state::Event>) {
        loop {
            match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(key_event) => {
                    tx.send(state::Event::KeyInput(key_event)).unwrap()
                }
                _ => {}
            }
        }
    }
}

impl Widget for &Host {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (menu_area, body_area, footer_area) = get_layout_areas(area);
        render_menu(menu_area, buf);
        render_body(body_area, buf, self.tab);
        render_footer(footer_area, buf, self.background_progress);

        if self.state == state::HostState::ShuttingDown {
            render_confirm_message(body_area, buf, "Exit?", "Are you sure you want to exit?");
        }
    }
}

fn get_layout_areas(area: ratatui::prelude::Rect) -> (Rect, Rect, Rect) {
    let top_layout = Layout::vertical(Constraint::from_percentages([90, 10]));
    let [app_area, footer_area] = top_layout.areas(area);
    let layout = Layout::horizontal(Constraint::from_percentages([20, 80]));
    let [menu_area, body_area] = layout.areas(app_area);

    (menu_area, body_area, footer_area)
}

fn render_menu(area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    let menu_block = Block::bordered()
        .title(" menu ")
        .title_alignment(Alignment::Center)
        .border_set(border::THICK);

    menu_block.render(area, buf);
}

fn render_body(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    tab: state::SelectedTab,
) {
    let body_block = Block::bordered()
        .title(" TUI Web Client ")
        .title_alignment(Alignment::Center)
        .border_set(border::THICK);

    let tab_area = body_block.inner(area);
    render_tabs(tab_area, buf, tab);

    body_block.render(area, buf);
}

fn render_footer(area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, progress: f64) {
    let instructions = Line::from(vec![
        " Quit:".into(),
        "<q/Q> ".green().bold(),
        " Change Tab:".into(),
        " ◄ ► ".green().bold(),
        " Run:".into(),
        " <r/R> ".green().bold(),
        " Cancel(All):".into(),
        " <c/C> ".green().bold(),
    ])
    .centered();

    let footer_block = Block::bordered()
        .title(" Background Processes ")
        .title_bottom(instructions)
        .border_set(border::THICK);

    let progress_bar = Gauge::default()
        .gauge_style(Style::default().fg(Color::Green))
        .block(footer_block)
        .label(format!("Back ground worker: {:.2}%", progress * 100_f64))
        .ratio(progress);

    progress_bar.render(
        Rect {
            x: area.left(),
            y: area.top(),
            width: area.width,
            height: 3,
        },
        buf,
    );
}

fn render_tabs(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    tab: state::SelectedTab,
) {
    //split up body area for tabs
    let vertical = Layout::vertical([Length(1), Min(0)]);
    let [header_area, inner_area] = vertical.areas(area);
    let horizontal = Layout::horizontal([Min(0), Length(20)]);
    let [tabs_area, title_area] = horizontal.areas(header_area);

    "Example Tabbed Data".bold().render(title_area, buf);

    let titles = state::SelectedTab::iter().map(|tab| {
        format!("  {:#}  ", tab)
            .fg(Color::Gray)
            .bg(Color::default())
    });
    let highlight_style = (Color::default(), Color::LightBlue);
    let selected_tab_index = tab as usize;

    Tabs::new(titles)
        .highlight_style(highlight_style)
        .select(selected_tab_index)
        .padding("", "")
        .divider(" ")
        .render(tabs_area, buf);

    let tab_block = Block::bordered()
        .border_set(symbols::border::PROPORTIONAL_TALL)
        .padding(Padding::horizontal(1))
        .border_style(Color::LightBlue);

    match tab {
        state::SelectedTab::Tab1 => {
            Paragraph::new("Hello World")
                .block(tab_block)
                .render(inner_area, buf);
        }
        state::SelectedTab::Tab2 => {
            Paragraph::new("Welcome to the Ratatui tabs example!")
                .block(tab_block)
                .render(inner_area, buf);
        }
        state::SelectedTab::Tab3 => {
            Paragraph::new("Look! I'm different than others!")
                .block(tab_block)
                .render(inner_area, buf);
        }
        state::SelectedTab::Tab4 => {
            Paragraph::new(
                "I know, these are some basic changes. But I think you got the main idea.",
            )
            .block(tab_block)
            .render(inner_area, buf);
        }
    }
}

fn render_confirm_message(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    title: &str,
    message: &str,
) {
    let popup_block = Block::bordered()
        .title(title)
        .title_bottom(Line::from(" <y>/<n> ").centered())
        .border_set(border::DOUBLE)
        .style(Style::default().bg(Color::Blue));

    let width = (message.len() + 4) as u16;
    let height = 3;
    let x = if (area.width / 2) - (width / 2) + area.x > 0 {
        (area.width / 2) - (width / 2) + area.x
    } else {
        area.x
    };
    let y = if (area.height / 2) - (height / 2) + area.y > 0 {
        (area.height / 2) - (height / 2) + area.y
    } else {
        area.y
    };

    Paragraph::new(message).block(popup_block).render(
        Rect {
            x,
            y,
            width,
            height,
        },
        buf,
    );
}
