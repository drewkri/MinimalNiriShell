use std::process::Command;

use anyhow::anyhow;
use fast_image_resize::{
    FilterType::Bilinear, PixelType, ResizeAlg, ResizeOptions, SrcCropping, images::Image,
};
use image::ImageReader;
use log::{error, info, trace, warn};
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::client::{
        Connection, Proxy, QueueHandle,
        protocol::{wl_output, wl_pointer::WlPointer, wl_seat, wl_shm::Format, wl_surface},
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState, pointer::PointerHandler},
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShellHandler, LayerSurface},
    },
    shm::ShmHandler,
};

use crate::{
    AppState,
    WindowType::{self},
};

#[cfg(feature = "bench")]
use std::time::Instant;

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // no windows need redrawing, ever.
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        info!(
            "Received info about new output: {:?}",
            self.output_state.info(&output)
        );
        self.create_windows_for_output(&output, qh);
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        info!("Output updated: {:?}", self.output_state.info(&output));
        self.destroy_windows_for_output(&output);
        self.create_windows_for_output(&output, qh);
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        info!("Output destroyed: {:?}", self.output_state.info(&output));
        self.destroy_windows_for_output(&output);
    }
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");
            self.pointer = Some(pointer);
            info!("Acquired pointer capability")
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_some() {
            self.pointer.take().unwrap().release();
            info!("Removed pointer capability")
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

delegate_registry!(AppState);

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

impl LayerShellHandler for AppState {
    fn closed(
        &mut self,
        _conn: &Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    ) {
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let id = layer.wl_surface().id().protocol_id();
        if let Some(window) = self.windows.get(&id) {
            match window.window_type {
                crate::WindowType::HotCorner => {
                    // Draw a fully transparent window
                    // Required to get the window to accept inputs, but we never need to draw anything again
                    // and nothing will be visible
                    let width = configure.new_size.0 as i32;
                    let height = configure.new_size.1 as i32;
                    let stride = width * 4;
                    let (buffer, canvas) = self
                        .pool
                        .create_buffer(width, height, stride, Format::Argb8888)
                        .expect("unable to create buffer");
                    canvas.iter_mut().for_each(|b| *b = 0); // set the whole thing to zeros
                    buffer
                        .attach_to(layer.wl_surface())
                        .expect("unable to attach buffer to surface");
                    layer.wl_surface().commit();
                    info!("Presented transparent frame for hot corner with id {}", id)
                }
                crate::WindowType::DesktopBackground => {
                    cfg_select! {
                        feature = "bench" => {
                            let now = Instant::now();
                            info!("Starting image draw");
                            self.fill_window_with_image(
                                false,
                                window.surface.clone(),
                                configure.new_size.0,
                                configure.new_size.1,
                            ).unwrap();
                            info!("Drew background image in {:?}", now.elapsed());
                        }
                        _ => {
                            if let Err(e) = self.fill_window_with_image(
                                true,
                                window.surface.clone(),
                                configure.new_size.0,
                                configure.new_size.1,
                            ) {
                                error!("Failed drawing background image with error {:?}", e);
                            } else {
                                info!("Drew background image to window with id {}", id);
                            }
                        }
                    }
                }
                crate::WindowType::OverviewBackdrop => {
                    // try loading the image first
                    cfg_select! {
                        feature = "bench" => {
                            let now = Instant::now();
                            info!("Starting image draw");
                            self.fill_window_with_image(
                                true,
                                window.surface.clone(),
                                configure.new_size.0,
                                configure.new_size.1,
                            ).unwrap();
                            info!("Drew backdrop image in {:?}", now.elapsed());
                        }
                        _ => {
                            if let Err(e) = self.fill_window_with_image(
                                true,
                                window.surface.clone(),
                                configure.new_size.0,
                                configure.new_size.1,
                            ) {
                                error!("Failed drawing backdrop image with error {:?}", e);
                            } else {
                                info!("Drew backdrop image to window with id {}", id);
                            }
                        }
                    }
                }
            }
        } else {
            warn!("Received configure request for unknown window ID: {}", id);
        }
    }
}

impl ShmHandler for AppState {
    fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
        &mut self.shm
    }
}

impl PointerHandler for AppState {
    // the only actual input responding needed in this app is responding to when the pointer enters
    // the corner surface
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            if let Some(window) = self.windows.get(&event.surface.id().protocol_id())
                && window.window_type == WindowType::HotCorner
                && let smithay_client_toolkit::seat::pointer::PointerEventKind::Enter { serial: _ } =
                    event.kind
            {
                let exit_status = Command::new("vicinae")
                    .arg("state")
                    .arg("open")
                    .spawn()
                    .and_then(|mut p| p.wait());
                trace!(
                    "Received pointer enter, probing vicinae returned {:?}",
                    exit_status
                );
                // check if exit code is NOT 0
                if exit_status
                    .ok()
                    .is_none_or(|status| status.code().is_none_or(|c| c != 0))
                {
                    // this command returns 0 when vicinae is open
                    info!("Opening vicinae quick menu");
                    let _ = Command::new("vicinae")
                        .arg("vicinae://launch/@drewkri/quick-menu/quick-menu")
                        .spawn();
                }
            }
        }
    }
}

smithay_client_toolkit::delegate_dispatch2!(AppState);

impl AppState {
    fn fill_window_with_image(
        &mut self,
        use_backdrop: bool,
        layer: LayerSurface,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let path = if use_backdrop {
            self.args.backdrop.as_ref().ok_or(anyhow!(
                "Missing backdrop image path, but requested to draw backdrop image"
            ))?
        } else {
            self.args.background.as_ref().ok_or(anyhow!(
                "Missing background image path, but requested to draw background image"
            ))?
        };
        let stride = width * 4;
        let (buffer, canvas) = self
            .pool
            .create_buffer(width as i32, height as i32, stride as i32, Format::Abgr8888)
            .expect("unable to create buffer");
        let mut img = ImageReader::open(path)?.decode()?;
        let img_converted = img.as_mut_rgba8().unwrap();

        // this also copies the result to the canvas
        let mut canvas_image = Image::from_slice_u8(width, height, canvas, PixelType::U8x4)
            .expect("Failed to create image");
        let opts = ResizeOptions {
            algorithm: match self.args.filter_type {
                crate::BackgroundFilterType::Linear => ResizeAlg::Convolution(Bilinear),
                crate::BackgroundFilterType::Nearest => ResizeAlg::Nearest,
            },
            cropping: match self.args.fill_type {
                crate::FillType::Strecth => SrcCropping::None,
                crate::FillType::Fill => SrcCropping::FitIntoDestination((0.5, 0.5)),
            },

            ..Default::default()
        };
        self.resizer
            .resize(img_converted, &mut canvas_image, Some(&opts))
            .expect("Failed to resize image");

        buffer
            .attach_to(layer.wl_surface())
            .expect("Unable to attach buffer to surface");
        layer.wl_surface().commit();

        Ok(())
    }
}
