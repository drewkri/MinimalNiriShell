use smithay_client_toolkit::{
    reexports::client::{Proxy, QueueHandle, protocol::wl_output::WlOutput},
    shell::{WaylandSurface, wlr_layer::Anchor},
};

use crate::{
    AppState, CornerPosition, Window,
    WindowType::{DesktopBackground, HotCorner, OverviewBackdrop},
};

impl AppState {
    /// Creates new windows for the given output according to the arguments passed to the CLI.
    pub fn create_windows_for_output(&mut self, output: &WlOutput, qh: &QueueHandle<Self>) {
        let output_id = output.id().protocol_id();

        // Create windows for each output
        let corner_surface = self.compositor_state.create_surface(&qh);
        let corner_window = self.layer_shell.create_layer_surface(
            &qh,
            corner_surface,
            smithay_client_toolkit::shell::wlr_layer::Layer::Top,
            Some("vicinae-shell-integration.hot-corner"),
            None,
        );
        corner_window.set_size(
            self.args.corner_width.unwrap_or(10),
            self.args.corner_height.unwrap_or(10),
        );
        corner_window.set_layer(smithay_client_toolkit::shell::wlr_layer::Layer::Top);
        // Set anchor based on cli args
        let anchor = match self.args.corner_position {
            CornerPosition::TopRight => {
                smithay_client_toolkit::shell::wlr_layer::Anchor::TOP
                    | smithay_client_toolkit::shell::wlr_layer::Anchor::RIGHT
            }
            CornerPosition::TopLeft => {
                smithay_client_toolkit::shell::wlr_layer::Anchor::TOP
                    | smithay_client_toolkit::shell::wlr_layer::Anchor::LEFT
            }
            CornerPosition::BottomRight => {
                smithay_client_toolkit::shell::wlr_layer::Anchor::BOTTOM
                    | smithay_client_toolkit::shell::wlr_layer::Anchor::RIGHT
            }
            CornerPosition::BottomLeft => {
                smithay_client_toolkit::shell::wlr_layer::Anchor::BOTTOM
                    | smithay_client_toolkit::shell::wlr_layer::Anchor::LEFT
            }
        };
        corner_window.set_anchor(anchor);
        corner_window.commit();
        self.windows.insert(
            corner_window.wl_surface().id().protocol_id(),
            Window {
                surface: corner_window,
                window_type: HotCorner,
                output_id,
            },
        );

        // Create background window if needed
        if self.args.background.is_some() {
            let bg_surface = self.compositor_state.create_surface(&qh);
            let bg_window = self.layer_shell.create_layer_surface(
                &qh,
                bg_surface,
                smithay_client_toolkit::shell::wlr_layer::Layer::Top,
                Some("vicinae-shell-integration.background"),
                None,
            );
            bg_window.set_size(0, 0); // autosized
            bg_window.set_layer(smithay_client_toolkit::shell::wlr_layer::Layer::Background);
            bg_window.set_anchor(Anchor::all());
            bg_window.set_keyboard_interactivity(
                smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None,
            );
            bg_window.set_exclusive_zone(-1);
            bg_window.commit();
            self.windows.insert(
                bg_window.wl_surface().id().protocol_id(),
                Window {
                    surface: bg_window,
                    window_type: DesktopBackground,
                    output_id,
                },
            );
        }

        // Create backdrop window if needed
        // Nearly a copy of above except the namespace is different
        // and the listed type is different
        if self.args.backdrop.is_some() {
            let bg_surface = self.compositor_state.create_surface(&qh);
            let bg_window = self.layer_shell.create_layer_surface(
                &qh,
                bg_surface,
                smithay_client_toolkit::shell::wlr_layer::Layer::Top,
                Some("vicinae-shell-integration.backdrop"),
                None,
            );
            bg_window.set_size(0, 0); // autosized
            bg_window.set_layer(smithay_client_toolkit::shell::wlr_layer::Layer::Background);
            bg_window.set_anchor(Anchor::all());
            bg_window.set_keyboard_interactivity(
                smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None,
            );
            bg_window.set_exclusive_zone(-1);
            bg_window.commit();
            self.windows.insert(
                bg_window.wl_surface().id().protocol_id(),
                Window {
                    surface: bg_window,
                    window_type: OverviewBackdrop,
                    output_id,
                },
            );
        }
    }

    /// Destroys all windows on this output.
    pub fn destroy_windows_for_output(&mut self, output: &WlOutput) {
        let output_id = output.id().protocol_id();
        self.windows
            .retain(|_, window| window.output_id != output_id);
    }
}
