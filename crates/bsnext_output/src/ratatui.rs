use self::common::{init_terminal, install_hooks, restore_terminal, Tui};
use crate::OutputWriter;
use bsnext_dto::{ExternalEventsDTO, GetServersMessageResponseDTO};
use std::io::{BufWriter, Write};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::{
    io::{self},
    thread,
    time::{Duration, Instant},
};

use crate::pretty::{print_server_updates, server_display, PrettyPrint};
use bsnext_dto::internal::{AnyEvent, InternalEvents, StartupEvent};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};

pub struct Ratatui(App);

pub struct RatatuiSender(Sender<RatatuiEvent>);

impl OutputWriter for RatatuiSender {
    fn handle_external_event<W: Write>(
        &self,
        _sink: &mut W,
        evt: &ExternalEventsDTO,
    ) -> anyhow::Result<()> {
        match self
            .0
            .send(RatatuiEvent::Evt(AnyEvent::External(evt.clone())))
        {
            Ok(_) => tracing::info!("sent..."),
            Err(_) => tracing::error!("could not send"),
        }
        Ok(())
    }
    fn handle_internal_event<W: Write>(
        &self,
        _sink: &mut W,
        evt: InternalEvents,
    ) -> anyhow::Result<()> {
        match self.0.send(RatatuiEvent::Evt(AnyEvent::Internal(evt))) {
            Ok(_) => tracing::info!("sent..."),
            Err(_) => tracing::error!("could not send"),
        }
        Ok(())
    }
}
impl OutputWriter for Ratatui {
    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        PrettyPrint.handle_startup_event(sink, evt)
    }
}

impl Ratatui {
    pub fn try_new() -> anyhow::Result<Self> {
        let app = App::new();
        Ok(Ratatui(app))
    }

    pub fn install(self) -> anyhow::Result<(RatatuiSender, JoinHandle<()>, JoinHandle<()>)> {
        tracing::debug!("TUI: installing ratatui hooks");
        install_hooks()?;
        let mut terminal = init_terminal()?;
        tracing::debug!("TUI: init... terminal");
        let mut app = self.0;
        let (tx, rx) = mpsc::channel();
        let sender = RatatuiSender(tx.clone());
        Ok((
            sender,
            thread::spawn(move || {
                tracing::debug!("TUI: on new thread... terminal");
                app.run(&mut terminal, rx).expect("running");
                tracing::debug!("TUI: tui all done");
                restore_terminal().expect("restore");
                tracing::debug!("TUI: terminal restored");
            }),
            input_handling(tx.clone()),
        ))
    }
}

fn input_handling(tx: mpsc::Sender<RatatuiEvent>) -> JoinHandle<()> {
    let tick_rate = Duration::from_millis(500);
    thread::spawn(move || {
        let last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout).is_ok_and(|r| r) {
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
    events: FixedSizeQueue,
    server_status: Option<GetServersMessageResponseDTO>,
}

enum RatatuiEvent {
    Input(crossterm::event::KeyEvent),
    #[allow(dead_code)]
    Tick,
    Resize,
    Evt(AnyEvent),
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
            events: FixedSizeQueue::new(10),
            server_status: None,
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
                RatatuiEvent::Evt(AnyEvent::Internal(evt)) => {
                    match &evt {
                        InternalEvents::ServersChanged { server_resp, .. } => {
                            self.server_status = Some(server_resp.clone());
                        }
                        InternalEvents::InputError(_) => {
                            todo!("InternalEvents::InputError")
                        }
                        InternalEvents::StartupError(_) => {
                            todo!("InternalEvents::StartupError")
                        }
                    }
                    self.events.add(RecordedEvent::new(AnyEvent::Internal(evt)));
                }
                RatatuiEvent::Evt(ext_event) => {
                    self.events.add(RecordedEvent::new(ext_event));
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

    fn on_tick(&mut self) {
        self.scroll = (self.scroll + 1) % 10;
    }

    /// Create some lines to display in the paragraph.
    fn create_servers(&self) -> Vec<Line<'static>> {
        self.server_status
            .as_ref()
            .map(|server_resp| {
                server_resp
                    .servers
                    .iter()
                    .map(|s| Line::raw(server_display(&s.identity, &s.socket_addr)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Create some lines to display in the paragraph.
    fn create_events(&mut self) -> Vec<Line<'static>> {
        self.events
            .get()
            .iter()
            .map(|evt| match &evt.evt {
                AnyEvent::Internal(int) => match int {
                    InternalEvents::ServersChanged { child_results, .. } => {
                        (evt.now, print_server_updates(child_results))
                    }
                    InternalEvents::InputError(input_error) => {
                        todo!("InternalEvents::InputError")
                    }
                    InternalEvents::StartupError(_) => {
                        todo!("InternalEvents::StartupError")
                    }
                },
                AnyEvent::External(ext) => {
                    let mut writer = BufWriter::new(Vec::new());
                    PrettyPrint
                        .handle_external_event(&mut writer, ext)
                        .expect("can write");
                    (
                        evt.now,
                        vec![String::from_utf8(writer.into_inner().expect("into_inner"))
                            .expect("as_utf8")],
                    )
                }
            })
            .flat_map(|(dt, strs)| {
                strs.iter()
                    .map(|str| Line::raw(format!("{} {str}", dt.format("%-I.%M%P"))))
                    .collect::<Vec<_>>()
            })
            .collect()
        // .flatten()
        // .collect()
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);
        Paragraph::new(self.create_servers())
            .block(title_block("Servers"))
            .gray()
            .render(areas[0], buf);
        Paragraph::new(self.create_events())
            .block(title_block(
                format!("Events ({})", self.events.get().len()).as_str(),
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

use std::collections::VecDeque;

#[derive(Debug)]
struct RecordedEvent {
    evt: AnyEvent,
    now: chrono::DateTime<chrono::Local>,
}

impl RecordedEvent {
    pub fn new(evt: AnyEvent) -> Self {
        Self {
            evt,
            now: chrono::Local::now(),
        }
    }
}

#[derive(Debug, Default)]
struct FixedSizeQueue {
    deque: VecDeque<RecordedEvent>,
    capacity: usize,
}

impl FixedSizeQueue {
    fn new(capacity: usize) -> Self {
        FixedSizeQueue {
            deque: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn add(&mut self, value: RecordedEvent) {
        if self.deque.len() == self.capacity {
            self.deque.pop_back();
        }
        self.deque.push_front(value);
    }

    fn get(&self) -> &VecDeque<RecordedEvent> {
        &self.deque
    }
}

// fn main() {
// let mut queue = FixedSizeQueue::new(200);
//
// for i in 1..=201 {
//     queue.add(format!("Element {}", i));
// }
//
// for elem in queue.get() {
//     println!("{}", elem);
// }
// }
