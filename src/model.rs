use std::{collections::HashMap, fmt::Display};

use ratatui::crossterm::event::KeyEvent;

use crate::{lane_widget::LaneState, selectlist_widget::SelectListState, task_widget::TaskView};

pub(crate) struct Model<'a> {
    pub(crate) tasks: HashMap<TaskState, Vec<Task>>,
    pub(crate) running_state: RunningState,

    pub(crate) active_pane: SelectedPane,
    pub(crate) active_lane: usize,
    pub(crate) lanes: Vec<LaneState>,
    pub(crate) tags: SelectListState,

    pub(crate) task_view: Option<TaskView<'a>>,
    pub(crate) last_error: Option<anyhow::Error>,
}

#[derive(Default, Clone, PartialEq)]
pub(crate) struct Task {
    pub(crate) id: Option<u64>,
    pub(crate) state: TaskState,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Default, PartialEq)]
pub(crate) enum SelectedPane {
    #[default]
    Lanes,
    Tags,
}
#[derive(Default, PartialEq)]
pub(crate) enum RunningState {
    #[default]
    MainView,
    TaskView,
    Done,
}

#[derive(Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum TaskState {
    #[default]
    Todo = 0,
    InProgress = 1,
    Blocked = 2,
    Done = 3,
}

impl Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str((*self).into())
    }
}

impl From<TaskState> for &'static str {
    fn from(value: TaskState) -> Self {
        match value {
            TaskState::Todo => "TODO",
            TaskState::InProgress => "In progress",
            TaskState::Blocked => "Blocked",
            TaskState::Done => "Done",
        }
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
    NextPane,
    PrevPane,
    NextLane,
    PrevLane,
    NextTask,
    PrevTask,
    NextTag,
    PrevTag,
    ToggleTag,
    NewTask,
    OpenTask,
    CloseTask,
    SaveTask,
    MoveTask(TaskState),
    FocusNext,
    FocusPrev,
    CloseError,
    Quit,
}
