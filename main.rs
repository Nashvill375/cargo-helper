#![allow(unused)]

/* 
eframe, egui_extras, and egui_alignments (dependencies of this program) was found at https://crates.io/crates/eframe
with the following license attached


                        YEAR      copyright holder(s)
eframe and egui_extras: 2018-2021 Emil Ernerfeldt <emil.ernerfeldt@gmail.com>
egui_alignments: Unfortunately I was unable to find any information about this person(s), except for his/her username: a-littlebit
    a link to the generic license was attached without any information where "<year>" and "<Fullname>" should have gone

Copyright (c)  <year> <fullname>

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
*/

use crate::egui::UiBuilder;
use crate::egui::DragValue;
use std::fs::ReadDir;
use crate::CargoType::*;
use crate::egui::Response;
use crate::egui::Label;
use std::path::Path;
use crate::egui::InnerResponse;
use crate::egui::Widget;
use crate::egui::Ui;
use crate::egui::Checkbox;
use crate::egui::Align;
use egui_alignments::row;
use egui_alignments::Aligner;
use std::str::FromStr;
use std::cmp::Ordering::*;
use crate::egui::Vec2;
use crate::egui::ImageSource;
use std::{
    io,
    fs::{
        File,
        read_to_string,
        read_dir,
        write,
        exists
    },
    process::{
        Command,
        ExitStatus
    },
    time::{
        Duration,
        Instant,
    }
};
use eframe::egui;
use eframe::egui::{
    include_image,
    Image,
    viewport::ViewportBuilder,
    containers::{
        PopupCloseBehavior::CloseOnClickOutside,
        menu::{
            MenuButton,
            MenuConfig,
        },
        panel::CentralPanel,
        scroll_area::ScrollArea
    },
    Vec2b,
    Button,
    Color32,
    RichText,
    FontId,
    TextFormat,
    text::LayoutJob,
};

fn main() {
    let mut native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native("Program Manager", native_options, Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(ProgramManager::new(cc)))
    }));
}


struct ProgramManager {
    file_name: String,
    file_vexide: bool,
    vexide_slot: f64,
    directory: String,
    term_vis: Instant,
    add_dir: String,
    command_output: RichText,
    paths: PathsFile,
    popup_behave: MenuConfig,
    settings: Settings
}

impl ProgramManager {
    fn new(_cc: &eframe::CreationContext) -> ProgramManager {
        let settings = Settings::new();
        let mut popup_behave = MenuConfig::default();
        popup_behave.close_behavior = CloseOnClickOutside;
        ProgramManager {
            file_name: "File".to_string(),
            file_vexide: false,
            vexide_slot: 0.0,
            directory: "none".to_string(),
            term_vis: Instant::now(),
            command_output: RichText::default(),
            add_dir: settings.default_dir.clone(),
            paths: PathsFile::new(),
            popup_behave,
            settings,
        }
    }
    fn cargo(&mut self, cmd: &str) {
        let mut command = Command::new("cargo");

        command.current_dir(&self.directory);

        let std_out = match (self.file_vexide, self.vexide_slot) {
            (true, 0.0) => command.arg(cmd).status(),
            (true, index) => command.args([cmd, "-s", &format!("{index}")]).status(),
            (false, _) => command.arg(cmd).status(),
        };
        self.cmd_out(std_out);
        self.term_vis = Instant::now();
    }

    fn cmd_out(&mut self, cmd: io::Result<ExitStatus>) {
        self.command_output = if cmd.unwrap().success() {
            RichText::new("Success").color(Color32::GREEN)
        } else {
            RichText::new("Failure").color(Color32::RED)
        };
    }

    fn back_slash(&mut self) {
        let index = self.add_dir.rfind("/");
        self.add_dir.truncate(index.unwrap());
    }

    fn settings_widget(&mut self, ui: &mut Ui, popup: bool) -> InnerResponse<()> {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Default directory: ");
                ui.text_edit_singleline(&mut self.settings.default_dir);
            });
            ui.checkbox(&mut self.settings.show_hidden, "Show hidden files and directories");
            if ui.button("Finish").clicked() {
                if exists(&self.settings.default_dir).expect("Could not determine existence of path") {
                    self.add_dir = self.settings.default_dir.clone();
                    if !exists("settings.csv").unwrap() {
                        File::create("settings.csv");
                    }

                    let settings_format = format!("Default Directory, Show Hidden\n{}, {}", 
                        self.settings.default_dir, self.settings.show_hidden);

                    write("settings.csv", settings_format);
                    if popup {
                        ui.close()
                    }
                }
            }
        })
    }
}

struct Settings {
    default_dir: String,
    show_hidden: bool
}

impl Settings {
    fn new() -> Self {
        if exists("settings.csv").expect("Could not determine the existence of settings") {
            let file = read_to_string("settings.csv").expect("unable to read settings");
            let file_info: Vec<&str> = 
            file
            .lines()
            .nth(1)
            .expect("Could not read the file's contents")
            .split(',')
            .collect();

            Self {
                default_dir: file_info[0].to_string(),
                show_hidden: file_info[1].parse().unwrap_or_else(|_| {
                    false
                })
            }
        } else {
            Self {
                default_dir: "none".to_string(),
                show_hidden: false
            }
        }
    }
}

struct PathsFile {
    file: String,
    roots: Vec<String>,
}

impl PathsFile {
    fn new() -> Self {
        let mut roots = vec!();
        if exists("paths.csv").unwrap() {
            read_to_string("paths.csv").unwrap()
        } else {
            File::create("paths.csv").unwrap();
            read_to_string("paths.csv").unwrap()
        }.split(',').for_each(|path| {
            roots.push(path.to_string());
        });
        let file = read_to_string("paths.csv").unwrap();
        
        Self {
            file,
            roots
        }
    }
}

struct FileButton {
    text: String,
    cargo: CargoType
}

#[derive(PartialEq, Clone, Copy)]
enum CargoType {
    Dir,
    Cargo,
    Vexide
}

impl FileButton {
    fn new(text: String, cargo: CargoType) -> Self {
        Self {
            text,
            cargo,
        }
    }
}

fn filter_paths(directs: &Vec<String>, path: String) -> String {
    let mut output = String::new();
    directs.into_iter()
    .filter(|index| **index != path)
    .for_each(|file_path| {                                            
        output.push_str(&format!(" {file_path},"));
    });
    output.pop();
    output
}

// https://dashboardicons.com/icons/rust CC BY 4.0 
const RUST_PNG: ImageSource = include_image!("rust.png");
//No licese found for this image
const FOLDER_PNG: ImageSource = include_image!("folder.png");
//No license found for this image
const VEXIDE_SVG: ImageSource = include_image!("vexide.svg");

impl eframe::App for ProgramManager {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if !exists("settings.csv").expect("Could not determine the existence of settings") {
                ui.heading("Hello!");
                ui.label("Welcome to the Rust Program Manager! \nBefore we get started, please input some default settings.");
                self.settings_widget(ui, false);
            } else {
                ui.vertical(|main|{
                    row(main, Align::TOP, |main| {
                        Aligner::left_top().show(main, |main| {
                            main.collapsing(RichText::new(&self.file_name)
                            .size(20.0), |dropdown| {
                                if !read_to_string("paths.csv").unwrap().trim().is_empty() {
                                    for file in &self.paths.roots {
                                        let named_file = Path::new(&file)
                                        .file_name()
                                        .expect("Could not retrive file name")
                                        .to_str()
                                        .expect("Could not convert path into file name");
                                        let button = dropdown.add(
                                            Button::new(
                                                RichText::new(named_file)
                                                .size(20.0)
                                            ).frame(false)
                                        );
                                        if button.clicked() {
                                            self.file_name = named_file.to_string();
                                            self.directory = file.to_string();
                                            if read_to_string(Path::new(file.trim()).join("Cargo.lock")).expect(&format!("{:?}", Path::new(file).join("Cargo.lock"))).contains("vexide") {
                                                self.file_vexide = true;
                                            } else {
                                                self.file_vexide = false;
                                            }
                                        }
                                    }
                                } else {
                                    dropdown.heading("No file yet");
                                }
                            });
                        });
                        Aligner::right_top().show(main, |main| {
                            MenuButton::from_button(
                                Button::new(
                                    RichText::new("⚙")
                                    .color(Color32::WHITE)
                                    .size(24.0)
                                ).frame_when_inactive(false)
                            ).config(self.popup_behave.clone()).ui(main, |settings| {
                                self.settings_widget(settings, true);
                            });
                        });
                    });
                    main.horizontal(|build|{

                        let cargo_build = build.add_enabled(self.file_name != "File", Button::new("Build"));

                        let button_text = match self.file_vexide {
                            true => "Upload",
                            false => "Build"
                        };

                        let cargo_run = build.add_enabled(self.file_name != "File", Button::new(button_text));

                        let mut scope_ui_builder = UiBuilder {
                            invisible: !self.file_vexide,
                            ..Default::default()
                        };

                        build.scope_builder(scope_ui_builder, |vex_slot| {
                            vex_slot.add(DragValue::new(&mut self.vexide_slot).range(0..=8).max_decimals(1).prefix("Slot: ").custom_formatter(|slot,_| {
                                match slot {
                                    0.0 => "Default".to_string(),
                                    x => format!("{x}")
                                }
                            }));
                        });

                        if self.term_vis.elapsed() > Duration::from_secs(5) {
                            self.term_vis = Instant::now() + Duration::from_secs(6);
                        }

                        if cargo_run.clicked() {
                            self.cargo(&button_text.to_lowercase());
                        }

                        if cargo_build.clicked() {
                            self.cargo("build");
                        }
                        build.add_visible(self.term_vis.elapsed() < Duration::from_secs(3), eframe::egui::Label::new(self.command_output.clone()));
                    
                    });
                    main.horizontal(|docs| {
                        let mut doc_cmd = Command::new("cargo");
                        if docs.add_enabled(self.file_name != "File", Button::new("Document")).clicked() {
                            self.cmd_out(doc_cmd.arg("doc").status());
                        }

                        if docs.add_enabled(self.file_name != "File", Button::new("Open docs")).clicked() {
                            self.cmd_out(doc_cmd.args(["doc","--open"]).status());
                        }
                    });
                    MenuButton::new("Add file").config(self.popup_behave.clone()).ui(main, |popup|{
                        ScrollArea::both().max_width(150.0).min_scrolled_width(150.0).min_scrolled_height(250.0).show(popup, |menu|{
                            let mut text = LayoutJob::default();

                            text.append(
                                "⬅", 
                                0.0, 
                                TextFormat::simple(FontId::proportional(14.0), Color32::RED));
                            text.append(
                                &self.add_dir, 
                                0.0, 
                                TextFormat::default());

                            if menu.add(Button::new(text).fill(Color32::BLACK)).clicked() {
                                self.back_slash();
                            }

                            let mut cd = read_dir(&self.add_dir).unwrap_or_else(|_| panic!("Error at {}", self.add_dir)).filter(|file| {
                                if self.settings.show_hidden {
                                    true
                                } else {
                                    let dir = file.as_ref().unwrap();
                                    #[cfg(windows)]
                                    if dir.metadata().expect("Could not read file metadata").file_attributes() != 2 {
                                        true
                                    } else {
                                        false
                                    }
                                    #[cfg(target_os = "linux")]
                                    if dir.file_name().display().to_string().starts_with('.') {
                                        false
                                    } else {
                                        true
                                    }
                                }
                            });

                            let mut file_buttons: Vec<FileButton> = cd.filter_map(|dir| {
                                let file = dir.as_ref().unwrap();
                                if file.path().is_dir() {

                                    if file.path().join("Cargo.toml").exists() {
                                        Some(FileButton::new(
                                            file.file_name().display().to_string(),
                                            if file.path().join("Cargo.lock").display().to_string().contains("[[package]]\nname = \"vexide\"") {
                                                Vexide
                                            } else {
                                                Cargo
                                            }
                                        ))
                                    } else {
                                        Some(FileButton::new(
                                            file.file_name().display().to_string(),
                                            Dir
                                        ))
                                    }
                                } else {
                                    None
                                }
                            }).collect();

                            file_buttons.as_mut_slice().sort_by(|file, next_file| {
                                match (file.cargo, next_file.cargo) {
                                    (Vexide, Vexide) => Equal,
                                    (Vexide, Cargo) => Greater,
                                    (Vexide, Dir) => Greater,
                                    (Cargo, Vexide) => Less,
                                    (Cargo, Cargo) => Equal,
                                    (Cargo, Dir) => Greater,
                                    (Dir, Vexide) => Less,
                                    (Dir, Cargo) => Less,
                                    (Dir, Dir) => Equal,
                                }
                            });

                            for files in file_buttons {
                                let image = if files.cargo == Cargo {
                                    Image::new(RUST_PNG).fit_to_exact_size(Vec2::splat(16.0))
                                } else if files.cargo == Vexide {
                                    Image::new(VEXIDE_SVG).fit_to_exact_size(Vec2::splat(16.0))
                                } else {
                                    Image::new(FOLDER_PNG).fit_to_exact_size(Vec2::splat(16.0))
                                };
                                let button = Button::image_and_text(image, &files.text);
                                if menu.add(button).clicked() {
                                    if files.cargo == Vexide || files.cargo == Cargo {
                                        if read_to_string("paths.csv").unwrap().trim().is_empty() {
                                            write("paths.csv", format!("{}/{}", self.add_dir, files.text));
                                        } else {
                                            write("paths.csv", format!("{}, {}/{}", self.paths.file, self.add_dir, files.text));
                                        }
                                        menu.close();
                                        self.paths = PathsFile::new();
                                    } else {
                                        self.add_dir.push_str(&format!("/{}", files.text))
                                    }
                                }
                            }
                        });
                    });

                    MenuButton::new("Remove file").config(self.popup_behave.clone()).ui(main, |remove| {
                        ScrollArea::vertical().max_width(150.0).max_height(250.0).auto_shrink(Vec2b::new(false, true)).show(remove, |remove|{
                            for path in &self.paths.roots.clone() {
                                let button = remove.add(
                                    Button::new(
                                        RichText::new(
                                            Path::new(
                                                path.trim()
                                            )
                                        .file_name()
                                        .expect("Could not derive names from removable directories")
                                        .display()
                                        .to_string()
                                        )
                                        .size(20.0)
                                    ).frame(false)
                                );
                                if button.clicked() {
                                    let mut filtered_paths: String = 
                                    {
                                        filter_paths(&self.paths.roots, path.to_string())
                                    };
                                    write("paths.csv", filtered_paths.trim());
                                    self.paths = PathsFile::new();
                                    remove.close()
                                }
                            }
                        })
                    });
                });
            }
        });
        ctx.request_repaint_after_secs(0.25);
    }
}