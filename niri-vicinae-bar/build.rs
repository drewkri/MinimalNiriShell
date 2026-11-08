fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/niri-touch-hud.gresource.xml",
        "niri-touch-hud.gresource",
    );
}
