extern crate libc;
extern crate pty;
extern crate regex;
extern crate ws;
#[macro_use]
extern crate lazy_static;
extern crate qstring;

mod terminal;
mod variables;

use pty::fork::*;
use regex::{Captures, Regex, RegexBuilder};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::{mpsc, Mutex};
use std::{env, fs, process, thread, time};

fn apply_template(data: &str, variables: &HashMap<String, String>) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"%([\w:]+)%").unwrap();
    }
    String::from(RE.replace_all(data, |captures: &Captures| {
        let parts: Vec<_> = captures[1].split(":").collect();
        // let escape_type = parts[0];
        let key = parts.last().unwrap();
        if let Some(value) = variables.get(*key) {
            // TODO: escape
            value.to_string()
        } else {
            eprintln!("Failed to resolve variable: {:?}", key);
            format!("%?{}%", key)
        }
    }))
}

fn decode_2b(data: &str) -> u32 {
    let data: Vec<_> = data.bytes().collect();
    (data[0] as u32 - 1) + (data[1] as u32 - 1) * 127
}

struct ServerState {
    clients: HashMap<u64, Arc<ws::Sender>>,
    new_clients: Vec<u64>,
    vars: HashMap<String, String>,
    id_counter: u64,
    prev_width: u32,
    prev_height: u32,
    prev_attrs: u32,
    prev_static_opts: String,
    prev_state_id: u32,
    prev_bell_id: u32,
    prev_title: String,
    prev_cursor: String,
    prev_line_sizes: String,
}

struct ConnHandler {
    id: u64,
    out: Arc<ws::Sender>,
    state: Arc<Mutex<ServerState>>,
    shell_in: mpsc::Sender<Vec<u8>>,
}

impl ConnHandler {
    fn not_found() -> ws::Response {
        ws::Response::new(404, "Not Found", b"not found".to_vec())
    }

    fn server_error() -> ws::Response {
        ws::Response::new(500, "Internal Server Error", b"error".to_vec())
    }

    fn template(path: &Path, vars: &HashMap<String, String>) -> ws::Response {
        match fs::read(path) {
            Ok(bytes) => {
                let mut contents = String::from(String::from_utf8_lossy(&bytes));
                contents = apply_template(&contents, vars);
                let mut res = ws::Response::new(200, "OK", contents.bytes().collect::<Vec<_>>());
                res.headers_mut()
                    .push(("Content-Type".into(), b"text/html; charset=utf-8".to_vec()));
                res
            }
            Err(_) => Self::server_error(),
        }
    }

    fn add_headers(res: &mut ws::Response, file_path: &Path) {
        if let Some(ext) = file_path.extension() {
            if let Some(ext) = ext.to_str() {
                match ext {
                    "html" => res
                        .headers_mut()
                        .push(("Content-Type".into(), b"text/html; charset=utf-8".to_vec())),
                    "css" => res
                        .headers_mut()
                        .push(("Content-Type".into(), b"text/css; charset=utf-8".to_vec())),
                    "js" => res.headers_mut().push((
                        "Content-Type".into(),
                        b"application/javascript; charset=utf-8".to_vec(),
                    )),
                    "svg" => res.headers_mut().push((
                        "Content-Type".into(),
                        b"image/svg+xml; charset=utf-8".to_vec(),
                    )),
                    _ => (),
                }
            }
        }
    }
}

impl ws::Handler for ConnHandler {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
        lazy_static! {
            static ref CFG_SET_RE: Regex = Regex::new(r"^(/cfg/\w+)/set(.*)").unwrap();
        }

        let mut state = self.state.lock().unwrap();

        match req.resource() {
            path if CFG_SET_RE.is_match(path) => {
                let captures = CFG_SET_RE.captures(path).unwrap();

                for (key, value) in qstring::QString::from(&captures[2]).into_iter() {
                    if state.vars.contains_key(&key) {
                        state.vars.insert(key, value);
                    }
                }

                let mut res = ws::Response::new(301, "Found", Vec::new());
                res.headers_mut()
                    .push(("Location".into(), captures[1].bytes().collect()));
                Ok(res)
            }
            "/term/update.ws" => ws::Response::from_request(req),
            path if path.starts_with("/js/")
                || path.starts_with("/img/")
                || path.starts_with("/css/")
                || path == "/favicon.ico" =>
            {
                // static
                let file_path = PathBuf::from("web".to_string() + path);
                match fs::read(&file_path) {
                    Ok(contents) => {
                        let mut res = ws::Response::new(200, "OK", contents);
                        Self::add_headers(&mut res, &file_path);
                        Ok(res)
                    }
                    Err(_) => Ok(Self::not_found()),
                }
            }
            "/" => Ok(Self::template(&PathBuf::from("web/term.tpl"), &state.vars)),
            "/cfg/term" => Ok(Self::template(
                &PathBuf::from("web/cfg_term.tpl"),
                &state.vars,
            )),
            "/cfg/network" => Ok(Self::template(
                &PathBuf::from("web/cfg_network.tpl"),
                &state.vars,
            )),
            "/cfg/system" => Ok(Self::template(
                &PathBuf::from("web/cfg_system.tpl"),
                &state.vars,
            )),
            "/cfg/wifi" => Ok(Self::template(
                &PathBuf::from("web/cfg_wifi.tpl"),
                &state.vars,
            )),
            "/help" => Ok(Self::template(&PathBuf::from("web/help.html"), &state.vars)),
            "/about" => Ok(Self::template(&PathBuf::from("web/about.tpl"), &state.vars)),
            path if path.starts_with("/cfg/wifi/scan") => Ok(ws::Response::new(
                200,
                "OK",
                b"{
                    \"result\": {
                      \"inProgress\": 0,
                      \"APs\": [
                        {
                          \"essid\": \"horse\",
                          \"rssi_perc\": 100,
                          \"enc\": 0
                        }
                      ]
                    }
                }"
                    .to_vec(),
            )),
            path if path.starts_with("/api/v1/ping") => {
                Ok(ws::Response::new(200, "OK", b"pong".to_vec()))
            }
            // TODO: config pages
            _ => Ok(Self::not_found()),
        }
    }

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        let mut state = self.state.lock().unwrap();
        println!("+ connection from {:?}", shake.peer_addr);
        state.clients.insert(self.id, Arc::clone(&self.out));
        state.new_clients.push(self.id);

        Ok(())
    }

    fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
        if let ws::Message::Text(message) = message {
            if message.len() < 2 {
                return Ok(());
            }
            let msg_type = message.chars().next().unwrap();
            let data = &message[1..];

            match msg_type {
                's' => {
                    // string input
                    self.shell_in
                        .send(data.bytes().collect::<Vec<_>>())
                        .unwrap();
                }
                'b' => unimplemented!("Button presses"),
                'm' | 'p' | 'r' => {
                    let row = decode_2b(&data[0..2]);
                    let col = decode_2b(&data[2..4]);
                    let button = decode_2b(&data[4..6]);
                    let modifiers = decode_2b(&data[6..8]);

                    let ctrl = modifiers & 1 != 0;
                    let shift = modifiers & 2 != 0;
                    let opt = modifiers & 4 != 0;
                    let meta = modifiers & 8 != 0;

                    // xterm only for now
                    let x = col + 1;
                    let y = row + 1;
                    let mut event_code = if msg_type == 'r' {
                        3
                    } else {
                        match button {
                            0 => 3,
                            1 => 0,
                            2 => 1,
                            3 => 2,
                            4 => 64,
                            5 => 65,
                            _ => 0,
                        }
                    };
                    if shift {
                        event_code |= 4
                    }
                    if opt || meta {
                        event_code |= 8
                    }
                    if ctrl {
                        event_code |= 16
                    }

                    let mut msg = String::from("'\x1b[M");
                    unsafe {
                        msg.push(std::char::from_u32_unchecked(32 + event_code));
                        msg.push(std::char::from_u32_unchecked(32 + x));
                        msg.push(std::char::from_u32_unchecked(32 + y));
                    }
                    self.shell_in.send(msg.bytes().collect::<Vec<_>>()).unwrap();
                }
                _ => {
                    eprintln!("Unhandled message type {:?}", msg_type);
                }
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _: ws::CloseCode, _: &str) {
        let mut state = self.state.lock().unwrap();
        println!("− connection");
        state.clients.remove(&self.id);
    }
}

const TERM_WIDTH: u32 = 100;
const TERM_HEIGHT: u32 = 36;

fn main() {
    let fork = Fork::from_ptmx().unwrap();

    if let Some(mut master) = fork.is_parent().ok() {
        let slave_fd;
        unsafe {
            let fd = master.as_raw_fd();
            let flags = libc::fcntl(fd, libc::F_GETFL as i32, 0);
            if flags == -1 {
                panic!("PTY: wrong fd?");
            }
            libc::fcntl(fd, libc::F_SETFL as i32, flags | libc::O_NONBLOCK);
            master.grantpt().unwrap();
            master.unlockpt().unwrap();
            let slave_name = master.ptsname().unwrap();
            slave_fd = libc::open(slave_name, libc::O_RDWR | libc::O_NOCTTY);
            let win_size = libc::winsize {
                ws_col: TERM_WIDTH as u16,
                ws_row: TERM_HEIGHT as u16,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            libc::ioctl(slave_fd, libc::TIOCSWINSZ, &win_size);
        }

        let (shell_in, shell_recv) = mpsc::channel();
        let state = Arc::new(Mutex::new(ServerState {
            clients: HashMap::new(),
            new_clients: Vec::new(),
            vars: variables::defaults(),
            id_counter: 0,
            prev_width: 0,
            prev_height: 0,
            prev_attrs: 0,
            prev_static_opts: "".into(),
            prev_state_id: 0,
            prev_bell_id: 0,
            prev_title: "".into(),
            prev_cursor: "".into(),
            prev_line_sizes: "".into(),
        }));

        let state_clone = Arc::clone(&state);
        thread::spawn(move || {
            ws::listen("127.0.0.1:3000", |out| {
                let out = Arc::new(out);
                let mut state = state_clone.lock().unwrap();
                state.id_counter += 1;
                let id = state.id_counter;
                ConnHandler {
                    id,
                    out,
                    state: Arc::clone(&state_clone),
                    shell_in: shell_in.clone(),
                }
            }).unwrap();
        });

        const TOPIC_CHANGE_SCREEN_OPTS: u8 = 1;
        const TOPIC_CHANGE_CONTENT_ALL: u8 = 1 << 1;
        const TOPIC_CHANGE_CONTENT_PART: u8 = 1 << 2;
        const TOPIC_CHANGE_TITLE: u8 = 1 << 3;
        const TOPIC_CHANGE_BUTTONS: u8 = 1 << 4;
        const TOPIC_CHANGE_CURSOR: u8 = 1 << 5;
        const TOPIC_INTERNAL: u8 = 1 << 6;
        const TOPIC_BELL: u8 = 1 << 7;

        let mut terminal = terminal::Terminal::new(TERM_WIDTH, TERM_HEIGHT);
        let mut buf = [0; 4096];
        let mut heartbeat_time = time::Instant::now();
        let start_time = time::Instant::now();
        loop {
            loop {
                if let Ok(data) = shell_recv.try_recv() {
                    master.write(&data).unwrap();
                } else {
                    break;
                }
            }

            let mut data = Vec::with_capacity(4096);
            loop {
                let bytes_read = master.read(&mut buf).unwrap();
                if bytes_read > 0 {
                    data.extend_from_slice(&buf[0..bytes_read]);
                } else {
                    break;
                }
            }

            if !data.is_empty() {
                // TODO: consider sending raw bytes
                let data_str = String::from_utf8_lossy(&data);
                terminal.write(&data_str);
            }

            {
                let mut state = state.lock().unwrap();
                let new_clients: Vec<_> = state.new_clients.drain(..).collect();

                if !new_clients.is_empty() {
                    // TODO: less hacky solution
                    state.prev_attrs = 0;
                    state.prev_static_opts = "".into();
                    state.prev_state_id = 0;
                    state.prev_bell_id = 0;
                    state.prev_title = "".into();
                    state.prev_cursor = "".into();
                    state.prev_line_sizes = "".into();
                }

                let update_debug = if heartbeat_time.elapsed().as_secs() > 1 {
                    for (_, client) in &state.clients {
                        client.send(".").unwrap();
                    }
                    heartbeat_time = time::Instant::now();

                    true
                } else {
                    false
                };

                let attrs = terminal.attributes();
                let static_opts =
                    format!("{}{}", state.vars["font_stack"], state.vars["font_size"]);
                let state_id = terminal.state_id();
                let bell_id = terminal.bell_id();
                let title = terminal.title();
                let cursor = terminal.cursor();
                let line_sizes = terminal.line_sizes();

                let mut topic_flags = 0;
                let mut content = String::new();

                if update_debug {
                    topic_flags |= TOPIC_INTERNAL;
                    content.push('D');
                    content.push(terminal::encode_as_code_point(0)); // attrs
                    content.push(terminal::encode_as_code_point(0));
                    let scroll_margin = terminal.scroll_margin();
                    content.push(scroll_margin[0]);
                    content.push(scroll_margin[1]);
                    // charset
                    content.push(terminal::encode_as_code_point(terminal.current_code_page()));
                    content.push(terminal.get_code_page(0));
                    content.push(terminal.get_code_page(1));

                    // cursor fg/bg
                    content.push(terminal::encode_as_code_point(0));
                    content.push(terminal::encode_as_code_point(0));

                    // free memory
                    content.push(terminal::encode_as_code_point(999999));

                    // connection count
                    content.push(terminal::encode_as_code_point(state.clients.len() as u32));
                }

                let size_changed = terminal.width != state.prev_width || terminal.height != state.prev_height;

                if attrs != state.prev_attrs || size_changed {
                    state.prev_attrs = attrs;

                    if size_changed {
                        unsafe {
                            let win_size = libc::winsize {
                                ws_col: terminal.width as u16,
                                ws_row: terminal.height as u16,
                                ws_xpixel: 0,
                                ws_ypixel: 0,
                            };
                            libc::ioctl(slave_fd, libc::TIOCSWINSZ, &win_size);
                        }
                        state.prev_width = terminal.width;
                        state.prev_height = terminal.height;
                    }

                    topic_flags |= TOPIC_CHANGE_SCREEN_OPTS;
                    content.push('O');
                    content.push(terminal::encode_as_code_point(terminal.height));
                    content.push(terminal::encode_as_code_point(terminal.width));
                    content.push(terminal::encode_as_code_point(
                        state.vars["theme"].parse().unwrap_or(0),
                    ));

                    lazy_static! {
                        static ref HEX_COLOR_RE: Regex = RegexBuilder::new(r"^#[\da-f]{6}$")
                            .case_insensitive(true)
                            .build()
                            .unwrap();
                    }

                    let default_fg = &state.vars["default_fg"];
                    let default_bg = &state.vars["default_bg"];
                    content += &if HEX_COLOR_RE.is_match(default_fg) {
                        terminal::encode_24color(
                            u32::from_str_radix(&default_fg[1..], 16).unwrap_or(0) + 256,
                        )
                    } else {
                        terminal::encode_24color(default_fg.parse().unwrap_or(7))
                    };
                    content += &if HEX_COLOR_RE.is_match(default_bg) {
                        terminal::encode_24color(
                            u32::from_str_radix(&default_bg[1..], 16).unwrap_or(0) + 256,
                        )
                    } else {
                        terminal::encode_24color(default_bg.parse().unwrap_or(0))
                    };
                    content.push(terminal::encode_as_code_point(attrs));
                }

                if static_opts != state.prev_static_opts {
                    state.prev_static_opts = static_opts;

                    content.push('P');
                    content += &state.vars["font_stack"];
                    content.push('\x01');
                    content.push(terminal::encode_as_code_point(
                        state.vars["font_size"].parse().unwrap_or(0),
                    ));
                }

                if title != state.prev_title {
                    topic_flags |= TOPIC_CHANGE_TITLE;
                    content.push('T');
                    content += &title;
                    content.push('\x01');
                    state.prev_title = title;
                }
                // TODO: buttons??

                if bell_id != state.prev_bell_id {
                    state.prev_bell_id = bell_id;
                    topic_flags |= TOPIC_BELL;
                    content.push('!');
                }

                if cursor != state.prev_cursor {
                    topic_flags |= TOPIC_CHANGE_CURSOR;
                    content.push('C');
                    content += &cursor;
                    state.prev_cursor = cursor;
                }

                if line_sizes != state.prev_line_sizes {
                    // no topic flag for this one it seems, so fake one
                    topic_flags |= TOPIC_CHANGE_CONTENT_PART;
                    content += &line_sizes;
                    state.prev_line_sizes = line_sizes;
                }

                if state_id != state.prev_state_id || terminal.is_rainbow() {
                    let elapsed = start_time.elapsed();
                    let t =
                        elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.;
                    let screen = terminal.serialize_screen(t, !new_clients.is_empty());
                    if !screen.is_empty() {
                        topic_flags |= TOPIC_CHANGE_CONTENT_PART;
                        content += &screen;
                    }
                }

                content.insert(0, terminal::encode_as_code_point(topic_flags.into()));
                content.insert(0, 'U');

                if !state.clients.is_empty() && topic_flags != 0 {
                    state
                        .clients
                        .values()
                        .next()
                        .unwrap()
                        .broadcast(content)
                        .unwrap();
                }
            }
            thread::sleep(time::Duration::new(0, 16_666_667));
        }
    } else {
        // pty child thread

        // periodically check if parent is dead
        thread::spawn(|| loop {
            if unsafe { libc::getppid() } == 1 {
                process::exit(0);
            }
            thread::sleep(time::Duration::new(1, 0));
        });

        let home = env::var("HOME").unwrap();
        let shell = env::var("SHELL").unwrap();
        let path = if cfg!(target_os = "macos") {
            String::from("/usr/bin:/bin:/usr/sbin:/sbin")
        } else {
            // super inconsistent on linux, just take the env value
            env::var("PATH").unwrap()
        };
        let tmpdir = env::var("TMPDIR").unwrap();
        let user = env::var("USER").unwrap();

        loop {
            let status = Command::new(&shell)
                .arg("--login")
                .env_clear()
                .env("TERM", "xterm-256color")
                .env("LANG", "en_US.UTF-8")
                .env("HOME", &home)
                .env("TERM_PROGRAM", "ESPTerm Emulator")
                .env("TMPDIR", &tmpdir)
                .env("PATH", &path)
                .env("USER", &user)
                .current_dir(&home)
                .status()
                .expect("Failed to start shell");

            println!("\x1b[0;41m\x1b[2K\x1b[GExited ({})\x1b[0m", status);
            print!("Press return to restart");
            std::io::stdout().flush().unwrap();
            let mut stdin = std::io::stdin();
            stdin.lock().bytes().next();
        }
    }
}
