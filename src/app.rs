use anyhow::Result;
use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    color_scheme::COLOR_SCHEME,
    lane_widget::{LaneState, LaneWidget},
    model::{LaneList, Message, Model, RunningState, Task, TaskState},
    task_widget::TaskView,
};

pub struct App<'a> {
    model: Model<'a>,
}

impl<'a> App<'a> {
    pub fn load() -> Result<Self> {
        let all_tasks = [
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
                    .filter(|t| t.state == state)
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
    let layout = Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]).split(frame.area());
    status_bar(model, frame, layout[1]);
    match model.running_state {
        RunningState::MainView => main_view(model, frame, layout[0]),
        RunningState::TaskView => task_view(model, frame, layout[0]),
        RunningState::Done => {}
    }
}

fn status_bar(model: &mut Model, frame: &mut Frame, area: Rect) {
    let text = match model.running_state {
        RunningState::MainView => {
            "Hint: use Tab and Shift+Tab to move between lanes, arrows or hjkl for navigation. Enter opens task."
        }
        RunningState::TaskView => {
            "Hint: Esc to close without saving, Ctrl+S to save and close. Tags are comma-separated"
        }
        RunningState::Done => return,
    };

    let c = Paragraph::new(text)
        .bg(COLOR_SCHEME.status_bar_bg)
        .fg(COLOR_SCHEME.status_bar_fg);
    frame.render_widget(c, area);
}

fn main_view(model: &mut Model, frame: &mut Frame, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    for (lane, area) in model.lanes.iter_mut().zip(layout.iter()) {
        let lane_widget = LaneWidget {
            title: lane.for_state.into(),
        };
        frame.render_stateful_widget(&lane_widget, *area, &mut lane.state);
    }
}

fn task_view(model: &mut Model, frame: &mut Frame, area: Rect) {
    main_view(model, frame, area); // draw lanes in background to keep visual context
    if let Some(v) = &model.task_view {
        frame.render_widget(v, area);
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
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => Some(Message::NextLane),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => Some(Message::PrevLane),
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
            model.task_view = Some(lane_tasks[selected_task].clone().into());
            model.running_state = RunningState::TaskView;
        }
        Message::CloseTask => {
            model.running_state = RunningState::MainView;
            model.task_view = None;
        }
        Message::SaveTask => {
            model.running_state = RunningState::MainView;
            let task: Task = model.task_view.take().unwrap().into();
            let mut tasks = model.tasks.get_mut(&task.state).unwrap().borrow_mut();
            for existing_task in tasks.iter_mut() {
                if task.id == existing_task.id {
                    *existing_task = task;
                    return None;
                }
            }
            tasks.push(task);
        }
        Message::FocusNext => match model.running_state {
            RunningState::TaskView => {
                if let Some(tv) = model.task_view.as_mut() {
                    tv.next_field();
                }
            }
            _ => {}
        },
        Message::FocusPrev => match model.running_state {
            RunningState::TaskView => {
                if let Some(tv) = model.task_view.as_mut() {
                    tv.prev_field();
                }
            }
            _ => {}
        },
        Message::KeyPress(event) => match model.running_state {
            RunningState::TaskView => {
                if let Some(tv) = model.task_view.as_mut() {
                    tv.process_event(event);
                }
            }
            _ => {}
        },
    };
    None
}
