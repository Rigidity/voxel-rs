use std::{sync::Arc, time::Instant};

use winit::{
    dpi::PhysicalPosition,
    window::{CursorGrabMode, Window},
};

use crate::{GameState, Input, Renderer};

pub struct AppState {
    pub renderer: Renderer,
    frame_count: u32,
    last_fps_print_time: Instant,
    last_frame_time: Instant,
    pub game_state: GameState,
    pub input: Input,
    pub window: Arc<Window>,
}

impl AppState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let renderer = Renderer::new(window.clone()).await?;
        let game_state = GameState::new();
        let input = Input::new();
        let now = Instant::now();

        Ok(Self {
            renderer,
            frame_count: 0,
            last_fps_print_time: now,
            last_frame_time: now,
            game_state,
            input,
            window,
        })
    }

    pub fn update(&mut self) {
        // Calculate delta time
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        let is_mouse_locked = self.input.is_mouse_locked();

        self.game_state.tick(&mut self.input, delta_time);

        if is_mouse_locked != self.input.is_mouse_locked() {
            if self.input.is_mouse_locked() {
                self.window.set_cursor_visible(false);

                let size = self.window.inner_size();
                let center = PhysicalPosition::new(size.width as f64 / 2., size.height as f64 / 2.);

                if let Err(error) = self.window.set_cursor_position(center) {
                    log::warn!("Failed to set cursor position: {error}");
                }

                if let Err(error_locked) = self.window.set_cursor_grab(CursorGrabMode::Locked)
                    && let Err(error_confined) =
                        self.window.set_cursor_grab(CursorGrabMode::Confined)
                {
                    log::warn!("Failed to lock mouse: {error_locked}");
                    log::warn!("Failed to confine mouse: {error_confined}");
                }
            } else {
                if let Err(error) = self.window.set_cursor_grab(CursorGrabMode::None) {
                    log::warn!("Failed to unlock mouse: {error}");
                }

                self.window.set_cursor_visible(true);
            }
        }

        let size = self.window.inner_size();

        self.game_state.projection.resize(size.width, size.height);

        self.renderer
            .update_camera(&self.game_state.camera, &self.game_state.projection);

        self.input.finish_tick();

        self.renderer.tick(&mut self.game_state.world);

        // FPS calculation
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_print_time);

        if elapsed.as_secs() >= 1 {
            let fps = self.frame_count as f64 / elapsed.as_secs_f64();
            println!("FPS: {:.2}", fps);
            self.frame_count = 0;
            self.last_fps_print_time = now;
        }
    }
}
