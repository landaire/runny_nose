extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::mpsc, time::Duration};

use directories::ProjectDirs;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use nwd::NwgUi;
use nwg::{AnimationTimer, NativeUi};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static WATCHER: OnceCell<RecommendedWatcher> = OnceCell::new();

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    replays_path: Option<PathBuf>,
}

#[derive(Default, NwgUi)]
pub struct RunnyNose {
    #[nwg_control(size: (600, 400), position: (300, 300), title: "Runny Nose - WoWs build sniffer", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnInit: [RunnyNose::on_init], OnWindowClose: [RunnyNose::say_goodbye] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "Replay Directory")]
    #[nwg_layout_item(layout: grid, col: 0, row: 0, col_span: 2)]
    replays_label: nwg::Label,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 1, row: 0, col_span: 2)]
    replays_path_textbox: nwg::TextInput,

    #[nwg_control(text: "...")]
    #[nwg_layout_item(layout: grid, col: 3, row: 0)]
    #[nwg_events( OnButtonClick: [RunnyNose::select_replay_path] )]
    replays_path_button: nwg::Button,

    #[nwg_control(ex_flags: ListViewExFlags::GRID | ListViewExFlags::FULL_ROW_SELECT, list_style: ListViewStyle::Detailed)]
    #[nwg_layout_item(layout: grid, col: 0, row: 2, col_span: 4, row_span: 10)]
    games_list_view: nwg::ListView,

    #[nwg_resource(title: "WoWs Replay Directory", action: nwg::FileDialogAction::OpenDirectory)]
    file_dialog: nwg::FileDialog,

    settings: RefCell<Settings>,

    filesystem_watcher: RefCell<Option<RecommendedWatcher>>,

    recv_events: RefCell<Option<mpsc::Receiver<Result<notify::Event, notify::Error>>>>,

    #[nwg_control(interval: Duration::from_secs(1), active: true)]
    #[nwg_events(OnTimerTick: [RunnyNose::on_timer_tick])]
    timer: AnimationTimer,
}

impl RunnyNose {
    fn say_hello(&self) {}

    fn on_timer_tick(&self) {
        if let Some(recv) = &*self.recv_events.borrow() {
            if let Ok(res) = recv.try_recv() {
                println!("{:?}", res);
                match res {
                    Ok(evt) => match evt.kind {
                        notify::EventKind::Create(_) => {
                            for path in &evt.paths {
                                if let Some(extension) = path.extension() {
                                    if extension == "wowsreplay" {
                                        self.parse_replay(path);
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Err(_) => todo!(),
                }
            }
        }
    }

    fn say_goodbye(&self) {
        nwg::stop_thread_dispatch();
    }

    fn on_init(self: &Rc<Self>) {
        let (tx, rx) = mpsc::channel();
        *self.recv_events.borrow_mut() = Some(rx);
        let mut watcher = notify::recommended_watcher(tx).expect("failed to create watcher");

        let settings = load_settings();
        if let Some(path) = &settings.replays_path {
            self.replays_path_textbox.set_text(path.to_str().unwrap());
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .expect("failed to watch directory");
        }

        WATCHER.set(watcher).expect("failed to set watcher");

        *self.settings.borrow_mut() = settings;

        for &column in &["Team", "Name", "Class", "Ship", "Captain Points Allocated"] {
            self.games_list_view.insert_column(column);
        }
        self.games_list_view.set_headers_enabled(true);

        self.games_list_view
            .insert_items_row(None, &["Red", "Test", "test", "Test", "Test"])
    }

    fn select_replay_path(&self) {
        if self.file_dialog.run(Some(&self.window)) {
            if let Ok(path) = self.file_dialog.get_selected_item() {
                self.replays_path_textbox.set_text(path.to_str().unwrap());

                let mut fs_watcher = self.filesystem_watcher.borrow_mut();
                if let Some(watcher) = fs_watcher.as_mut() {
                    if let Some(replays_path) = &self.settings.borrow().replays_path {
                        watcher
                            .unwatch(replays_path)
                            .expect("failed to unwatch directory");
                    }
                }
                let path = PathBuf::from(path);

                self.settings.borrow_mut().replays_path = Some(PathBuf::from(path));

                save_settings(&self.settings.borrow());
            }
        }
    }
}

fn proj_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "landaire", "runny_nose")
}

fn config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = proj_dirs() {
        Some(proj_dirs.config_dir().join("config.toml"))
    } else {
        None
    }
}

fn save_settings(settings: &Settings) {
    if let Some(proj_dirs) = proj_dirs() {
        // Check if the directory exists -- if not, let's create it
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir).expect("failed to create config dir");
        }
        std::fs::write(
            config_path().expect("failed to get config path"),
            toml::to_vec(settings).expect("failed to serialize settings"),
        )
        .expect("failed to write settings")
    }
}

fn load_settings() -> Settings {
    let config_path = config_path();
    let default_settings = Settings::default();
    if let Some(config_path) = config_path {
        if config_path.exists() {
            // We may fail to load settigns here because they're corrupt or something -- just plow
            // over settings in that event
            if let Ok(settings) = toml::from_slice(
                std::fs::read(config_path)
                    .expect("failed to read settings data")
                    .as_slice(),
            ) {
                return settings;
            }
        }
    }

    save_settings(&default_settings);
    default_settings
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let ui = Default::default();
    let _app = RunnyNose::build_ui(ui).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
