use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, params};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    time::Duration,
};
use tui_widget_list::ListState;

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    color_scheme::COLOR_SCHEME,
    error_widget::ErrorWidget,
    lane_widget::{LaneState, LaneWidget},
    model::{LaneList, Message, Model, RunningState, Task, TaskMeta, TaskState},
    selectlist_widget::{SelectList, SelectListState},
    task_widget::TaskView,
};

pub struct App<'a> {
    model: Model<'a>,
    db: rusqlite::Connection,
}

impl<'a> App<'a> {
    pub fn load() -> Result<Self> {
        let mut tasks = HashMap::new();
        let db = Connection::open("awdy.db").context("opening database")?;
        let mut tags = Vec::new();

        {
            db.execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                state INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT
            )",
                (),
            )
            .context("initializing database")?;
            db.execute(
                "CREATE TABLE IF NOT EXISTS tags (
                tag TEXT NOT NULL,
                task_id INTEGER NOT NULL,
                PRIMARY KEY (tag, task_id)
            )",
                (),
            )
            .context("initializing database")?;
            let mut stmt = db
                .prepare("SELECT id, state, title FROM tasks")
                .context("loading tasks")?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(TaskMeta {
                        id: Some(r.get::<usize, i64>(0)? as u64),
                        state: r.get::<usize, i32>(1)?.into(),
                        title: r.get(2)?,
                    })
                })
                .context("reading tasks from DB")?;
            for result in rows {
                let task = result.context("decoding task")?;
                match tasks.get_mut(&task.state) {
                    None => {
                        tasks.insert(task.state, Rc::new(RefCell::new(vec![task])));
                    }
                    Some(v) => {
                        let mut v = v.borrow_mut();
                        v.push(task);
                    }
                }
            }

            let mut stmt = db
                .prepare("SELECT DISTINCT tag FROM tags ORDER BY tag DESC")
                .context("loading tags")?;
            let rows = stmt
                .query_map([], |r| Ok(r.get(0)?))
                .context("querying tags")?;
            for row in rows {
                tags.push(row?);
            }
        }

        let mut lanes = Vec::with_capacity(4);
        for state in [
            TaskState::Todo,
            TaskState::InProgress,
            TaskState::Blocked,
            TaskState::Done,
        ] {
            if !tasks.contains_key(&state) {
                tasks.insert(state, Rc::new(RefCell::new(Vec::new())));
            }
            lanes.push(LaneList {
                for_state: state,
                state: LaneState::new(false, tasks.get(&state).unwrap().clone()),
            });
        }

        let tags_list = SelectListState {
            list_state: ListState::default(),
            items: tags.into_iter().map(|v| (v, false)).collect(),
            active: false,
        };

        lanes[0].state.active = true;
        Ok(Self {
            db,
            model: Model {
                active_lane: 0,
                running_state: RunningState::MainView,
                tags_list,
                tasks,
                lanes,
                task_view: None,
                last_error: None,
            },
        })
    }

    pub fn run(mut self) -> Result<()> {
        let mut terminal = ratatui::try_init()?;

        while self.model.running_state != RunningState::Done {
            // Render the current view
            terminal.draw(|f| self.view(f))?;

            // Handle events and map to a Message
            let mut current_msg = self.handle_event()?;

            // Process updates as long as they return a non-None message
            while current_msg.is_some() {
                current_msg = self.update(current_msg.unwrap());
            }
        }
        ratatui::restore();
        Ok(())
    }
    fn view(&mut self, frame: &mut Frame) {
        let layout =
            Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]).split(frame.area());
        self.status_bar(frame, layout[1]);
        match &self.model.running_state {
            RunningState::MainView => self.main_view(frame, layout[0]),
            RunningState::TaskView => self.task_view(frame, layout[0]),
            RunningState::Done => {}
        }
        if self.model.last_error.is_some() {
            self.show_error(frame, layout[0]);
        }
    }

    fn status_bar(&self, frame: &mut Frame, area: Rect) {
        let text = match self.model.running_state {
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

    fn main_view(&mut self, frame: &mut Frame, area: Rect) {
        let panes = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);
        let lane_areas = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(panes[1]);

        frame.render_stateful_widget(
            &SelectList {
                title: "Tags".to_string(),
            },
            panes[0],
            &mut self.model.tags_list,
        );

        for (lane, area) in self.model.lanes.iter_mut().zip(lane_areas.iter()) {
            let lane_widget = LaneWidget {
                title: lane.for_state.into(),
            };
            frame.render_stateful_widget(&lane_widget, *area, &mut lane.state);
        }
    }

    fn task_view(&mut self, frame: &mut Frame, area: Rect) {
        self.main_view(frame, area); // draw lanes in background to keep visual context
        if let Some(v) = self.model.task_view.as_ref() {
            frame.render_widget(v, area);
        }
    }

    fn show_error(&mut self, frame: &mut Frame, area: Rect) {
        let p = ErrorWidget {
            title: "ERROR".to_string(),
            message: format!("{:#}", self.model.last_error.as_ref().unwrap()),
        };
        frame.render_widget(p, area);
    }

    fn handle_event(&self) -> Result<Option<Message>> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    return Ok(self.handle_key(key));
                }
            }
        }
        Ok(None)
    }

    fn handle_key(&self, key: event::KeyEvent) -> Option<Message> {
        if self.model.last_error.is_some() {
            return Some(Message::CloseError);
        }

        match self.model.running_state {
            RunningState::MainView => match key.code {
                KeyCode::Char('q') => Some(Message::Quit),
                KeyCode::Char('n') => Some(Message::NewTask),
                KeyCode::Char('1') => Some(Message::MoveTask(TaskState::Todo)),
                KeyCode::Char('2') => Some(Message::MoveTask(TaskState::InProgress)),
                KeyCode::Char('3') => Some(Message::MoveTask(TaskState::Blocked)),
                KeyCode::Char('4') => Some(Message::MoveTask(TaskState::Done)),
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

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Quit => {
                self.model.running_state = RunningState::Done;
            }
            Message::CloseError => self.model.last_error = None,
            Message::NextLane => {
                self.model.lanes[self.model.active_lane].state.active = false;
                self.model.active_lane = (self.model.active_lane + 1) % self.model.lanes.len();
                self.model.lanes[self.model.active_lane].state.active = true;
            }
            Message::PrevLane => {
                self.model.lanes[self.model.active_lane].state.active = false;
                self.model.active_lane =
                    (self.model.active_lane + self.model.lanes.len() - 1) % self.model.lanes.len();
                self.model.lanes[self.model.active_lane].state.active = true;
            }
            Message::NextTask => {
                self.model.lanes[self.model.active_lane]
                    .state
                    .list_state
                    .next();
            }
            Message::PrevTask => {
                self.model.lanes[self.model.active_lane]
                    .state
                    .list_state
                    .previous();
            }
            Message::OpenTask => {
                let selected_task = self.model.lanes[self.model.active_lane]
                    .state
                    .list_state
                    .selected
                    .unwrap();
                let lane_tasks = self
                    .model
                    .tasks
                    .get(&self.model.lanes[self.model.active_lane].for_state)
                    .unwrap()
                    .borrow();
                self.model.task_view = match self.load_task(lane_tasks[selected_task].id.unwrap()) {
                    Ok(t) => Some(t.into()),
                    Err(e) => {
                        self.model.last_error = Some(e);
                        return None;
                    }
                };
                self.model.running_state = RunningState::TaskView;
            }
            Message::NewTask => {
                let mut task = Task::default();
                task.state = self.model.lanes[self.model.active_lane].for_state;
                self.model.task_view = Some(task.into());
                self.model.running_state = RunningState::TaskView;
            }
            Message::CloseTask => {
                self.model.running_state = RunningState::MainView;
                self.model.task_view = None;
            }
            Message::SaveTask => {
                let task: Task = self.model.task_view.take().unwrap().into();
                let task_meta = match self.save_task(&task).context("saving task") {
                    Ok(t) => t,
                    Err(e) => {
                        self.model.last_error = Some(e);
                        self.model.task_view = Some(task.into());
                        return None;
                    }
                };
                self.model.running_state = RunningState::MainView;
                let mut tasks = self
                    .model
                    .tasks
                    .get_mut(&task_meta.state)
                    .unwrap()
                    .borrow_mut();
                for existing_task in tasks.iter_mut() {
                    if task_meta.id == existing_task.id {
                        *existing_task = task_meta;
                        return None;
                    }
                }
                tasks.push(task_meta);
            }
            Message::MoveTask(to_state) => {
                let from_state = self.model.lanes[self.model.active_lane].for_state;
                if to_state == from_state {
                    return None;
                }
                let selected_task = match self.model.lanes[self.model.active_lane]
                    .state
                    .list_state
                    .selected
                {
                    Some(v) => v,
                    None => return None,
                };
                let mut from_tasks = self.model.tasks.get(&from_state).unwrap().borrow_mut();
                if from_tasks.len() == 0 {
                    return None;
                }

                match self.update_task_state(to_state, from_tasks[selected_task].id.unwrap()) {
                    Ok(_) => {}
                    Err(e) => {
                        self.model.last_error = Some(e);
                        return None;
                    }
                };

                let task = from_tasks.remove(selected_task);
                let mut to_tasks = self.model.tasks.get(&to_state).unwrap().borrow_mut();
                to_tasks.push(task);

                // move focus upwards if last task in list was selected
                match self.model.lanes[self.model.active_lane]
                    .state
                    .list_state
                    .selected
                {
                    Some(idx) if idx >= from_tasks.len() => {
                        self.model.lanes[self.model.active_lane]
                            .state
                            .list_state
                            .select(Some(from_tasks.len().saturating_sub(1)));
                    }
                    _ => {}
                }
            }
            Message::FocusNext => match self.model.running_state {
                RunningState::TaskView => {
                    if let Some(tv) = self.model.task_view.as_mut() {
                        tv.next_field();
                    }
                }
                _ => {}
            },
            Message::FocusPrev => match self.model.running_state {
                RunningState::TaskView => {
                    if let Some(tv) = self.model.task_view.as_mut() {
                        tv.prev_field();
                    }
                }
                _ => {}
            },
            Message::KeyPress(event) => match self.model.running_state {
                RunningState::TaskView => {
                    if let Some(tv) = self.model.task_view.as_mut() {
                        tv.process_event(event);
                    }
                }
                _ => {}
            },
        };
        None
    }

    fn load_task(&self, id: u64) -> Result<Task> {
        let mut stmt = self
            .db
            .prepare("SELECT state, title, description FROM tasks WHERE id = ?")?;
        let mut task = stmt.query_row([id as i64], |r| {
            Ok(Task {
                id: Some(id),
                state: r.get::<usize, i32>(0)?.into(),
                title: r.get(1)?,
                description: r.get(2)?,
                tags: Vec::new(),
            })
        })?;
        stmt = self.db.prepare("SELECT tag FROM tags WHERE task_id = ?")?;
        for row in stmt.query_map([id as i64], |r| Ok(r.get(0)?))? {
            task.tags.push(row?);
        }
        Ok(task)
    }

    fn update_task_state(&self, state: TaskState, id: u64) -> Result<()> {
        self.db
            .execute(
                "UPDATE tasks SET state = ? WHERE id = ?",
                params![state as i32, id as i64],
            )
            .context("updating task state")?;
        Ok(())
    }

    fn save_task(&mut self, task: &Task) -> Result<TaskMeta> {
        let (sql, params) = match task.id {
            Some(id) => (
                "UPDATE tasks SET state=?,title=?,description=? WHERE id=?",
                params![task.state as i32, task.title, task.description, id as i64],
            ),
            None => (
                "INSERT INTO tasks (state, title, description) VALUES (?, ?, ?)",
                params![task.state as i32, task.title, task.description],
            ),
        };

        let tx = self.db.transaction().context("starting transaction")?;
        let id;
        {
            tx.execute(sql, params).context("saving task")?;
            id = match task.id {
                Some(id) => id,
                None => tx.last_insert_rowid() as u64,
            };

            let mut stmt = tx
                .prepare("SELECT tag FROM tags WHERE task_id=?")
                .context("querying tags")?;
            let mut old_tags: HashSet<String> = HashSet::new();
            for row in stmt
                .query_map([id as i64], |r| Ok(r.get(0)?))
                .context("querying tags")?
            {
                old_tags.insert(row?);
            }
            let new_tags = HashSet::from_iter(task.tags.iter().map(|x| x.clone()));
            let tags_to_remove = old_tags.difference(&new_tags);
            let tags_to_add = new_tags.difference(&old_tags);
            let mut stmt = tx
                .prepare("INSERT INTO tags (tag, task_id) VALUES (?, ?)")
                .context("inserting new task tags")?;
            for tag in tags_to_add {
                stmt.execute(params![tag, id as i64])
                    .context("inserting new tags")?;
            }
            let mut stmt = tx
                .prepare("DELETE FROM tags WHERE tag = ? AND task_id = ?")
                .context("removing task old tags")?;
            for tag in tags_to_remove {
                stmt.execute(params![tag, id as i64])
                    .context("removing task old tags")?;
            }
        }
        tx.commit()?;

        Ok(TaskMeta {
            id: Some(id),
            state: task.state,
            title: task.title.clone(),
        })
    }
}
