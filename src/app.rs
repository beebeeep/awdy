use anyhow::{Context, Result};
use rusqlite::{Connection, params, params_from_iter};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tui_widget_list::ListState;

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::Paragraph,
};

use crate::{
    color_scheme::COLOR_SCHEME,
    error_widget::ErrorWidget,
    lane_widget::{LaneState, LaneWidget},
    model::{Message, Model, RunningState, SelectedPane, Task, TaskState},
    selectlist_widget::{SelectList, SelectListState},
};

const ARCHIVE_TAG: &'static str = "Archive";

pub struct App<'a> {
    model: Model<'a>,
    db: rusqlite::Connection,
}

impl<'a> App<'a> {
    pub fn load(db_path: &str) -> Result<Self> {
        let mut tasks = HashMap::new();
        let db = match Connection::open(db_path).context("opening database") {
            Ok(db) => db,
            Err(_) => Connection::open("awdy.db").context("opening database")?,
        };

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
        }

        let mut lanes = Vec::with_capacity(4);
        for state in [
            TaskState::Todo,
            TaskState::InProgress,
            TaskState::Blocked,
            TaskState::Done,
        ] {
            tasks.insert(state, Vec::new());
            lanes.push(LaneState::new());
        }

        let tags_list = SelectListState {
            list_state: ListState::default(),
            items: Vec::new(),
        };

        lanes[0].selected = true;
        let mut r = Self {
            db,
            model: Model {
                active_lane: 0,
                active_pane: SelectedPane::Lanes,
                running_state: RunningState::MainView,
                tags: tags_list,
                tasks,
                lanes,
                task_view: None,
                last_error: None,
            },
        };

        r.update_filtered_tasks()?;
        r.update_tags()?;

        Ok(r)
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
                "Hint: use Tab and Shift+Tab to move between panes, arrows or hjkl for navigation. Move task between panes using keys 1,2,3,4. Enter opens task, <n> creates new task, <a> archives task"
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
        let panes =
            Layout::horizontal([Constraint::Percentage(10), Constraint::Fill(1)]).split(area);
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
                inactive: self.model.active_pane != SelectedPane::Tags,
            },
            panes[0],
            &mut self.model.tags,
        );

        for ((idx, mut lane), area) in self
            .model
            .lanes
            .iter_mut()
            .enumerate()
            .zip(lane_areas.iter())
        {
            let state = TaskState::from(idx as i32);
            let lane_widget = LaneWidget {
                title: state.into(),
                inactive: self.model.active_pane != SelectedPane::Lanes,
                tasks: self.model.tasks.get(&state).unwrap(),
            };
            frame.render_stateful_widget(&lane_widget, *area, &mut lane);
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
                KeyCode::Char('a') => Some(Message::ToggleTaskTag(String::from(ARCHIVE_TAG))),
                KeyCode::Right | KeyCode::Char('l') => Some(Message::NextLane),
                KeyCode::Left | KeyCode::Char('h') => Some(Message::PrevLane),
                KeyCode::Tab => Some(Message::NextPane),
                KeyCode::BackTab => Some(Message::PrevPane),
                KeyCode::Down | KeyCode::Char('j')
                    if self.model.active_pane == SelectedPane::Lanes =>
                {
                    Some(Message::NextTask)
                }
                KeyCode::Down | KeyCode::Char('j')
                    if self.model.active_pane == SelectedPane::Tags =>
                {
                    Some(Message::NextTag)
                }
                KeyCode::Up | KeyCode::Char('k')
                    if self.model.active_pane == SelectedPane::Lanes =>
                {
                    Some(Message::PrevTask)
                }
                KeyCode::Up | KeyCode::Char('k')
                    if self.model.active_pane == SelectedPane::Tags =>
                {
                    Some(Message::PrevTag)
                }
                KeyCode::Enter | KeyCode::Char('e')
                    if self.model.active_pane == SelectedPane::Lanes =>
                {
                    Some(Message::OpenTask)
                }
                KeyCode::Char(' ') if self.model.active_pane == SelectedPane::Tags => {
                    Some(Message::ToggleTag)
                }
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
            Message::NextPane => match self.model.active_pane {
                SelectedPane::Lanes => {
                    self.model.active_pane = SelectedPane::Tags;
                }
                SelectedPane::Tags => {
                    self.model.active_pane = SelectedPane::Lanes;
                }
            },
            Message::PrevPane => match self.model.active_pane {
                SelectedPane::Lanes => {
                    self.model.active_pane = SelectedPane::Tags;
                }
                SelectedPane::Tags => {
                    self.model.active_pane = SelectedPane::Lanes;
                }
            },
            Message::NextLane => {
                self.model.lanes[self.model.active_lane].selected = false;
                self.model.active_lane = (self.model.active_lane + 1) % self.model.lanes.len();
                self.model.lanes[self.model.active_lane].selected = true;
            }
            Message::PrevLane => {
                self.model.lanes[self.model.active_lane].selected = false;
                self.model.active_lane =
                    (self.model.active_lane + self.model.lanes.len() - 1) % self.model.lanes.len();
                self.model.lanes[self.model.active_lane].selected = true;
            }
            Message::NextTask => {
                self.model.lanes[self.model.active_lane].list_state.next();
            }
            Message::PrevTask => {
                self.model.lanes[self.model.active_lane]
                    .list_state
                    .previous();
            }
            Message::NextTag => {
                self.model.tags.list_state.next();
            }
            Message::PrevTag => {
                self.model.tags.list_state.previous();
            }
            Message::ToggleTag => {
                if let Some(idx) = self.model.tags.list_state.selected {
                    self.model.tags.items[idx].1 ^= true;
                    if let Err(e) = self.update_filtered_tasks() {
                        self.model.last_error = Some(e);
                    }
                }
            }
            Message::OpenTask => {
                let state = TaskState::from(self.model.active_lane as i32);
                let selected_task = self.model.lanes[self.model.active_lane]
                    .list_state
                    .selected
                    .unwrap();
                let lane_tasks = self.model.tasks.get(&state).unwrap();
                if lane_tasks.is_empty() {
                    return None;
                }
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
                let task = Task {
                    state: TaskState::from(self.model.active_lane as i32),
                    ..Default::default()
                };
                self.model.task_view = Some(task.into());
                self.model.running_state = RunningState::TaskView;
            }
            Message::CloseTask => {
                self.model.running_state = RunningState::MainView;
                self.model.task_view = None;
            }
            Message::SaveTask => {
                let mut task: Task = self.model.task_view.take().unwrap().into();
                match self.save_task(&mut task).context("saving task") {
                    Ok(t) => t,
                    Err(e) => {
                        self.model.last_error = Some(e);
                        self.model.task_view = Some(task.into());
                        return None;
                    }
                };
                if let Err(e) = self.update_tags().context("updating tag list") {
                    self.model.last_error = Some(e);
                    self.model.task_view = Some(task.into());
                    return None;
                }

                self.model.running_state = RunningState::MainView;
                let tasks = self.model.tasks.get_mut(&task.state).unwrap();
                for existing_task in tasks.iter_mut() {
                    if task.id == existing_task.id {
                        *existing_task = task;
                        return None;
                    }
                }
                tasks.push(task);
            }
            Message::MoveTask(to_state) => {
                let from_state = TaskState::from(self.model.active_lane as i32);
                if to_state == from_state {
                    return None;
                }
                let selected_task =
                    match self.model.lanes[self.model.active_lane].list_state.selected {
                        Some(v) => v,
                        None => return None,
                    };

                // first, update the db
                let from_tasks = self.model.tasks.get(&from_state).unwrap();
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

                // next, update current view in place
                let from_tasks = self.model.tasks.get_mut(&from_state).unwrap();
                // move focus upwards if last task in list was selected
                match self.model.lanes[self.model.active_lane].list_state.selected {
                    Some(idx) if idx >= from_tasks.len() - 1 => {
                        self.model.lanes[self.model.active_lane]
                            .list_state
                            .select(Some(from_tasks.len().saturating_sub(2)));
                    }
                    _ => {}
                }
                let task = from_tasks.remove(selected_task);
                self.model.tasks.get_mut(&to_state).unwrap().push(task);
            }
            Message::ToggleTaskTag(tag) => {
                let selected_task =
                    match self.model.lanes[self.model.active_lane].list_state.selected {
                        Some(v) => v,
                        None => return None,
                    };
                let mut task = {
                    let task = self
                        .model
                        .tasks
                        .get_mut(&self.model.active_lane.into())
                        .unwrap()
                        .get_mut(selected_task)?;
                    match task.tags.iter().position(|v| v == &tag) {
                        Some(idx) => {
                            task.tags.remove(idx);
                        }
                        None => {
                            task.tags.push(tag);
                        }
                    }
                    task.clone()
                };
                if let Err(e) = self.save_task(&mut task).context("saving tags") {
                    self.model.last_error = Some(e);
                    return None;
                }
                if let Err(e) = self.update_tags().context("updating tags") {
                    self.model.last_error = Some(e);
                    return None;
                }
                if let Err(e) = self.update_filtered_tasks().context("updating tasks view") {
                    self.model.last_error = Some(e);
                    return None;
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

    fn update_filtered_tasks(&mut self) -> Result<()> {
        for state in [
            TaskState::Todo,
            TaskState::InProgress,
            TaskState::Blocked,
            TaskState::Done,
        ] {
            self.model.tasks.get_mut(&state).unwrap().truncate(0);
        }

        let tags: Vec<_> = self
            .model
            .tags
            .items
            .iter()
            .filter(|x| x.1)
            .map(|x| x.0.clone())
            .collect();
        let placeholders = std::iter::repeat_n("?", tags.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = if tags.is_empty() {
            // All tasks except those with "Archive" tag
            "SELECT id, state, title, description FROM tasks WHERE
                NOT EXISTS (SELECT 1 FROM tags WHERE tags.task_id = tasks.id AND tags.tag = 'Archive')"
        } else {
            &format!(
                "SELECT id, state, title, description FROM tasks JOIN tags ON tags.task_id = tasks.id WHERE tags.tag IN ({})",
                placeholders
            )
        };
        let tx = self.db.transaction().context("loading tasks from DB")?;
        {
            let mut stmt = tx.prepare(sql)?;
            let rows = stmt
                .query_map(params_from_iter(tags), |r| {
                    Ok(Task {
                        id: Some(r.get::<usize, i64>(0)? as u64),
                        state: r.get::<usize, i32>(1)?.into(),
                        title: r.get(2)?,
                        description: r.get(3)?,
                        tags: Vec::new(),
                    })
                })
                .context("reading tasks from DB")?;
            for row in rows {
                let mut task = row.context("decoding task")?;
                let mut stmt =
                    tx.prepare("SELECT tag FROM tags WHERE task_id = ? ORDER BY tag DESC")?;
                for tag_row in stmt.query_map([task.id.unwrap() as i64], |r| r.get(0))? {
                    task.tags.push(tag_row?);
                }
                self.model.tasks.get_mut(&task.state).unwrap().push(task);
            }
        }
        tx.commit()?;

        // reset focus in task lists
        for lane in &mut self.model.lanes {
            lane.list_state.selected = Some(0);
        }

        Ok(())
    }

    fn update_tags(&mut self) -> Result<()> {
        let mut stmt = self
            .db
            .prepare("SELECT DISTINCT tag FROM tags ORDER BY tag DESC")
            .context("loading tags")?;
        let rows = stmt.query_map([], |r| r.get(0)).context("querying tags")?;

        let mut selected_tags = HashSet::new();
        for (tag, selected, _) in &self.model.tags.items {
            if *selected {
                selected_tags.insert(tag.clone());
            }
        }
        self.model.tags.items.truncate(0);
        for row in rows {
            let tag = row?;
            let selected = selected_tags.contains(&tag);
            let clean_mark = if tag == ARCHIVE_TAG { "-" } else { " " };
            self.model.tags.items.push((tag, selected, clean_mark));
        }
        let item_count = self.model.tags.items.len();
        match self.model.tags.list_state.selected {
            Some(idx) if idx >= item_count - 1 => {
                self.model
                    .tags
                    .list_state
                    .select(Some(item_count.saturating_sub(2)));
            }
            _ => {}
        }

        Ok(())
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
        stmt = self
            .db
            .prepare("SELECT tag FROM tags WHERE task_id = ? ORDER BY tag DESC")?;
        for row in stmt.query_map([id as i64], |r| r.get(0))? {
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

    // save_task persists task in DB. Sets task.id if needed
    fn save_task(&mut self, task: &mut Task) -> Result<()> {
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
                .query_map([id as i64], |r| r.get(0))
                .context("querying tags")?
            {
                old_tags.insert(row?);
            }
            let new_tags = HashSet::from_iter(task.tags.iter().cloned());
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

        task.id = Some(id);
        Ok(())
    }
}
