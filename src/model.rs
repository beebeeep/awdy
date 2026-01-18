use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ratatui::crossterm::event::KeyEvent;

use crate::{lane_widget, task_widget::TaskView};

#[derive(Debug)]
pub(crate) struct Model<'a> {
    pub(crate) tasks: HashMap<TaskState, Rc<RefCell<Vec<TaskMeta>>>>,
    pub(crate) running_state: RunningState,

    // used by MainView
    pub(crate) active_lane: usize,
    pub(crate) lanes: Vec<LaneList>,

    pub(crate) task_view: Option<TaskView<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct TaskMeta {
    pub(crate) id: Option<u64>,
    pub(crate) state: TaskState,
    pub(crate) title: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Task {
    pub(crate) id: Option<u64>,
    pub(crate) state: TaskState,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    // pub(crate) assignees: Vec<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct LaneList {
    pub(crate) for_state: TaskState,
    pub(crate) state: lane_widget::LaneState,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) enum RunningState {
    #[default]
    MainView,
    TaskView,
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

impl From<TaskState> for String {
    fn from(t: TaskState) -> Self {
        match t {
            TaskState::Todo => "TODO",
            TaskState::InProgress => "In progress",
            TaskState::Blocked => "Blocked",
            TaskState::Done => "Done",
        }
        .to_string()
    }
}

impl From<i32> for TaskState {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::InProgress,
            2 => Self::Blocked,
            3 => Self::Done,
            _ => Self::Todo,
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum Message {
    KeyPress(KeyEvent),
    NextLane,
    PrevLane,
    NextTask,
    PrevTask,
    NewTask,
    OpenTask,
    CloseTask,
    SaveTask,
    MoveTask(TaskState),
    FocusNext,
    FocusPrev,
    Quit,
}
