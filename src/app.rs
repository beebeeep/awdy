use anyhow::Result;
use std::time::Duration;
use tui_widget_list::ListState;

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph},
};

use crate::{
    lane::{LaneState, LaneWidget},
    model::{LaneList, Message, Model, RunningState, Task, TaskState},
};

pub struct App {
    model: Model,
}

impl App {
    pub fn load() -> Result<Self> {
        let tasks = vec![
            Task {
                state: TaskState::Todo,
                title: "Do stuff".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                state: TaskState::Todo,
                title: "Do more stuff".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                state: TaskState::Todo,
                title: "Don't do shit".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                state: TaskState::InProgress,
                title: "Writing shit".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                state: TaskState::Done,
                title: "Give a shit".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
        ];
        let todo_tasks = tasks.iter().filter(|t| t.state == TaskState::Todo);
        let in_progress_tasks = tasks.iter().filter(|t| t.state == TaskState::InProgress);
        let blocked_tasks = tasks.iter().filter(|t| t.state == TaskState::Blocked);
        let done_tasks = tasks.iter().filter(|t| t.state == TaskState::Done);
        let lanes = vec![
            LaneList {
                task_state: TaskState::Todo,
                state: LaneState::from_tasks(todo_tasks),
            },
            LaneList {
                task_state: TaskState::InProgress,
                state: LaneState::from_tasks(in_progress_tasks),
            },
            LaneList {
                task_state: TaskState::Blocked,
                state: LaneState::from_tasks(blocked_tasks),
            },
            LaneList {
                task_state: TaskState::Done,
                state: LaneState::from_tasks(done_tasks),
            },
        ];

        Ok(Self {
            model: Model {
                running_state: RunningState::Running,
                tasks,
                lanes,
            },
        })
    }

    pub fn run(mut self) -> Result<()> {
        let mut terminal = ratatui::try_init()?;

        while self.model.running_state != RunningState::Done {
            // Render the current view
            terminal.draw(|f| view(&mut self.model, f))?;

            // Handle events and map to a Message
            let mut current_msg = handle_event(&self.model)?;

            // Process updates as long as they return a non-None message
            while current_msg.is_some() {
                current_msg = update(&mut self.model, current_msg.unwrap());
            }
        }
        ratatui::restore();
        Ok(())
    }
}
fn view(model: &mut Model, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(frame.area());

    let header_style = Style::default().add_modifier(Modifier::BOLD);

    for (lane, area) in model.lanes.iter_mut().zip(layout.into_iter()) {
        let lane_widget = LaneWidget {
            title: lane.task_state.into(),
        };
        frame.render_stateful_widget(&lane_widget, *area, &mut lane.state);
    }
}

/// Convert Event to Message
///
/// We don't need to pass in a `model` to this function in this example
/// but you might need it as your project evolves
fn handle_event(_: &Model) -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') => Some(Message::Increment),
        KeyCode::Char('k') => Some(Message::Decrement),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Increment => {
            todo!();
        }
        Message::Decrement => {
            todo!();
        }
        Message::Reset => todo!(),
        Message::Quit => {
            // You can handle cleanup and exit here
            model.running_state = RunningState::Done;
        }
    };
    None
}
