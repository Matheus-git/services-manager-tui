use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent};

use crate::domain::service::Service;
use crate::terminal::app::{Actions, AppEvent};
use crate::usecases::services_manager::ServicesManager;

pub struct ServiceDetails {
    service: Option<Arc<Mutex<Service>>>,
    unit_file: String,
    sender: Sender<AppEvent>,
    scroll: u16,
    usecase: Rc<RefCell<ServicesManager>>,
}

impl ServiceDetails {
    pub fn new(sender: Sender<AppEvent>,  usecase: Rc<RefCell<ServicesManager>>) -> Self {
        Self {
            service: None,
            sender,
            unit_file: String::new(),
            scroll: 0,
            usecase
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(service_arc) = &self.service {
            let service = service_arc.lock().unwrap();

            let paragraph = Paragraph::new(self.unit_file.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" {} properties ", service.name()))
                        .title_alignment(Alignment::Center),
                )
                .scroll((self.scroll, 0));

            frame.render_widget(paragraph, area);
        }
    }

    pub fn on_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Right => {
                self.reset();
                self.sender.send(AppEvent::Action(Actions::GoLog)).unwrap();
            }
            KeyCode::Left => {
                self.reset();
                self.sender.send(AppEvent::Action(Actions::GoLog)).unwrap();
            }
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.scroll += 1;
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll += 10;
            }

            KeyCode::Char('q') => {
                self.reset();
                self.exit();
            }
            _ => {}
        }
    }

    pub fn shortcuts(&mut self) -> Vec<Line<'_>> {
        let help_text = vec![
            Line::from(vec![Span::styled(
                "Actions",
                Style::default()
                    .fg(Color::LightMagenta)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("Switch tabs: ←/→ | Go back: q"),
        ];

        help_text
    }

    pub fn reset(&mut self) {
        self.service = None;
        self.scroll = 0;
    }

    fn exit(&self) {
        self.sender.send(AppEvent::Action(Actions::GoList)).unwrap();
    }

    pub fn fetch_unit_file(&mut self) {
        if let Some(service_arc) = &self.service {
            let service = service_arc.lock().unwrap();
            match self.usecase.borrow().systemctl_cat(&service) {
                Ok(content) => {
                    self.unit_file = content;
                },
                Err(e) => {
                    self.sender.send(AppEvent::Error(e.to_string())).unwrap();
                }
            }
        }
    }

    pub fn update(&mut self, service: Service) {
        self.service = Some(Arc::new(Mutex::new(service)));
    }
}
