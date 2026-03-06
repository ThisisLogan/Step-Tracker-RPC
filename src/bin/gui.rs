use anyhow::{anyhow, Context as _, Result};
use directories::ProjectDirs;
use eframe::egui;
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};

#[derive(Clone, Debug)]
struct EnvConfig {
    api_url: String,
    api_token: String,

    enable_steps: bool,
    steps_discord_client_id: String,
    steps_large_image_key: String,

    enable_water: bool,
    water_discord_client_id: String,
    water_large_image_key: String,

    enable_sleep: bool,
    sleep_discord_client_id: String,
    sleep_large_image_key: String,

    obs_steps_file: String,
    obs_water_file: String,
    obs_sleep_file: String,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            api_token: String::new(),

            enable_steps: true,
            steps_discord_client_id: String::new(),
            steps_large_image_key: String::new(),

            enable_water: true,
            water_discord_client_id: String::new(),
            water_large_image_key: String::new(),

            enable_sleep: true,
            sleep_discord_client_id: String::new(),
            sleep_large_image_key: String::new(),

            obs_steps_file: String::new(),
            obs_water_file: String::new(),
            obs_sleep_file: String::new(),
        }
    }
}

impl EnvConfig {
    fn from_kv(map: &BTreeMap<String, String>) -> Self {
        let mut cfg = Self::default();

        cfg.api_url = map
            .get("API_URL")
            .cloned()
            .unwrap_or("https://steps.mayb.gay".to_string());
        cfg.api_token = map.get("API_TOKEN").cloned().unwrap_or_default();

        if let Some(v) = parse_bool(map.get("ENABLE_STEPS")) {
            cfg.enable_steps = v;
        }
        if let Some(v) = parse_bool(map.get("ENABLE_WATER")) {
            cfg.enable_water = v;
        }
        if let Some(v) = parse_bool(map.get("ENABLE_SLEEP")) {
            cfg.enable_sleep = v;
        }

        cfg.steps_discord_client_id = map
            .get("STEPS_DISCORD_CLIENT_ID")
            .cloned()
            .unwrap_or("1428159322432471223".to_string());
        cfg.steps_large_image_key = map
            .get("STEPS_DISCORD_LARGE_IMAGE_KEY")
            .cloned()
            .unwrap_or("man_walking_emoji_copy".to_string());

        cfg.water_discord_client_id = map
            .get("WATER_DISCORD_CLIENT_ID")
            .cloned()
            .unwrap_or("1450261879120199683".to_string());
        cfg.water_large_image_key = map
            .get("WATER_DISCORD_LARGE_IMAGE_KEY")
            .cloned()
            .unwrap_or("emoji_man_drinking".to_string());

        cfg.sleep_discord_client_id = map
            .get("SLEEP_DISCORD_CLIENT_ID")
            .cloned()
            .unwrap_or("1462670949043277829".to_string());
        cfg.sleep_large_image_key = map
            .get("SLEEP_DISCORD_LARGE_IMAGE_KEY")
            .cloned()
            .unwrap_or("sleeper".to_string());

        cfg.obs_steps_file = map.get("OBS_STEPS_FILE").cloned().unwrap_or_default();
        cfg.obs_water_file = map.get("OBS_WATER_FILE").cloned().unwrap_or_default();
        cfg.obs_sleep_file = map.get("OBS_SLEEP_FILE").cloned().unwrap_or_default();

        cfg
    }

    fn to_env_map(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::<String, String>::new();

        map.insert("API_URL".into(), self.api_url.clone());
        map.insert("API_TOKEN".into(), self.api_token.clone());

        map.insert("ENABLE_STEPS".into(), bool_to_str(self.enable_steps).into());
        map.insert("ENABLE_WATER".into(), bool_to_str(self.enable_water).into());
        map.insert("ENABLE_SLEEP".into(), bool_to_str(self.enable_sleep).into());

        map.insert(
            "STEPS_DISCORD_CLIENT_ID".into(),
            self.steps_discord_client_id.clone(),
        );
        map.insert(
            "STEPS_DISCORD_LARGE_IMAGE_KEY".into(),
            self.steps_large_image_key.clone(),
        );

        map.insert(
            "WATER_DISCORD_CLIENT_ID".into(),
            self.water_discord_client_id.clone(),
        );
        map.insert(
            "WATER_DISCORD_LARGE_IMAGE_KEY".into(),
            self.water_large_image_key.clone(),
        );

        map.insert(
            "SLEEP_DISCORD_CLIENT_ID".into(),
            self.sleep_discord_client_id.clone(),
        );
        map.insert(
            "SLEEP_DISCORD_LARGE_IMAGE_KEY".into(),
            self.sleep_large_image_key.clone(),
        );

        if !self.obs_steps_file.trim().is_empty() {
            map.insert("OBS_STEPS_FILE".into(), self.obs_steps_file.clone());
        }
        if !self.obs_water_file.trim().is_empty() {
            map.insert("OBS_WATER_FILE".into(), self.obs_water_file.clone());
        }
        if !self.obs_sleep_file.trim().is_empty() {
            map.insert("OBS_SLEEP_FILE".into(), self.obs_sleep_file.clone());
        }

        map
    }
}

fn parse_bool(v: Option<&String>) -> Option<bool> {
    parse_bool_raw(v?.trim())
}

fn parse_bool_raw(v: &str) -> Option<bool> {
    let s = v.trim().to_ascii_lowercase();
    match s.as_str() {
        "1" | "true" | "yes" | "y" | "on" => Some(true),
        "0" | "false" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

fn bool_to_str(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

fn tray_disabled_by_env() -> bool {
    match std::env::var("GUI_DISABLE_TRAY") {
        Ok(v) => parse_bool_raw(&v).unwrap_or(false),
        Err(_) => false,
    }
}

fn env_file_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ThisisLogan", "StepTrackerRPC") {
        return proj_dirs.config_dir().join(".env");
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".env")
}

fn read_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", path.display())),
    };

    let mut map = BTreeMap::<String, String>::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim().to_string();
        let mut val = v.trim().to_string();
        if (val.starts_with('"') && val.ends_with('"'))
            || (val.starts_with('\'') && val.ends_with('\''))
        {
            val = val[1..val.len().saturating_sub(1)].to_string();
        }
        if !key.is_empty() {
            map.insert(key, val);
        }
    }
    Ok(map)
}

fn write_env_file(path: &Path, map: &BTreeMap<String, String>) -> Result<()> {
    let mut out = String::new();
    out.push_str("# Generated by Step Tracker RPC GUI\n");
    out.push_str("# (this file may contain secrets like API_TOKEN)\n\n");

    for (k, v) in map {
        if v.trim().is_empty() {
            continue;
        }
        out.push_str(k);
        out.push('=');
        out.push_str(&encode_env_value(v));
        out.push('\n');
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(path, out).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn encode_env_value(value: &str) -> String {
    let needs_quotes = value
        .chars()
        .any(|c| c.is_whitespace() || c == '#' || c == '"');
    if !needs_quotes {
        return value.to_string();
    }
    let mut s = String::with_capacity(value.len() + 2);
    s.push('"');
    for c in value.chars() {
        match c {
            '\\' => s.push_str("\\\\"),
            '"' => s.push_str("\\\""),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            _ => s.push(c),
        }
    }
    s.push('"');
    s
}

fn rpc_binary_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("Failed to get current exe path")?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow!("Failed to resolve exe parent directory"))?;

    let rpc_name = if cfg!(windows) { "rpc.exe" } else { "rpc" };

    let sibling = dir.join(rpc_name);
    if sibling.exists() {
        return Ok(sibling);
    }

    let cwd = std::env::current_dir().context("Failed to get current dir")?;
    let debug = cwd.join("target").join("debug").join(rpc_name);
    if debug.exists() {
        return Ok(debug);
    }
    let release = cwd.join("target").join("release").join(rpc_name);
    if release.exists() {
        return Ok(release);
    }

    Err(anyhow!(
        "Could not find `{}` next to the GUI binary or in target/(debug|release). Build it first (e.g. `cargo build --bin rpc`).",
        rpc_name
    ))
}

fn make_tray_icon_rgba_32() -> (Vec<u8>, u32, u32) {
    let w = 32u32;
    let h = 32u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            rgba[idx] = 20;
            rgba[idx + 1] = 24;
            rgba[idx + 2] = 28;
            rgba[idx + 3] = 255;

            if x > 6 && x < 26 && y > 12 && y < 20 {
                rgba[idx] = 64;
                rgba[idx + 1] = 200;
                rgba[idx + 2] = 120;
            }
        }
    }

    (rgba, w, h)
}

#[derive(Debug, Clone)]
enum LogLine {
    Stdout(String),
    Stderr(String),
    System(String),
}

#[derive(Debug, Clone)]
enum TrayState {
    Ready,
    Unavailable { reason: String },
}

#[derive(Debug, Clone, Copy)]
enum TrayCommand {
    Start,
    Stop,
    Show,
    Hide,
    Quit,
}

#[derive(Clone)]
struct TrayIds {
    start: tray_icon::menu::MenuId,
    stop: tray_icon::menu::MenuId,
    show: tray_icon::menu::MenuId,
    hide: tray_icon::menu::MenuId,
    quit: tray_icon::menu::MenuId,
}

trait TrayBackend {
    fn state(&self) -> TrayState;
    fn ensure_ready(&mut self) -> Option<String>;
    fn pump_events(&mut self) -> Vec<TrayCommand>;
}

struct RealTrayBackend {
    tray: Option<tray_icon::TrayIcon>,
    tray_ids: Option<TrayIds>,
    attempted_init: bool,
    state: TrayState,
}

impl RealTrayBackend {
    fn new() -> Self {
        Self {
            tray: None,
            tray_ids: None,
            attempted_init: false,
            state: TrayState::Unavailable {
                reason: "Tray has not initialized yet".to_string(),
            },
        }
    }
}

impl TrayBackend for RealTrayBackend {
    fn state(&self) -> TrayState {
        self.state.clone()
    }

    fn ensure_ready(&mut self) -> Option<String> {
        if self.tray.is_some() {
            self.state = TrayState::Ready;
            return None;
        }

        if self.attempted_init {
            return None;
        }
        self.attempted_init = true;

        let icon = match (|| -> Result<tray_icon::Icon> {
            let (rgba, w, h) = make_tray_icon_rgba_32();
            Ok(tray_icon::Icon::from_rgba(rgba, w, h)?)
        })() {
            Ok(icon) => icon,
            Err(e) => {
                let reason = format!("Tray icon unavailable (failed to build icon): {e}");
                self.state = TrayState::Unavailable {
                    reason: reason.clone(),
                };
                return Some(reason);
            }
        };

        let tray_menu = tray_icon::menu::Menu::new();
        let start_item = tray_icon::menu::MenuItem::new("Start RPC", true, None);
        let stop_item = tray_icon::menu::MenuItem::new("Stop RPC", true, None);
        let show_item = tray_icon::menu::MenuItem::new("Show window", true, None);
        let hide_item = tray_icon::menu::MenuItem::new("Hide window", true, None);
        let quit_item = tray_icon::menu::MenuItem::new("Quit (stops RPC)", true, None);

        let submenu = match tray_icon::menu::Submenu::with_items(
            "Step Tracker RPC",
            true,
            &[
                &start_item,
                &stop_item,
                &tray_icon::menu::PredefinedMenuItem::separator(),
                &show_item,
                &hide_item,
                &tray_icon::menu::PredefinedMenuItem::separator(),
                &quit_item,
            ],
        ) {
            Ok(s) => s,
            Err(e) => {
                let reason = format!("Tray menu unavailable: {e}");
                self.state = TrayState::Unavailable {
                    reason: reason.clone(),
                };
                return Some(reason);
            }
        };

        if let Err(e) = tray_menu.append(&submenu) {
            let reason = format!("Tray menu unavailable: {e}");
            self.state = TrayState::Unavailable {
                reason: reason.clone(),
            };
            return Some(reason);
        }

        let tray = match tray_icon::TrayIconBuilder::new()
            .with_tooltip("Step Tracker RPC")
            .with_menu(Box::new(tray_menu))
            .with_icon(icon)
            .build()
        {
            Ok(t) => t,
            Err(e) => {
                let reason = format!("Tray icon unavailable: {e}");
                self.state = TrayState::Unavailable {
                    reason: reason.clone(),
                };
                return Some(reason);
            }
        };

        self.tray = Some(tray);
        self.tray_ids = Some(TrayIds {
            start: start_item.id().clone(),
            stop: stop_item.id().clone(),
            show: show_item.id().clone(),
            hide: hide_item.id().clone(),
            quit: quit_item.id().clone(),
        });
        self.state = TrayState::Ready;
        Some("Tray icon ready".to_string())
    }

    fn pump_events(&mut self) -> Vec<TrayCommand> {
        let mut out = Vec::new();

        if let Some(ids) = self.tray_ids.clone() {
            while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                if event.id == ids.start {
                    out.push(TrayCommand::Start);
                } else if event.id == ids.stop {
                    out.push(TrayCommand::Stop);
                } else if event.id == ids.show {
                    out.push(TrayCommand::Show);
                } else if event.id == ids.hide {
                    out.push(TrayCommand::Hide);
                } else if event.id == ids.quit {
                    out.push(TrayCommand::Quit);
                }
            }
        }

        while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            match event {
                tray_icon::TrayIconEvent::DoubleClick { .. } => out.push(TrayCommand::Show),
                tray_icon::TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                } => {
                    if button == tray_icon::MouseButton::Left
                        && button_state == tray_icon::MouseButtonState::Down
                    {
                        out.push(TrayCommand::Show);
                    }
                }
                _ => {}
            }
        }

        out
    }
}

struct NoTrayBackend {
    state: TrayState,
    announced: bool,
}

impl NoTrayBackend {
    fn disabled_by_env() -> Self {
        Self {
            state: TrayState::Unavailable {
                reason: "Tray disabled by GUI_DISABLE_TRAY=true".to_string(),
            },
            announced: false,
        }
    }

    fn unavailable(reason: String) -> Self {
        Self {
            state: TrayState::Unavailable { reason },
            announced: false,
        }
    }
}

impl TrayBackend for NoTrayBackend {
    fn state(&self) -> TrayState {
        self.state.clone()
    }

    fn ensure_ready(&mut self) -> Option<String> {
        if self.announced {
            return None;
        }
        self.announced = true;

        match &self.state {
            TrayState::Unavailable { reason } => Some(format!("Tray disabled: {reason}")),
            TrayState::Ready => None,
        }
    }

    fn pump_events(&mut self) -> Vec<TrayCommand> {
        Vec::new()
    }
}

struct GuiApp {
    env: EnvConfig,
    env_path: PathBuf,

    child: Option<Child>,
    log_rx: Option<mpsc::Receiver<LogLine>>,
    logs: Vec<String>,

    tray_backend: Box<dyn TrayBackend>,

    wants_hide: bool,
    wants_show: bool,
    wants_quit: bool,
}

impl GuiApp {
    fn new() -> Self {
        let env_path = env_file_path();
        let env = read_env_file(&env_path)
            .ok()
            .map(|m| EnvConfig::from_kv(&m))
            .unwrap_or_default();

        let tray_backend: Box<dyn TrayBackend> = if tray_disabled_by_env() {
            Box::new(NoTrayBackend::disabled_by_env())
        } else {
            Box::new(RealTrayBackend::new())
        };

        Self {
            env,
            env_path,
            child: None,
            log_rx: None,
            logs: Vec::new(),
            tray_backend,
            wants_hide: false,
            wants_show: false,
            wants_quit: false,
        }
    }

    fn tray_state(&self) -> TrayState {
        self.tray_backend.state()
    }

    fn tray_ready(&self) -> bool {
        matches!(self.tray_state(), TrayState::Ready)
    }

    fn push_log(&mut self, line: LogLine) {
        let s = match line {
            LogLine::Stdout(s) => s,
            LogLine::Stderr(s) => format!("stderr: {s}"),
            LogLine::System(s) => format!("[gui] {s}"),
        };
        self.logs.push(s);
        if self.logs.len() > 500 {
            let drain = self.logs.len() - 500;
            self.logs.drain(0..drain);
        }
    }

    fn running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.push_log(LogLine::System(format!("RPC exited: {status}")));
                    self.child = None;
                    self.log_rx = None;
                    false
                }
                Ok(None) => true,
                Err(e) => {
                    self.push_log(LogLine::System(format!("Failed to poll RPC: {e}")));
                    false
                }
            }
        } else {
            false
        }
    }

    fn load_env(&mut self) {
        match read_env_file(&self.env_path) {
            Ok(map) => {
                self.env = EnvConfig::from_kv(&map);
                self.push_log(LogLine::System(format!("Loaded {}", self.env_path.display())));
            }
            Err(e) => self.push_log(LogLine::System(format!("Load failed: {e}"))),
        }
    }

    fn save_env(&mut self) {
        let map = self.env.to_env_map();
        match write_env_file(&self.env_path, &map) {
            Ok(()) => self.push_log(LogLine::System(format!("Saved {}", self.env_path.display()))),
            Err(e) => self.push_log(LogLine::System(format!("Save failed: {e}"))),
        }
    }

    fn start_rpc(&mut self) {
        if self.running() {
            self.push_log(LogLine::System("RPC already running".into()));
            return;
        }

        let rpc_path = match rpc_binary_path() {
            Ok(p) => p,
            Err(e) => {
                self.push_log(LogLine::System(format!("{e}")));
                return;
            }
        };

        let (tx, rx) = mpsc::channel::<LogLine>();

        let mut cmd = Command::new(&rpc_path);
        cmd.current_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (k, v) in self.env.to_env_map() {
            cmd.env(k, v);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                self.push_log(LogLine::System(format!("Started RPC: {}", rpc_path.display())));

                if let Some(stdout) = child.stdout.take() {
                    let tx2 = tx.clone();
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().flatten() {
                            let _ = tx2.send(LogLine::Stdout(line));
                        }
                    });
                }
                if let Some(stderr) = child.stderr.take() {
                    let tx2 = tx.clone();
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().flatten() {
                            let _ = tx2.send(LogLine::Stderr(line));
                        }
                    });
                }

                self.child = Some(child);
                self.log_rx = Some(rx);
            }
            Err(e) => self.push_log(LogLine::System(format!("Failed to start RPC: {e}"))),
        }
    }

    fn stop_rpc(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.push_log(LogLine::System("Stopped RPC".into()));
        } else {
            self.push_log(LogLine::System("RPC not running".into()));
        }
        self.log_rx = None;
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(msg) = self.tray_backend.ensure_ready() {
            self.push_log(LogLine::System(msg));

            if !self.tray_ready() && !tray_disabled_by_env() {
                if let TrayState::Unavailable { reason } = self.tray_state() {
                    self.tray_backend = Box::new(NoTrayBackend::unavailable(reason));
                }
            }
        }

        for command in self.tray_backend.pump_events() {
            match command {
                TrayCommand::Start => self.start_rpc(),
                TrayCommand::Stop => self.stop_rpc(),
                TrayCommand::Show => self.wants_show = true,
                TrayCommand::Hide => self.wants_hide = true,
                TrayCommand::Quit => self.wants_quit = true,
            }
        }

        loop {
            let next = self.log_rx.as_ref().and_then(|rx| rx.try_recv().ok());
            let Some(line) = next else { break };
            self.push_log(line);
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            if self.tray_ready() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.wants_hide = true;
            } else {
                self.stop_rpc();
            }
        }

        if self.wants_hide {
            self.wants_hide = false;
            if self.tray_ready() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            } else {
                self.push_log(LogLine::System(
                    "Hide ignored: tray unavailable, keeping window visible".into(),
                ));
            }
        }
        if self.wants_show {
            self.wants_show = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                egui::UserAttentionType::Informational,
            ));
        }
        if self.wants_quit {
            self.wants_quit = false;
            self.stop_rpc();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let running = self.running();

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Step Tracker RPC");
                ui.separator();
                ui.label(if running {
                    "Status: running"
                } else {
                    "Status: stopped"
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load .env").clicked() {
                    self.load_env();
                }
                if ui.button("Save .env").clicked() {
                    self.save_env();
                }
                ui.separator();
                if ui
                    .add_enabled(!running, egui::Button::new("Start RPC"))
                    .clicked()
                {
                    self.start_rpc();
                }
                if ui
                    .add_enabled(running, egui::Button::new("Stop RPC"))
                    .clicked()
                {
                    self.stop_rpc();
                }
                ui.separator();
                if ui
                    .add_enabled(self.tray_ready(), egui::Button::new("Hide (keep running)"))
                    .clicked()
                {
                    self.wants_hide = true;
                }
            });

            ui.add_space(10.0);

            egui::CollapsingHeader::new("API")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("API_URL");
                    ui.text_edit_singleline(&mut self.env.api_url);
                    ui.add_space(4.0);
                    ui.label("API_TOKEN");
                    ui.add(egui::TextEdit::singleline(&mut self.env.api_token).password(true));
                });

            ui.add_space(8.0);
            egui::CollapsingHeader::new("Steps Rich Presence")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.env.enable_steps, "Enable steps");
                    ui.label("STEPS_DISCORD_CLIENT_ID");
                    ui.text_edit_singleline(&mut self.env.steps_discord_client_id);
                    ui.label("STEPS_DISCORD_LARGE_IMAGE_KEY");
                    ui.text_edit_singleline(&mut self.env.steps_large_image_key);
                });

            ui.add_space(8.0);
            egui::CollapsingHeader::new("Water Rich Presence")
                .default_open(false)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.env.enable_water, "Enable water");
                    ui.label("WATER_DISCORD_CLIENT_ID");
                    ui.text_edit_singleline(&mut self.env.water_discord_client_id);
                    ui.label("WATER_DISCORD_LARGE_IMAGE_KEY");
                    ui.text_edit_singleline(&mut self.env.water_large_image_key);
                });

            ui.add_space(8.0);
            egui::CollapsingHeader::new("Sleep Rich Presence")
                .default_open(false)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.env.enable_sleep, "Enable sleep");
                    ui.label("SLEEP_DISCORD_CLIENT_ID");
                    ui.text_edit_singleline(&mut self.env.sleep_discord_client_id);
                    ui.label("SLEEP_DISCORD_LARGE_IMAGE_KEY");
                    ui.text_edit_singleline(&mut self.env.sleep_large_image_key);
                });

            ui.add_space(8.0);
            egui::CollapsingHeader::new("OBS output files (optional)")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label("OBS_STEPS_FILE");
                    ui.text_edit_singleline(&mut self.env.obs_steps_file);
                    ui.label("OBS_WATER_FILE");
                    ui.text_edit_singleline(&mut self.env.obs_water_file);
                    ui.label("OBS_SLEEP_FILE");
                    ui.text_edit_singleline(&mut self.env.obs_sleep_file);
                });

            ui.add_space(12.0);
            ui.separator();
            ui.label("RPC logs");
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.logs {
                        ui.label(line);
                    }
                });

            ui.add_space(6.0);
            ui.small(format!("Env file: {}", self.env_path.to_string_lossy()));

            match self.tray_state() {
                TrayState::Ready => {
                    ui.small("Tray: active (close hides window; reopen from tray).");
                }
                TrayState::Unavailable { reason } => {
                    ui.small(format!("Tray: unavailable ({reason})"));
                    ui.small("Window close exits cleanly when tray is unavailable.");
                }
            }
        });

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Step Tracker RPC")
            .with_inner_size([520.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Step Tracker RPC",
        options,
        Box::new(|_cc| Ok(Box::new(GuiApp::new()))),
    )
    .map_err(|e| anyhow!("Failed to start GUI: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{encode_env_value, parse_bool_raw, read_env_file, write_env_file, GuiApp, TrayState};
    use once_cell::sync::Lazy;
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn parse_bool_raw_accepts_supported_values() {
        assert_eq!(parse_bool_raw("true"), Some(true));
        assert_eq!(parse_bool_raw("YES"), Some(true));
        assert_eq!(parse_bool_raw("1"), Some(true));
        assert_eq!(parse_bool_raw("off"), Some(false));
        assert_eq!(parse_bool_raw("No"), Some(false));
        assert_eq!(parse_bool_raw("0"), Some(false));
        assert_eq!(parse_bool_raw("maybe"), None);
    }

    #[test]
    fn encode_env_value_quotes_when_needed() {
        assert_eq!(encode_env_value("plain_value"), "plain_value");
        assert_eq!(encode_env_value("value with spaces"), "\"value with spaces\"");
        assert_eq!(encode_env_value("hash#value"), "\"hash#value\"");
        assert_eq!(
            encode_env_value("quote\"value"),
            "\"quote\\\"value\""
        );
    }

    #[test]
    fn read_write_env_file_round_trip() {
        let test_dir = unique_temp_dir();
        fs::create_dir_all(&test_dir).expect("failed to create test directory");

        let env_path = test_dir.join(".env");
        let mut data = BTreeMap::new();
        data.insert("API_URL".to_string(), "https://example.com".to_string());
        data.insert("API_TOKEN".to_string(), "token123".to_string());
        data.insert("ENABLE_STEPS".to_string(), "true".to_string());
        data.insert("OBS_STEPS_FILE".to_string(), "/tmp/steps.txt".to_string());
        data.insert("EMPTY_VALUE".to_string(), "   ".to_string());

        write_env_file(&env_path, &data).expect("failed to write env file");
        let loaded = read_env_file(&env_path).expect("failed to read env file");

        assert_eq!(loaded.get("API_URL").map(String::as_str), Some("https://example.com"));
        assert_eq!(loaded.get("API_TOKEN").map(String::as_str), Some("token123"));
        assert_eq!(loaded.get("ENABLE_STEPS").map(String::as_str), Some("true"));
        assert_eq!(
            loaded.get("OBS_STEPS_FILE").map(String::as_str),
            Some("/tmp/steps.txt")
        );
        assert!(!loaded.contains_key("EMPTY_VALUE"));

        fs::remove_dir_all(test_dir).expect("failed to remove test directory");
    }

    #[test]
    fn gui_startup_smoke_test_no_tray() {
        let _lock = ENV_LOCK.lock().expect("failed to lock env mutex");
        std::env::set_var("GUI_DISABLE_TRAY", "true");

        let app = GuiApp::new();
        assert!(!app.tray_ready());
        assert!(matches!(
            app.tray_state(),
            TrayState::Unavailable { .. }
        ));
    }

    fn unique_temp_dir() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "step-tracker-rpc-gui-tests-{}-{}",
            std::process::id(),
            timestamp
        ))
    }
}
