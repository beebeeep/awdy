use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::lane;
use crossterm::event::KeyEvent;

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) tasks: HashMap<TaskState, Rc<RefCell<Vec<Task>>>>,
    pub(crate) running_state: RunningState,
    pub(crate) active_lane: usize,
    pub(crate) lanes: Vec<LaneList>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Task {
    pub(crate) state: TaskState,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    // pub(crate) assignees: Vec<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct LaneList {
    pub(crate) for_state: TaskState,
    pub(crate) state: lane::LaneState,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) enum RunningState {
    #[default]
    MainView,
    Done,
}

#[derive(Hash, Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum TaskState {
    #[default]
    Todo,
    InProgress,
    Blocked,
    Done,
}

impl Into<String> for TaskState {
    fn into(self) -> String {
        match self {
            TaskState::Todo => "TODO",
            TaskState::InProgress => "In progress",
            TaskState::Blocked => "Blocked",
            TaskState::Done => "Done",
        }
        .to_string()
    }
}

#[derive(PartialEq)]
pub(crate) enum Message {
    KeyPress(KeyEvent),
    NextLane,
    PrevLane,
    NextTask,
    PrevTask,
    Quit,
}
