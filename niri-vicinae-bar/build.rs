fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/niri-vicinae-hud.gresource.xml",
        "niri-vicinae-hud.gresource",
    );
}
