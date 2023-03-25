use glam::Vec2;

#[derive(Default)]
pub struct InputState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    rot_left: bool,
    rot_right: bool,
}

impl InputState {
    pub fn forward(&mut self, movement: bool) {
        self.forward = movement;
    }

    pub fn backward(&mut self, movement: bool) {
        self.backward = movement;
    }

    pub fn left(&mut self, movement: bool) {
        self.left = movement;
    }

    pub fn right(&mut self, movement: bool) {
        self.right = movement;
    }

    pub fn rot_left(&mut self, movement: bool) {
        self.rot_left = movement;
    }

    pub fn rot_right(&mut self, movement: bool) {
        self.rot_right = movement;
    }

    pub fn clear(&mut self) {
        self.forward = false;
        self.backward = false;
        self.left = false;
        self.right = false;
        self.rot_left = false;
        self.rot_right = false;
    }

    pub fn move_vec(&self) -> Vec2 {
        let mut vec = Vec2::ZERO;
        vec.x += if self.forward { 1.0 } else { 0.0 };
        vec.x += if self.backward { -1.0 } else { 0.0 };
        vec.y += if self.left { -1.0 } else { 0.0 };
        vec.y += if self.right { 1.0 } else { 0.0 };
        if vec.length() > 1.0 {
            vec.normalize()
        } else {
            vec
        }
    }

    pub fn rot_dir(&self) -> Option<RotationDirection> {
        match (self.rot_left, self.rot_right) {
            (true, true) => None,
            (true, false) => Some(RotationDirection::Left),
            (false, true) => Some(RotationDirection::Right),
            (false, false) => None,
        }
    }
}

pub enum RotationDirection {
    Left,
    Right,
}
