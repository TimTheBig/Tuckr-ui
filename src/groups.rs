use tuckr::dotfiles::{self, ReturnCode};

/// Exclude groups sufixed with _<os_name>
// fn default_exclude() -> Vec<String> {
// 	tuckr::dotfiles::group_ends_with_target_name(group)
// }
pub fn load_groups(output: &mut String) -> Result<Vec<String>, ReturnCode> {
	let dotfiles_dir = match dotfiles::get_dotfiles_path(output) {
		Ok(path) => path,
		Err(e) => {
			eprintln!("{e}");
			return Err(ReturnCode::NoSetupFolder);
		}
	}
	.join("Configs");

	let groups: Vec<_> = dotfiles_dir
		.read_dir()
		.unwrap()
		.filter_map(|f| {
			let f = f.unwrap();
			if f.file_type().unwrap().is_dir() {
				Some(f.file_name().into_string().unwrap())
			} else {
				None
			}
		})
		.collect();

	Ok(groups)
}
