/// exacute a tuckr command
use crate::cmd::run;
/// dnd file pickers
use crate::filepicker::{hook_file_picker, push_file_picker};
use egui::{include_image, Image, KeyboardShortcut, Modifiers};
use egui::{Button, Color32, DroppedFile, Ui};
use egui_multiselect::MultiSelect;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use tuckr::dotfiles::ReturnCode;
/// the tuckr state
use tuckr::Cli;

#[derive(Default, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum HookType {
	Pre,
	#[default]
	Post,
}

impl Display for HookType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			HookType::Pre => write!(f, "Pre"),
			HookType::Post => write!(f, "Post"),
		}
	}
}

/// enum for each page/command
#[derive(Default, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub enum Page {
	/// help page
	#[default]
	Help,
	/// the simlink status
	Status,
	/// (exclude, force, adopt)
	Add(Option<Vec<String>>, bool, bool),
	/// exclude
	Rm(Option<Vec<String>>),
	/// (exclude, force, adopt)
	Set(Option<Vec<String>>, bool, bool),
	/// files
	Push(Option<Vec<String>>),
	Pop,
	Init,
	/// create and edit hooks
	Hooks,
}

impl Page {
	/// Prepare for runing a command, push will only use the first group\
	/// `\*` is all groups, None if page is help
	pub fn into_cli(self, groups: Vec<String>) -> Result<Cli, String> {
		match self {
			Page::Help => Err(include_str!("../assets/help.txt").to_string()),
			Page::Status => Ok(Cli::Status { groups: None }),
			// use combobox for exclude and something groups
			Page::Add(exclude, force, adopt) => Ok(Cli::Add {
				groups,
				exclude: exclude.unwrap_or_default(),
				force,
				adopt,
			}),
			Page::Rm(exclude) => Ok(Cli::Rm {
				groups,
				exclude: exclude.unwrap_or_default(),
			}),
			Page::Set(exclude, force, adopt) => Ok(Cli::Set {
				groups,
				exclude: exclude.unwrap_or_default(),
				force,
				adopt,
			}),
			Page::Push(f) => Ok(Cli::Push {
				group: groups[0].clone(),
				files: match f {
					Some(ps) => ps,
					None => return Err("select a path".into()),
				},
			}),
			Page::Pop => Ok(Cli::Pop { groups }),
			Page::Init => Ok(Cli::Init),
			Page::Hooks => Err("editer".into()),
		}
	}
}

impl Display for Page {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Page::Add(_, _, _) => write!(f, "Add"),
			Page::Help => write!(f, "Help"),
			Page::Init => write!(f, "Init"),
			Page::Pop => write!(f, "Pop"),
			Page::Push(_) => write!(f, "Push"),
			Page::Rm(_) => write!(f, "Rm"),
			Page::Set(_, _, _) => write!(f, "Set"),
			Page::Status => write!(f, "Status"),
			Page::Hooks => write!(f, "Hooks"),
		}
	}
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
	/// if the adopt flag is used on add and set
	#[serde(skip)]
	adopt: bool,
	/// timer between checking groups
	#[serde(skip)]
	pub check_count: u32,
	/// exclude
	exclude: Option<Vec<String>>,
	/// if the force flag is used on add and set
	#[serde(skip)]
	force: bool,
	/// Avalibule groups
	#[serde(skip)]
	found_groups: Option<Vec<String>>,
	/// The selected groups
	#[serde(skip)]
	groups: Option<Vec<String>>,
	label: String,
	#[serde(skip)]
	pub output: String,
	/// The page with command data incoded
	pub page: Page,
	/// The last opened hook script
	#[serde(skip)]
	pub code: String,
	/// Open hook shell file
	#[serde(skip)]
	pub opened_hook: Option<String>,
	#[serde(skip)]
	pub dropped_files: Vec<DroppedFile>,
	/// if a new hook should be pre or post
	new_hook_type: HookType,
	/// Paths to files to push
	#[serde(skip)]
	pub push_files: Option<Vec<String>>,
}

// todo add code block for hooks with the egui syntax_highlighting feature

// # consts
// icons
pub const FOLDER_IMAGE: egui::ImageSource<'_> = include_image!("../assets/folder.svg");
// visuals
const PANEL_FILL: Color32 = Color32::from_rgba_premultiplied(5, 18, 29, 247);
const BUTTON_SIZE: f32 = 7.0;
const ROUNDING: f32 = 5.5;
// colors
const INACTIVE: Color32 = Color32::from_rgb(62, 62, 110);
const HOVERED: Color32 = Color32::from_rgb(72, 72, 115);
const ACTIVE: Color32 = Color32::from_rgb(82, 82, 125);
const OPEN: Color32 = Color32::from_rgb(74, 74, 115);

fn visuals() -> egui::Visuals {
	let mut visuals = egui::Visuals {
		dark_mode: true,
		..egui::Visuals::default()
	};

	// Background
	visuals.window_stroke = egui::Stroke::NONE;
	visuals.extreme_bg_color = Color32::from_hex("#11304390").unwrap();
	visuals.code_bg_color = Color32::from_hex("#01243390").unwrap();
	visuals.faint_bg_color = Color32::from_hex("#01243360").unwrap();
	visuals.panel_fill = PANEL_FILL;
	visuals.override_text_color = Color32::from_hex("#14AAA9").ok();
	visuals.widgets.noninteractive.bg_stroke.color = Color32::from_hex("#113443").unwrap();
	// Widget size
	visuals.widgets.inactive.expansion = BUTTON_SIZE;
	visuals.widgets.hovered.expansion = BUTTON_SIZE;
	visuals.widgets.active.expansion = BUTTON_SIZE;
	visuals.widgets.open.expansion = BUTTON_SIZE;
	// Widget rounding
	visuals.widgets.inactive.rounding = ROUNDING.into();
	visuals.widgets.hovered.rounding = ROUNDING.into();
	visuals.widgets.active.rounding = ROUNDING.into();
	visuals.widgets.open.rounding = ROUNDING.into();
	// Widget color
	visuals.widgets.inactive.bg_fill = INACTIVE;
	visuals.widgets.hovered.bg_fill = HOVERED;
	visuals.widgets.active.bg_fill = ACTIVE;
	visuals.widgets.open.bg_fill = OPEN;

	visuals.widgets.inactive.weak_bg_fill = INACTIVE;
	visuals.widgets.hovered.weak_bg_fill = HOVERED;
	visuals.widgets.active.weak_bg_fill = ACTIVE;
	visuals.widgets.open.weak_bg_fill = OPEN;

	visuals
}

fn fonts() -> egui::FontDefinitions {
	let mut font = egui::FontDefinitions::default();
	font.font_data.insert(
		"FiraCode".to_string(),
		egui::FontData::from_static(include_bytes!("../assets/FiraCode-Regular.ttf")),
	);
	font.families
		.get_mut(&egui::FontFamily::Proportional)
		.unwrap()
		.insert(0, "FiraCode".to_string());

	font
}

fn format_vec_str(vstr: &mut Vec<String>) -> String {
	let mut formated_str = String::with_capacity(vstr.len() * 10);
	formated_str.push_str(&vstr.remove(0));
	for str in vstr {
		formated_str.push_str(", ");
		formated_str.push_str(str);
	}
	formated_str
}

impl TemplateApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where we customize the look and feel of egui
		cc.egui_ctx.set_visuals(visuals());
		cc.egui_ctx.set_fonts(fonts());

		cc.egui_ctx.style_mut(|style| {
			for (text_style, font_id) in style.text_styles.iter_mut() {
				font_id.size = match text_style {
					egui::TextStyle::Small => 12.0,
					egui::TextStyle::Body => 14.0,
					egui::TextStyle::Monospace => 14.0,
					egui::TextStyle::Button => 15.0,
					egui::TextStyle::Heading => 23.0,
					egui::TextStyle::Name(_) => 16.0,
				}
			}
		});

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		TemplateApp::default()
	}
}

impl Default for TemplateApp {
	fn default() -> Self {
		Self {
			force: false,
			adopt: false,
			check_count: 9975,
			page: Page::default(),
			groups: None,
			exclude: None,
			found_groups: None,
			label: String::new(),
			output: String::new(),
			code: String::new(),
			opened_hook: None,
			dropped_files: Vec::new(),
			new_hook_type: HookType::default(),
			push_files: None,
		}
	}
}

impl eframe::App for TemplateApp {
	/// make the window transparent
	fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
		[0.0, 0.0, 0.0, 0.0]
	}

	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui
		type GroupsHandle = JoinHandle<(Result<Vec<String>, ReturnCode>, String)>;

		let mut groups_handle: Option<GroupsHandle> = None;
		if self.check_count >= 10000 {
			groups_handle = Some(thread::spawn(|| {
				let mut output = "".to_string();
				(crate::groups::load_groups(&mut output), output)
			}));
			self.check_count = 0;
		}
		self.check_count += 1;

		egui::CentralPanel::default()
			.frame(
				egui::Frame::default()
					.fill(PANEL_FILL)
					.inner_margin(egui::Margin::symmetric(12.0, 12.0)),
			)
			.show(ctx, |ui| {
				// The central panel the region left after adding TopPanel's and SidePanel's
				ui.heading("Tuckr UI");

				ui.style_mut().spacing.item_spacing = egui::vec2(15.0, 15.0);
				ui.separator();

				ui.vertical_centered(|ui| {
					ui.horizontal(|ui| {
						egui::ComboBox::from_id_source(4)
							.selected_text(format!("{}", self.page))
							.show_ui(ui, |ui| {
								ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 5.0);
								ui.selectable_value(
									&mut self.page,
									Page::Add(self.exclude.clone(), self.force, self.adopt),
									"Add",
								);
								ui.selectable_value(&mut self.page, Page::Help, "Help");
								ui.selectable_value(&mut self.page, Page::Init, "Init");
								ui.selectable_value(&mut self.page, Page::Pop, "Pop");
								ui.selectable_value(&mut self.page, Page::Push(self.push_files.clone()), "Push");
								ui.selectable_value(&mut self.page, Page::Rm(self.exclude.clone()), "Rm");
								ui.selectable_value(
									&mut self.page,
									Page::Set(self.exclude.clone(), self.force, self.adopt),
									"Set",
								);
								ui.selectable_value(&mut self.page, Page::Status, "Status");
								ui.selectable_value(&mut self.page, Page::Hooks, "Hooks");
								ui.style_mut().spacing.item_spacing = egui::vec2(15.0, 15.0);
							});

						ui.add_space(10.0);

						// icon for refresh button
						let refresh_icon = Image::new(include_image!("../assets/refresh.svg"));

						// check if refresh button or CTRL+R are presed
						if ui.input_mut(|i| {
							egui::InputState::consume_shortcut(
								i,
								&KeyboardShortcut {
									modifiers: Modifiers::COMMAND,
									logical_key: egui::Key::R,
								},
							)
						}) || ui.add(Button::image(refresh_icon)).clicked()
						{
							self.check_count = 0;
							groups_handle = Some(thread::spawn(|| {
								let mut output = "".to_string();
								(crate::groups::load_groups(&mut output), output)
							}));
						}
					});

					ui.end_row();
					// group  selector
					if self.found_groups.is_some() {
						group_select(self, ui);
					}

					// flags
					ui.horizontal(|ui| {
						ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 5.0);

						ui.checkbox(&mut self.force, "");
						ui.label("Force");
						ui.add_space(10.0);
						ui.checkbox(&mut self.adopt, "");
						ui.label("Adopt");

						ui.style_mut().spacing.item_spacing = egui::vec2(15.0, 15.0);
					});

					// if the page is hooks list groups and hook files then open it in a editer
					match self.page {
						Page::Hooks => {
							let save_icon = Image::new(include_image!("../assets/save.svg")).fit_to_original_size(0.23);
							let new_icon = Image::new(include_image!("../assets/new.svg")).fit_to_original_size(0.23);
							ui.horizontal(|ui| {
								if (ui.add(Button::image(save_icon))).clicked() {
									let save = fs::write(
										match &self.opened_hook {
											Some(p) => p,
											None => return,
										},
										&self.code,
									);
									match save {
										Ok(()) => self.output = "saved".to_string(),
										Err(e) => self.output = e.to_string(),
									}
								}
								ui.add_space(3.0);

								let hooks_dir = match tuckr::dotfiles::get_dotfiles_path(&mut "".into()) {
									Ok(p) => Some(p.join("Hooks")),
									Err(e) => return self.output.push_str(&e.to_string()),
								};
								hook_file_picker(self, ui, hooks_dir.clone());
								ui.add_space(3.0);
								new_hook(self, ui, hooks_dir, new_icon);
							});

							code_editer(self, ui);
						}
						Page::Push(_) => push_file_picker(self, ui),
						_ => (),
					}

					// groups
					ui.label(format_vec_str(
						&mut self.found_groups.clone().unwrap_or(vec!["".into()]),
					));

					if self.page != Page::Hooks && ui.button("Exacute").clicked() {
						match self
							.page
							.clone()
							.into_cli(self.groups.clone().unwrap_or(vec![r"\*".into()]))
						{
							Ok(cli) => self.output = run(cli).0,
							Err(h) => {
								self.output = h;
								self.label = "select a group".into();
							}
						};
					}

					ui.label(&self.output);

					ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
						egui::warn_if_debug_build(ui);
						#[cfg(debug_assertions)]
						ui.label(self.check_count.to_string());
					});
				});
			});

		if let Some(groups) = groups_handle {
			if let Ok(g) = groups.join() {
				self.output.push_str(&g.1);
				self.found_groups = g.0.ok();
				self.groups = self.found_groups.clone()
			}
		}
	}
}

fn code_editer(app: &mut TemplateApp, ui: &mut Ui) {
	let theme = egui_extras::syntax_highlighting::CodeTheme::from_style(ui.style());
	// todo patch egui_extras to use tmTheme file

	let mut layouter = |ui: &egui::Ui, code: &str, wrap_width: f32| {
		let mut layout_job = egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, code, "bash");
		layout_job.wrap.max_width = wrap_width;
		ui.fonts(|f| f.layout_job(layout_job))
	};

	egui::ScrollArea::vertical().show(ui, |ui| {
		ui.add(
			egui::TextEdit::multiline(&mut app.code)
				.font(egui::TextStyle::Monospace) // for cursor height
				.code_editor()
				.desired_rows(10)
				.lock_focus(true)
				.desired_width(f32::INFINITY)
				.layouter(&mut layouter),
		);
	});
}

fn group_select(app: &mut TemplateApp, ui: &mut Ui) {
	ui.add(MultiSelect::new(
		"test_multiselect",
		&mut app.groups.as_mut().unwrap().clone(),
		app.groups.as_mut().unwrap(),
		app.found_groups.as_ref().unwrap(),
		|ui, _text| ui.selectable_label(false, _text),
		match app.page {
			Page::Push(_) => &1,
			_ => &255,
		},
		"Choose one or more groups",
	));
}

fn new_hook(app: &mut TemplateApp, ui: &mut Ui, hooks_dir: Option<PathBuf>, new_icon: Image<'_>) {
	if ui.add(Button::image_and_text(new_icon, "hook")).clicked() {
		let mut file_name = app.new_hook_type.to_string().to_lowercase();
		file_name.push_str(".sh");

		rfd::FileDialog::new()
			.add_filter("shell scripts", &["sh"])
			.set_file_name(file_name)
			.set_directory(hooks_dir.unwrap_or_default())
			.save_file();
	}
	ui.add_space(3.0);

	egui::ComboBox::from_label("stage")
		.selected_text(app.new_hook_type.to_string())
		.show_ui(ui, |ui| {
			ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 5.0);
			ui.selectable_value(&mut app.new_hook_type, HookType::Pre, "Pre");
			ui.selectable_value(&mut app.new_hook_type, HookType::Post, "Post");
			ui.style_mut().spacing.item_spacing = egui::vec2(15.0, 15.0);
		});
}
