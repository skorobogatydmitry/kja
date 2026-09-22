fn main() -> eframe::Result {
    env_logger::init();

    // shellpass::set_panic_handler();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_inner_size([600.0, 200.0])
            .with_min_inner_size([400.0, 150.0])
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        kj_assoc::NAME_VERSION,
        options,
        Box::new(|cc| {
            // egui_extras::install_image_loaders(&cc.egui_ctx);
            kj_assoc::App::new(cc)
        }),
    )
}
