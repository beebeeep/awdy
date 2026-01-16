use anyhow::Result;
use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};
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
        let all_tasks = vec![
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
        let mut tasks = HashMap::new();
        let mut lanes = Vec::with_capacity(4);
        for state in [
            TaskState::Todo,
            TaskState::InProgress,
            TaskState::Blocked,
            TaskState::Done,
        ] {
            let state_tasks = Rc::new(RefCell::new(
                all_tasks
                    .iter()
                    .filter(|t| t.state == TaskState::Todo)
                    .map(Clone::clone)
                    .collect(),
            ));
            let lane = LaneList {
                for_state: state,
                state: LaneState::new(false, state_tasks.clone()),
            };
            lanes.push(lane);
            tasks.insert(state, state_tasks);
        }
        lanes[0].state.active = true;

        Ok(Self {
            model: Model {
                active_lane: 0,
                running_state: RunningState::MainView,
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
    match model.running_state {
        RunningState::MainView => main_view(model, frame),
        RunningState::Done => return,
    }
}

fn main_view(model: &mut Model, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(frame.area());

    for (lane, area) in model.lanes.iter_mut().zip(layout.into_iter()) {
        let lane_widget = LaneWidget {
            title: lane.for_state.into(),
        };
        frame.render_stateful_widget(&lane_widget, *area, &mut lane.state);
    }
}

fn handle_event(m: &Model) -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(m, key));
            }
        }
    }
    Ok(None)
}

fn handle_key(m: &Model, key: event::KeyEvent) -> Option<Message> {
    match m.running_state {
        RunningState::MainView => match key.code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Tab => Some(Message::NextLane),
            KeyCode::BackTab => Some(Message::PrevLane),
            KeyCode::Down | KeyCode::Char('j') => Some(Message::NextTask),
            KeyCode::Up | KeyCode::Char('k') => Some(Message::PrevTask),
            _ => None,
        },
        RunningState::Done => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
        Message::KeyPress(_key_event) => {}
        Message::NextLane => {
            model.lanes[model.active_lane].state.active = false;
            model.active_lane = (model.active_lane + 1) % model.lanes.len();
            model.lanes[model.active_lane].state.active = true;
        }
        Message::PrevLane => {
            model.lanes[model.active_lane].state.active = false;
            model.active_lane = (model.active_lane + model.lanes.len() - 1) % model.lanes.len();
            model.lanes[model.active_lane].state.active = true;
        }
        Message::NextTask => {
            model.lanes[model.active_lane].state.list_state.next();
        }
        Message::PrevTask => {
            model.lanes[model.active_lane].state.list_state.previous();
        }
    };
    None
}
