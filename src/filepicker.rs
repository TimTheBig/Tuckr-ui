use crate::app::{Page, TemplateApp, FOLDER_IMAGE};
use eframe::egui;
use egui::{Image, Ui};
use std::{fs, path::PathBuf};

pub fn push_file_picker(app: &mut TemplateApp, ui: &mut Ui) {
	ui.style_mut().spacing.item_spacing = egui::vec2(15.0, 15.0);

	// icons
	let folder_icon = Image::new(FOLDER_IMAGE).fit_to_original_size(1.05);

	if ui
		.add(egui::Button::image_and_text(folder_icon, "Open file…"))
		.clicked()
	{
		if let Some(path) = rfd::FileDialog::new().pick_files() {
			app.push_files = Some(path.iter().filter_map(|p| p.to_str().map(|p| p.to_string())).collect());
			app.page = Page::Push(app.push_files.take())
		}
	}

	if let Some(opened_hook) = &app.opened_hook {
		ui.horizontal(|ui| {
			ui.label("Picked file:");
			ui.label(opened_hook);
		});
	}

	preview_files_being_dropped(app, ui.ctx());

	// Collect dropped files:
	ui.ctx().input(|i| {
		if !i.raw.dropped_files.is_empty() {
			app.dropped_files.clone_from(&i.raw.dropped_files);
		}
	});
}

pub fn hook_file_picker(app: &mut TemplateApp, ui: &mut Ui, hooks_dir: Option<PathBuf>) {
	ui.style_mut().spacing.item_spacing = egui::vec2(15.0, 15.0);

	// icons
	let folder_icon = Image::new(FOLDER_IMAGE).fit_to_original_size(1.05);

	if ui
		.add(egui::Button::image_and_text(folder_icon, "Open file…"))
		.clicked()
	{
		if let Some(path) = rfd::FileDialog::new()
			.add_filter("shell scripts", &["sh"])
			.set_directory(hooks_dir.unwrap_or(PathBuf::from("/")))
			.pick_file()
		{
			app.opened_hook = Some(path.display().to_string());
			// set the contens of the code window to the selected file
			if let Some(s) = &app.opened_hook {
				app.code = String::from_utf8_lossy(&fs::read(s).unwrap()).into()
			}
		}
	}

	if let Some(opened_hook) = &app.opened_hook {
		ui.horizontal(|ui| {
			ui.label("Picked file:");
			ui.label(opened_hook);
		});
	}

	preview_files_being_dropped(app, ui.ctx());

	// Collect dropped files:
	ui.ctx().input(|i| {
		if !i.raw.dropped_files.is_empty() {
			app.dropped_files.clone_from(&i.raw.dropped_files);
		}
	});
}

/// Preview hovering files:
fn preview_files_being_dropped(app: &mut TemplateApp, ctx: &egui::Context) {
	use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
	use std::fmt::Write as _;

	if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
		let text = ctx.input(|i| {
			let mut text = "Dropping files:\n".to_owned();
			for file in &i.raw.hovered_files {
				if let Some(path) = &file.path {
					write!(text, "\n{}", path.display()).ok();
				} else if !file.mime.is_empty() {
					write!(text, "\n{}", file.mime).ok();
				} else {
					text += "\n???";
				}
			}
			text
		});

		let painter = ctx.layer_painter(LayerId::new(
			Order::Foreground,
			Id::new(format!("file_drop_target: {}", app.check_count)),
		));

		let screen_rect = ctx.screen_rect();
		painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
		painter.text(
			screen_rect.center(),
			Align2::CENTER_CENTER,
			text,
			TextStyle::Heading.resolve(&ctx.style()),
			Color32::WHITE,
		);
	}
}
