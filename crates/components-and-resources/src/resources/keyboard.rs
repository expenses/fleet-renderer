use crate::resources::{Camera, Orbit};
use ultraviolet::Vec3;
use winit::event::VirtualKeyCode;

#[derive(Default)]
pub struct KeyboardState {
    pub camera_forwards: bool,
    pub camera_left: bool,
    pub camera_back: bool,
    pub camera_right: bool,
    pub center_camera: Tapped,
    pub fire: bool,
    pub shift: bool,
    pub stop: Tapped,
    pub pause: Tapped,
    pub unload: Tapped,
    pub attack_move: Tapped,
    pub escape: Tapped,
    pub load: Tapped,
    pub build_fighter: Tapped,
    pub build_miner: Tapped,
    pub build_carrier: Tapped,
}

#[derive(Default)]
pub struct Tapped(pub bool);

impl Tapped {
    fn handle(&mut self, pressed: bool) {
        self.0 |= pressed;
    }

    fn reset(&mut self) {
        self.0 = false;
    }
}

impl KeyboardState {
    pub fn handle(&mut self, key: VirtualKeyCode, pressed: bool) {
        match key {
            VirtualKeyCode::Up => self.camera_forwards = pressed,
            VirtualKeyCode::Left => self.camera_left = pressed,
            VirtualKeyCode::Down => self.camera_back = pressed,
            VirtualKeyCode::Right => self.camera_right = pressed,
            VirtualKeyCode::C => self.center_camera.handle(pressed),
            VirtualKeyCode::F => self.fire = pressed,
            VirtualKeyCode::LShift => self.shift = pressed,
            VirtualKeyCode::S => self.stop.handle(pressed),
            VirtualKeyCode::P => self.pause.handle(pressed),
            VirtualKeyCode::U => self.unload.handle(pressed),
            VirtualKeyCode::A => self.attack_move.handle(pressed),
            VirtualKeyCode::Escape => self.escape.handle(pressed),
            VirtualKeyCode::L => self.load.handle(pressed),
            VirtualKeyCode::B => self.build_fighter.handle(pressed),
            VirtualKeyCode::N => self.build_miner.handle(pressed),
            VirtualKeyCode::M => self.build_carrier.handle(pressed),
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.center_camera.reset();
        self.stop.reset();
        self.pause.reset();
        self.unload.reset();
        self.escape.reset();
        self.attack_move.reset();
        self.load.reset();

        self.build_fighter.reset();
        self.build_miner.reset();
        self.build_carrier.reset();
    }

    pub fn move_camera(&self, camera: &mut Camera, orbit: &Orbit) -> bool {
        let forwards = self.camera_forwards as i8 - self.camera_back as i8;
        let right = self.camera_right as i8 - self.camera_left as i8;

        if forwards != 0 || right != 0 {
            let forwards = forwards as f32;
            let right = right as f32;

            camera.center -= Vec3::new(
                forwards * orbit.latitude.sin() - right * orbit.latitude.cos(),
                0.0,
                forwards * orbit.latitude.cos() + right * orbit.latitude.sin(),
            );

            true
        } else {
            false
        }
    }
}
