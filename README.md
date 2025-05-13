# Tuckr ui
### A ui for the stow alternative [Tuckr](https://github.com/RaphGL/tuckr), built with egui and rust

![984448282-Tuckr_ui](https://github.com/user-attachments/assets/a3b39068-a3b5-4736-8f79-cd1d37370bd8)

### Build and Bundle
to bundle deb use
```sh
cargo install --locked cargo-zigbuild
cargo deb --compress-type gz --profile release -o target/ --cargo-build zigbuild --no-strip --target x86_64-unknown-linux-gnu
```

to bundle .app for mac
```sh
cargo install cargo-bundle
cargo bundle --release
```

### icons
all icons are from flatcon
