use std::f32::consts::TAU;

use egui::{DragValue, Widget};
use glam::{Mat4, Vec2, Vec3};
use puffin::profile_function;

/// A user-controlled camera that orbits around the origin.
#[derive(Debug)]
pub struct ArcBallCamera {
    pitch_revs: f32,
    yaw_revs: f32,
    dist: f32,

    aspect_ratio: f32,
    fov_y_revs: f32,
}

impl Default for ArcBallCamera {
    fn default() -> Self {
        Self {
            pitch_revs: -1. / 16.,
            yaw_revs: 1. / 16.,
            dist: 10.,

            aspect_ratio: 16. / 9.,
            fov_y_revs: 1. / 8.,
        }
    }
}

impl ArcBallCamera {
    /// Call once per frame to update the camera's parameters with the current input state.
    pub fn update(&mut self, input: &egui::InputState) {
        profile_function!();

        let (width, height) = input.screen_rect().size().into();
        self.aspect_ratio = width / height;
        if input.pointer.primary_down() {
            // Note: cursor_delta comes from the filtered [`WindowEvent::CursorMoved`] events, even though winit's docs
            // recommend using unfiltered [`DeviceEvent::MouseMoved`] instead for 3D camera motion. But I want camera
            // movement to be directly proportional to the delta in pixel position of the cursor.

            let (delta_x, delta_y) = input.pointer.delta().into();
            let clip_cursor_diff =
                Vec2::new(delta_x as _, delta_y as _) / Vec2::new(width as _, height as _);

            let pitch_delta = clip_cursor_diff.y * self.fov_y_revs;
            self.pitch_revs = (self.pitch_revs - pitch_delta).clamp(-0.25, 0.25);

            let fov_x_revs =
                2. * ((TAU * self.fov_y_revs / 2.).tan() * self.aspect_ratio).atan() / TAU;
            let yaw_delta = clip_cursor_diff.x * fov_x_revs;
            self.yaw_revs = (self.yaw_revs - yaw_delta).rem_euclid(1.);
        }

        let (_, scroll_y) = input.smooth_scroll_delta.into();
        self.dist -= scroll_y * input.stable_dt;

        self.fov_y_revs = (self.fov_y_revs / input.zoom_delta()).clamp(0.0001, 1. / 2.);
    }

    /// Show a gui window for modifying the camera parameters.
    pub fn run_ui(&mut self, ui: &mut egui::Ui) {
        profile_function!();

        egui::Grid::new("Camera").num_columns(2).show(ui, |ui| {
            ui.label("pitch");
            DragValue::new(&mut self.pitch_revs)
                .suffix(" turns")
                .speed(0.01)
                .ui(ui);
            ui.end_row();
            self.pitch_revs = self.pitch_revs.clamp(-0.25, 0.25);

            ui.label("yaw");
            DragValue::new(&mut self.yaw_revs)
                .suffix(" turns")
                .speed(0.01)
                .ui(ui);
            ui.end_row();
            self.yaw_revs = self.yaw_revs.rem_euclid(1.);

            ui.label("distance");
            DragValue::new(&mut self.dist)
                .suffix("m")
                .speed(0.01)
                .ui(ui);
            ui.end_row();

            ui.label("vertical FOV");
            DragValue::new(&mut self.fov_y_revs)
                .suffix(" turns")
                .speed(0.01)
                .range(0. ..=0.5)
                .ui(ui);
            ui.end_row();
        });
    }

    /// Get the premultiplied view and projection matrices.
    pub fn view_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.dist * Vec3::Z);
        let pitch = Mat4::from_rotation_x(TAU * self.pitch_revs);
        let yaw = Mat4::from_rotation_y(TAU * self.yaw_revs);
        (yaw * pitch * translation).inverse()
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_rh(TAU * self.fov_y_revs, self.aspect_ratio, 0.1)
    }
}
