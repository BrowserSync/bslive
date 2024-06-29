use self::common::{init_terminal, install_hooks, restore_terminal, Tui};
use crate::OutputWriter;
use bsnext_dto::{ExternalEvents, StartupEvent};
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::{SendError, Sender};
use std::thread::JoinHandle;
use std::{
    io::{self},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::KeyEventKind;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Masked, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};

pub struct Ratatui(App);

pub struct RatatuiSender(Sender<RatatuiEvent>);

impl OutputWriter for RatatuiSender {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: ExternalEvents,
    ) -> anyhow::Result<()> {
        match self.0.send(RatatuiEvent::Ext(evt)) {
            Ok(_) => tracing::info!("sent..."),
            Err(_) => tracing::error!("could not send"),
        }
        Ok(())
    }

    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        dbg!(&evt);
        Ok(())
    }
}
impl OutputWriter for Ratatui {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: ExternalEvents,
    ) -> anyhow::Result<()> {
        // write!(sink, "{}", serde_json::to_string(&evt)?).map_err(|e| anyhow::anyhow!(e.to_string()))
        dbg!(&evt);
        Ok(())
    }

    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        dbg!(&evt);
        Ok(())
    }
}

impl Ratatui {
    pub fn try_new() -> anyhow::Result<Self> {
        let app = App::new();
        Ok(Ratatui(app))
    }

    pub fn install(mut self) -> anyhow::Result<(RatatuiSender, JoinHandle<()>, JoinHandle<()>)> {
        tracing::info!("installing ratatui hooks");
        install_hooks()?;
        let mut terminal = init_terminal()?;
        tracing::info!("init... terminal");
        let mut app = self.0;
        let (tx, rx) = mpsc::channel();
        let sender = RatatuiSender(tx.clone());
        Ok((
            sender,
            thread::spawn(move || {
                tracing::info!("on new thread... terminal");
                app.run(&mut terminal, rx).expect("running");
                tracing::info!("tui all done");
                restore_terminal().expect("restore");
                tracing::info!("terminal restored");
            }),
            input_handling(tx.clone()),
        ))
    }
}

fn input_handling(tx: mpsc::Sender<RatatuiEvent>) -> JoinHandle<()> {
    let tick_rate = Duration::from_millis(500);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout).is_ok_and(|r| r == true) {
                let evt = match crossterm::event::read() {
                    Ok(evt) => match evt {
                        Event::FocusGained => None,
                        Event::FocusLost => None,
                        Event::Key(key) => Some(RatatuiEvent::Input(key)),
                        Event::Mouse(_) => None,
                        Event::Paste(_) => None,
                        Event::Resize(_, _) => Some(RatatuiEvent::Resize),
                    },
                    Err(_) => None,
                };
                if let Some(evt) = evt {
                    match tx.send(evt) {
                        Ok(_) => {}
                        Err(_) => tracing::error!("couldn't send"),
                    };
                }
            }
            // if last_tick.elapsed() >= tick_rate {
            //     match tx.send(RatatuiEvent::Tick) {
            //         Ok(_) => {}
            //         Err(_) => tracing::error!("couldn't send"),
            //     };
            //     last_tick = Instant::now();
            // }
        }
    })
}

#[derive(Debug)]
struct App {
    should_exit: bool,
    scroll: u16,
    last_tick: Instant,
    events: Vec<ExternalEvents>,
}

enum RatatuiEvent {
    Input(crossterm::event::KeyEvent),
    Tick,
    Resize,
    Ext(ExternalEvents),
}

impl App {
    /// The duration between each tick.
    const TICK_RATE: Duration = Duration::from_millis(250);

    /// Create a new instance of the app.
    fn new() -> Self {
        Self {
            should_exit: false,
            scroll: 0,
            last_tick: Instant::now(),
            events: vec![],
        }
    }

    /// Run the app until the user exits.
    fn run(&mut self, terminal: &mut Tui, rx: mpsc::Receiver<RatatuiEvent>) -> anyhow::Result<()> {
        let mut redraw = true;
        loop {
            if self.should_exit {
                break;
            }
            if redraw {
                // terminal.draw(|f| self.)?;
                tracing::info!("drawing...");
                self.draw(terminal)?;
            }
            redraw = true;

            match rx.recv()? {
                RatatuiEvent::Input(event) => {
                    if event.code == crossterm::event::KeyCode::Char('q') {
                        break;
                    }
                }
                RatatuiEvent::Resize => {
                    terminal.autoresize()?;
                }
                RatatuiEvent::Tick => {
                    tracing::info!("did tick...");
                    if self.last_tick.elapsed() >= Self::TICK_RATE {
                        self.on_tick();
                        self.last_tick = Instant::now();
                    }
                }
                RatatuiEvent::Ext(ext_event) => {
                    self.events.push(ext_event);
                }
            }
        }

        // let mut redraw = true;
        // loop {
        //     if redraw {
        //         terminal.draw(|f| ui(f, &downloads))?;
        //     }
        //     redraw = true;
        // }
        // while !self.should_exit {
        //     self.draw(terminal)?;
        //     // self.handle_events()?;
        //     if self.last_tick.elapsed() >= Self::TICK_RATE {
        //         self.on_tick();
        //         self.last_tick = Instant::now();
        //     }
        // }
        if self.last_tick.elapsed() >= Self::TICK_RATE {
            self.last_tick = Instant::now();
        }
        Ok(())
    }

    /// Draw the app to the terminal.
    fn draw(&mut self, terminal: &mut Tui) -> io::Result<()> {
        terminal.draw(|frame| frame.render_widget(self, frame.size()))?;
        Ok(())
    }

    /// Handle events from the terminal.
    fn handle_events(sender: Sender<RatatuiEvent>) -> io::Result<()> {
        // let timeout = Self::TICK_RATE.saturating_sub(self.last_tick.elapsed());
        while event::poll(Duration::from_millis(500))? {
            match event::read()? {
                Event::FocusGained => tracing::info!("TUI--> FocusGained"),
                Event::FocusLost => tracing::info!("TUI--> FocusLost"),
                Event::Key(ke) => tracing::info!("TUI--> Key"),
                Event::Mouse(_) => tracing::info!("TUI--> Mouse"),
                Event::Paste(_) => tracing::info!("TUI--> Paste"),
                Event::Resize(c, r) => {
                    tracing::info!("TUI--> Resize {} {}", c, r);
                    match sender.send(crate::ratatui::RatatuiEvent::Resize) {
                        Ok(_) => {}
                        Err(_) => tracing::error!("TUI--> Resize {} {}", c, r),
                    }
                }
            }
            // if let  = event::read()? {
            //     if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
            //         self.should_exit = true;
            //     }
            // }
        }
        tracing::info!("event::poll complete");
        Ok(())
    }

    /// Update the app state on each tick.
    fn on_tick(&mut self) {
        self.scroll = (self.scroll + 1) % 10;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        Paragraph::new(create_servers(area))
            .block(title_block("Servers"))
            .gray()
            .render(areas[0], buf);
        Paragraph::new(create_events(area))
            .block(title_block(
                format!("Events ({})", self.events.len()).as_str(),
            ))
            .gray()
            .wrap(Wrap { trim: true })
            .render(areas[1], buf);
    }
}

/// Create a bordered block with a title.
fn title_block(title: &str) -> Block {
    Block::bordered()
        .gray()
        .title(title.bold().into_centered_line())
}

/// Create some lines to display in the paragraph.
fn create_servers(area: Rect) -> Vec<Line<'static>> {
    vec![
        Line::raw("http://localhost:3001"),
        Line::raw("http://localhost:3002"),
    ]
}
/// Create some lines to display in the paragraph.
fn create_events(area: Rect) -> Vec<Line<'static>> {
    vec![Line::raw("watching ./build")]
}

/// A module for common functionality used in the examples.
mod common {
    use std::{
        io::{self, stdout, Stdout},
        panic,
    };

    use color_eyre::{
        config::{EyreHook, HookBuilder, PanicHook},
        eyre,
    };
    use crossterm::ExecutableCommand;
    use ratatui::{
        backend::CrosstermBackend,
        crossterm::terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
        terminal::Terminal,
    };

    // A simple alias for the terminal type used in this example.
    pub type Tui = Terminal<CrosstermBackend<Stdout>>;

    /// Initialize the terminal and enter alternate screen mode.
    pub fn init_terminal() -> io::Result<Tui> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout());
        Terminal::new(backend)
    }

    /// Restore the terminal to its original state.
    pub fn restore_terminal() -> io::Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    /// Installs hooks for panic and error handling.
    ///
    /// Makes the app resilient to panics and errors by restoring the terminal before printing the
    /// panic or error message. This prevents error messages from being messed up by the terminal
    /// state.
    pub fn install_hooks() -> anyhow::Result<()> {
        let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();
        install_panic_hook(panic_hook);
        install_error_hook(eyre_hook)?;
        Ok(())
    }

    /// Install a panic hook that restores the terminal before printing the panic.
    fn install_panic_hook(panic_hook: PanicHook) {
        let panic_hook = panic_hook.into_panic_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let _ = restore_terminal();
            panic_hook(panic_info);
        }));
    }

    /// Install an error hook that restores the terminal before printing the error.
    fn install_error_hook(eyre_hook: EyreHook) -> anyhow::Result<()> {
        let eyre_hook = eyre_hook.into_eyre_hook();
        eyre::set_hook(Box::new(move |error| {
            let _ = restore_terminal();
            eyre_hook(error)
        }))?;
        Ok(())
    }
}
