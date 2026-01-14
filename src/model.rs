use tui_widget_list::ListState;

use crate::lane::LaneState;

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) tasks: Vec<Task>,
    pub(crate) running_state: RunningState,
    pub(crate) lanes: Vec<LaneList>,
}

#[derive(Debug, Default)]
pub(crate) struct Task {
    pub(crate) state: TaskState,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    // pub(crate) assignees: Vec<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct LaneList {
    pub(crate) task_state: TaskState,
    pub(crate) state: LaneState,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default)]
pub(crate) enum Mode {
    #[default]
    Main,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
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
    Increment,
    Decrement,
    Reset,
    Quit,
}
