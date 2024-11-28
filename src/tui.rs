use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, Event},
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use matrix_sdk::{
    room::Room,
    event_handler::Ctx,
};
use std::{
    io,
    io::{Stdout, Write, Error},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    StoreCommand,
    store_crdt::{Store, SyncUpdateEvent},
};

enum StateMachine{
    Main,
    Credentials,
    Update,
    Query
}


pub struct TerminalUI {
    stdout: Stdout,
    width: u16,
    height: u16,
    main_ui_width: u16,
    notification_width: u16,
    notifications: Arc<Mutex<Vec<String>>>,
    store: Arc<Store>,

    sm: StateMachine,
}

impl TerminalUI {
    /// Create a new instance of `TerminalUI`.
    pub fn new(s: Store) -> Result<Self, Error>
    {
        let stdout = std::io::stdout();
        let (width, height) = terminal::size()?;
        let notification_width = (width as f32 * 0.3) as u16;
        let main_ui_width = width - notification_width;

        let  context = NotificationContext::default();
        let client = s.client.write().unwrap();
        client.add_event_handler_context(context.clone());
        client.add_event_handler(self::on_update);
        drop(client);

        let store_ref = Arc::new(s);
        let store_ref_2 = store_ref.clone();
        tokio::spawn( async move {
            store_ref_2.send_update(StoreCommand::Add(1,1)).await;
            store_ref_2.start_sync().await;
        });
        

        Ok(Self {
            stdout,
            width, height,
            main_ui_width, notification_width,
            notifications: context.list,
            store: store_ref,
            sm: StateMachine::Main,
        })
    }

    /// Main application loop for rendering the UI.
    pub fn run(&mut self) -> Result<(), io::Error> {
        terminal::enable_raw_mode()?; // Enable raw mode for full control over the terminal

        loop {
            let (new_width, new_height) = terminal::size()?;
            if new_width != self.width || new_height != self.height {
                self.resize(new_width, new_height);
            }

            self.stdout.execute(Clear(ClearType::All))?;
            self.draw_main_ui()?;
            self.draw_notifications()?;
            
            match self.sm{
                StateMachine::Main => {
                    if event::poll(std::time::Duration::from_millis(500))? {
                        match event::read()? {
                            Event::Key(KeyEvent {code: KeyCode::Char('q'),..}) => {break;}
                            Event::Key(KeyEvent {code: KeyCode::Char('1'),..}) => {self.sm = StateMachine::Update;}
                            Event::Key(KeyEvent {code: KeyCode::Char('2'),..}) => {self.sm = StateMachine::Update;}
                            _ => {}
                        }
                    }
                },
                StateMachine::Update => {
                    self.sm = StateMachine::Main;
                    let store_ref = self.store.clone();
                    tokio::spawn( async move {
                        store_ref.send_update(StoreCommand::Add(1,1)).await;
                    });
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
                        if let Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        }) = event::read()?
                        {
                            break; // Exit on 'q' key press
                        }
                    }
                }
            }



            // Simulate some main loop logic
            thread::sleep(Duration::from_millis(100));
        }

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
            },
            StateMachine::Credentials => {
                self.stdout.execute(cursor::MoveTo(2, 2))?;
                self.stdout.write_all(b"Credentials")?;
            },
            StateMachine::Query => {
                self.stdout.execute(cursor::MoveTo(2, 2))?;
                self.stdout.write_all(b"Query Menu")?;

                let res = self.store.query(&1);
                self.stdout.execute(cursor::MoveTo(2, 3))?;
                let output = format!("Query Result: {}", res);
                self.stdout.write_all(&output.into_bytes())?;
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
            }
        }

        Ok(())
    }
}


#[derive(Debug, Default, Clone)]
pub struct NotificationContext{
    list: Arc<Mutex<Vec<String>>>,
}

pub async fn on_update(event: SyncUpdateEvent, room: Room, ctx: Ctx<NotificationContext>){
    let mut list = ctx.list.lock().unwrap();
    list.push("new event".to_string());
}
