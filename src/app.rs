use anyhow::Result;
use crossterm::event::KeyEvent;
use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};
use tui_textarea::TextArea;

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    lane_widget::{LaneState, LaneWidget},
    model::{LaneList, Message, Model, RunningState, Task, TaskState},
    task_widget::TaskView,
};

pub struct App<'a> {
    model: Model<'a>,
}

impl<'a> App<'a> {
    pub fn load() -> Result<Self> {
        let all_tasks = vec![
            Task {
                id: Some(1),
                state: TaskState::Todo,
                title: "Do stuff".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                id: Some(2),
                state: TaskState::Todo,
                title: "Do more stuff".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                id: Some(3),
                state: TaskState::Todo,
                title: "Don't do shit".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                id: Some(4),
                state: TaskState::InProgress,
                title: "Writing shit".to_string(),
                description: Some("lorem ipsum".to_string()),
                tags: vec!["chlos".to_string()],
            },
            Task {
                id: Some(5),
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
                task_view: None,
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
        RunningState::TaskView => task_view(model, frame),
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

fn task_view(model: &mut Model, frame: &mut Frame) {
    main_view(model, frame); // draw lanes in background to keep visual context
    if let Some(v) = &model.task_view {
        frame.render_widget(v, frame.area());
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
            KeyCode::Enter => Some(Message::OpenTask),
            _ => None,
        },
        RunningState::TaskView => match key {
            event::KeyEvent {
                code: KeyCode::Tab, ..
            } => Some(Message::FocusNext),
            event::KeyEvent {
                code: KeyCode::BackTab,
                ..
            } => Some(Message::FocusPrev),
            event::KeyEvent {
                code: KeyCode::Esc, ..
            } => Some(Message::CloseTask),
            event::KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => Some(Message::SaveTask),

            event => Some(Message::KeyPress(event)),
        },
        RunningState::Done => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
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
        Message::OpenTask => {
            let selected_task = model.lanes[model.active_lane]
                .state
                .list_state
                .selected
                .unwrap();
            let lane_tasks = model
                .tasks
                .get(&model.lanes[model.active_lane].for_state)
                .unwrap()
                .borrow();
            model.task_view = Some(TaskView::from_task(lane_tasks[selected_task].clone()));
            model.running_state = RunningState::TaskView;
        }
        Message::CloseTask => {
            model.running_state = RunningState::MainView;
            model.task_view = None;
        }
        Message::SaveTask => {}
        Message::FocusNext => match model.running_state {
            RunningState::TaskView => {
                if let Some(tv) = model.task_view.as_mut() {
                    tv.active_text_area = (tv.active_text_area + 1) % tv.text_areas.len();
                }
            }
            _ => {}
        },
        Message::FocusPrev => match model.running_state {
            RunningState::TaskView => {
                if let Some(tv) = model.task_view.as_mut() {
                    tv.active_text_area =
                        (tv.active_text_area + tv.text_areas.len() - 1) % tv.text_areas.len();
                }
            }
            _ => {}
        },
        Message::KeyPress(event) => match model.running_state {
            RunningState::TaskView => {
                if let Some(tv) = model.task_view.as_mut() {
                    // TODO: too much logic goes here, seems like we need to offload it to TaskView
                    tv.text_areas[tv.active_text_area].input(event);
                }
            }
            _ => {}
        },
    };
    None
}
