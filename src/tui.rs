use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, Event},
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
//use matrix_sdk::{
  //  room::Room,
    //event_handler::Ctx,
    //config::SyncSettings,
//};
use std::{
    io,
    io::{Stdout, Write, Error},
    sync::{Arc,Mutex},
    thread,
    time::Duration,
};

use crate::{
    crdt::crdt::CRDT,
    crdt::merkle_dag::dag::QueryRecord,
};

pub enum StateMachine{
    Main,
    Credentials,
    Update,
    Query,
    PrettyPrint,
}


pub struct TerminalUI {
    stdout: Stdout,
    width: u16,
    height: u16,
    main_ui_width: u16,
    notification_width: u16,
    notifications: Arc<Mutex<Vec<String>>>,
    store: Arc<CRDT>,
    query_log: QueryRecord,

    sm: StateMachine,
    update_input: String,
}

impl TerminalUI {
    /// Create a new instance of `TerminalUI`.
    pub async fn new(s: Arc<CRDT>) -> Result<Self, Error> {
        let stdout = std::io::stdout();
        let (width, height) = terminal::size()?;
        let notification_width = (width as f32 * 0.3) as u16;
        let main_ui_width = width - notification_width;

        let context = Arc::new(Mutex::new(vec!["Messages".to_string()]));
        let list_ref = Arc::clone(&context);

        //s.room.add_event_handler(self::on_update);
        //let s_ref = Arc::clone(&s);
        //let client = s_ref.client.write().await;
        //client.add_event_handler_context(context);

        Ok(Self {
            stdout,
            width, height,
            main_ui_width, notification_width,
            notifications: list_ref,
            store: s,
            sm: StateMachine::Main,
            query_log: Default::default(),
            update_input: "".to_string()
        })
    }

    pub fn handle_update_input(&mut self) {
        if let Ok(Event::Key(key_event)) = event::read() {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.update_input.push(c);
                }
                KeyCode::Backspace => {
                    if !self.update_input.is_empty() {
                        self.update_input.pop();
                    }
                }
                KeyCode::Enter => {
                    self.sm = StateMachine::Main;

                    let store_ref = Arc::clone(&self.store);
                    let input = self.update_input.clone();
                    self.update_input.clear();
                    tokio::spawn( async move {
                        store_ref.send_update(input).await;
                    });
                }
                KeyCode::Esc => {
                    self.sm = StateMachine::Main;
                    // Exit if Escape is pressed
                }
                _ => {}
            }
        }
    }

    /// Main application loop for rendering the UI.
    pub fn run(&mut self) -> Result<(), io::Error> {
        loop {
            let (new_width, new_height) = terminal::size()?;
            if new_width != self.width || new_height != self.height {
                self.resize(new_width, new_height);
            }
            
            terminal::enable_raw_mode()?; // Enable raw mode for full control over the terminal
            match self.sm{
                StateMachine::Main => {
                    if event::poll(std::time::Duration::from_millis(500))? {
                        match event::read()? {
                            Event::Key(KeyEvent {code: KeyCode::Char('q'),..}) => {break;}
                            Event::Key(KeyEvent {code: KeyCode::Char('1'),..}) => {self.sm = StateMachine::Update;}
                            Event::Key(KeyEvent {code: KeyCode::Char('2'),..}) => {self.sm = StateMachine::Query;}
                            Event::Key(KeyEvent {code: KeyCode::Char('3'),..}) => {self.sm = StateMachine::PrettyPrint;}
                            _ => {}
                        }
                    }
                },
                StateMachine::Update => {
                    self.handle_update_input();
                },
                StateMachine::Credentials => {
                    if event::poll(std::time::Duration::from_millis(500))? {
                        if let Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        }) = event::read()?
                        {
                            break; // Exit on 'q' key press
                        }
                    }
                },
                StateMachine::Query => {
                    if event::poll(std::time::Duration::from_millis(500))? {
                        if let Event::Key(KeyEvent {code: KeyCode::Char('q'),..}) = event::read()?
                        {
                            self.sm = StateMachine::Main;
                        }
                    }
                },
                StateMachine::PrettyPrint => {
                    if event::poll(std::time::Duration::from_millis(500))? {
                        if let Event::Key(KeyEvent {code: KeyCode::Char('q'),..}) = event::read()?
                        {
                            self.sm = StateMachine::Main;
                        }
                    }
                }
            }

            self.stdout.execute(Clear(ClearType::All))?;
            self.draw_main_ui()?;
            self.draw_notifications()?;

            // Simulate some main loop logic
            thread::sleep(Duration::from_millis(100));
        }

        self.stdout.execute(Clear(ClearType::All))?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    /// Handle terminal resizing.
    fn resize(&mut self, width: u16, height: u16){
        self.width = width;
        self.height = height;
        self.notification_width = (width as f32 * 0.3) as u16;
        self.main_ui_width = width - self.notification_width;
    }

    /// Draw the maCredentialsin UI section.
    fn draw_main_ui(&mut self) -> Result<(), Error> {
        // Draw a border for the main UI area
        for y in 0..self.height {
            self.stdout.execute(cursor::MoveTo(0, y))?;
            if y == 0 || y == self.height - 1 {
                // Top and bottom border
                self.stdout.write_all(&vec![b'-'; self.main_ui_width as usize])?;
            } else {
                // Left and right border
                self.stdout.write_all(b"|")?;
                self.stdout
                    .execute(cursor::MoveTo(self.main_ui_width - 1, y))?;
                self.stdout.write_all(b"|")?;
            }
        }
        
        match self.sm {
            StateMachine::Main => {
                self.stdout.execute(cursor::MoveTo(2, 2))?;
                self.stdout.write_all(b"Main Menu")?;
                self.stdout.execute(cursor::MoveTo(2, 3))?;
                self.stdout.write_all(b"1: update")?;
                self.stdout.execute(cursor::MoveTo(2, 4))?;
                self.stdout.write_all(b"2: query")?;
                self.stdout.execute(cursor::MoveTo(2, 5))?;
                self.stdout.write_all(b"3: print dag")?;
            },
            StateMachine::Update => {
                self.stdout.execute(cursor::MoveTo(2, 2))?;
                self.stdout.write_all(b"Update Menu")?;
                self.stdout.execute(cursor::MoveTo(2, 3))?;
                self.stdout.execute(Print(self.update_input.clone()));
            },
            StateMachine::Credentials => {
                self.stdout.execute(cursor::MoveTo(2, 2))?;
                self.stdout.write_all(b"Credentials")?;
            },
            StateMachine::Query => {
                self.stdout.execute(cursor::MoveTo(2, 2))?;
                self.stdout.write_all(b"Query Menu")?;

                let notifications = Arc::clone(&self.notifications);
                let log = self.store.query(|node| {
                    let mut not_lock = notifications.lock().unwrap();
                    not_lock.push(node.data.clone());
                }, Some(self.query_log.clone()));

                self.query_log = log;
                self.stdout.execute(cursor::MoveTo(2, 3))?;
                let output = format!("Query Done");
                self.stdout.write_all(&output.into_bytes())?;
            },
            StateMachine::PrettyPrint => {
                self.stdout.execute(cursor::MoveTo(2, 2))?;
                let pretty_map = self.store.pretty_print_dag();
                for c in pretty_map.chars() {
                    if c == '\n' {
                        self.stdout.execute(
                            cursor::MoveTo(2, crossterm::cursor::position()?.1 + 1))?;
                    } else {
                        // Print the character normally
                        self.stdout.execute(Print(c))?;
                        self.stdout.flush()?; // Flush the output to ensure immediate rendering
                    }
                }
            }
        }

        self.stdout.execute(cursor::MoveTo(2, self.height-2))?;
        self.stdout.write_all(b"q: exit the program\n")?;

        Ok(())
    }

    /// Draw the notifications section.
    fn draw_notifications(&mut self) -> Result<(), Error> {
        let start_x = self.main_ui_width;

        // Draw a border for the notifications area
        for y in 0..self.height {
            self.stdout.execute(cursor::MoveTo(start_x, y))?;
            if y == 0 || y == self.height - 1 {
                // Top and bottom border
                self.stdout
                    .write_all(&vec![b'-'; self.notification_width as usize])?;
            } else {
                // Left border
                self.stdout.write_all(b"|")?;
                self.stdout
                    .execute(cursor::MoveTo(start_x + self.notification_width - 1, y))?;
                self.stdout.write_all(b"|")?;
            }
        }

        // Print the notifications
        let notifs = self.notifications.lock().unwrap();
        for (i, notif) in notifs.iter().enumerate() {
            if (i as u16) < self.height - 2 {
                self.stdout
                    .execute(cursor::MoveTo(start_x + 2, i as u16 + 1))?;
                self.stdout.execute(SetForegroundColor(Color::Yellow))?;
                self.stdout.execute(Print(notif))?;
                self.stdout.execute(SetForegroundColor(Color::White))?;
            }
        }

        Ok(())
    }
}

