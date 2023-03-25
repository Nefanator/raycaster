
use glam::{Vec2, Vec3, Vec3Swizzles};
use winit::event::{KeyboardInput, VirtualKeyCode, WindowEvent, ElementState};

use crate::{input::InputState, level::{LevelState, Sector}};

pub struct GameState {
    pos: Vec2,
    rot: Vec3,

    input: InputState,
    level: LevelState
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            pos: Vec2::splat(0.0),
            rot: Vec3::NEG_Y,
            input: InputState::default(),
            level: LevelState::demo(),
        }
    }
}

impl GameState {
    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn rot(&self) -> Vec3 {
        self.rot
    }

    pub fn level(&self) -> &LevelState {
        &self.level
    }

    pub fn find_current_sector(&self) -> Option<&Sector> {
        self.level().find_current_sector(self.pos())   
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. } => self.keyboard_input(input),
            _ => false,
        }
    }

    fn keyboard_input(&mut self, input: &KeyboardInput) -> bool {
        let mut handled = true;

        let movement = input.state == ElementState::Pressed;

        match input.virtual_keycode {
            Some(VirtualKeyCode::W) => self.input.forward(movement),
            Some(VirtualKeyCode::S) => self.input.backward(movement),
            Some(VirtualKeyCode::A) => self.input.left(movement),
            Some(VirtualKeyCode::D) => self.input.right(movement),
            Some(VirtualKeyCode::Left) => self.input.rot_left(movement),
            Some(VirtualKeyCode::Right) => self.input.rot_right(movement),
            _ => handled = false,
        };

        handled
    }
    
    pub fn update(&mut self, delta: f32) {
        let move_vec = self.input.move_vec();

        if let Some(rot_dir) = self.input.rot_dir() {
            let angle = match rot_dir {
                crate::input::RotationDirection::Left => -delta,
                crate::input::RotationDirection::Right => delta,
            };

            let quat = glam::Quat::from_axis_angle(Vec3::Z, angle * 5.0);
            self.rot = quat * self.rot;
        }

        let norm_xy_look = self.rot.xy().normalize();
        let corrected_move_vec = norm_xy_look.rotate(move_vec);

        self.pos = self.pos + scale(corrected_move_vec, delta);
    }
}

fn scale(vec: Vec2, scale: f32) -> Vec2 {
    Vec2::new(vec.x * scale, vec.y * scale)
}
