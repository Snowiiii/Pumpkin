[![Crates.io](https://img.shields.io/crates/v/egui_logger)](https://crates.io/crates/egui_logger)
[![docs.rs](https://img.shields.io/docsrs/egui_logger)](https://docs.rs/egui_logger/latest/egui_logger/)



# egui_logger
This library implements [`log`](https://crates.io/crates/log) logging support into [`egui`](https://crates.io/crates/egui) applications.
There is also advanced search via regex.

## Demo
![demo](images/egui_logger.png "Demo")

## Example

### initilazing:
```rust
fn main() {
  // Should be called very early in the program.
  egui_logger::builder().init().unwrap();
}
```

### inside your ui logic:

```rust
fn ui(ctx: &egui::Context) {
    egui::Window::new("Log").show(ctx, |ui| {
        // draws the logger ui.
        egui_logger::logger_ui().show(ui);
    });
}
```

## Alternatives
- [egui_tracing](https://crates.io/crates/egui_tracing) primarily for the [tracing](https://crates.io/crates/tracing) create, but also supports log.

## Contribution
Feel free to open issues and pull requests.
