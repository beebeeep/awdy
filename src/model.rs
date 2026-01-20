use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ratatui::crossterm::event::KeyEvent;

use crate::{lane_widget, selectlist_widget::SelectListState, task_widget::TaskView};

pub(crate) struct Model<'a> {
    pub(crate) tasks: HashMap<TaskState, Rc<RefCell<Vec<TaskMeta>>>>,
    pub(crate) running_state: RunningState,

    pub(crate) active_lane: usize,
    pub(crate) lanes: Vec<LaneList>,
    pub(crate) tags_list: SelectListState,

    pub(crate) task_view: Option<TaskView<'a>>,
    pub(crate) last_error: Option<anyhow::Error>,
}

#[derive(Clone, PartialEq)]
pub(crate) struct TaskMeta {
    pub(crate) id: Option<u64>,
    pub(crate) state: TaskState,
    pub(crate) title: String,
}

#[derive(Default, Clone, PartialEq)]
pub(crate) struct Task {
    pub(crate) id: Option<u64>,
    pub(crate) state: TaskState,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    // pub(crate) assignees: Vec<String>,
    pub(crate) tags: Vec<String>,
}

pub(crate) struct LaneList {
    pub(crate) for_state: TaskState,
    pub(crate) state: lane_widget::LaneState,
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

impl From<Task> for TaskMeta {
    fn from(t: Task) -> Self {
        Self {
            id: t.id,
            state: t.state,
            title: t.title,
        }
    }
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
    CloseError,
    Quit,
}
