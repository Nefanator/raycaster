use crate::{
    game,
    primitives::{CricleDescriptor, LineDescriptor, VerticalLineDescriptor},
};

use std::{
    cmp::{max, min},
    f32::consts::PI,
    num::NonZeroU32,
    ops::Neg,
};

use glam::{Mat3, Vec2, Vec3, Vec3Swizzles};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::game::GameState;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    canvas: Vec<u8>,
    render_map: bool,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });

        let surface = unsafe { instance.create_surface(window).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|item| item.describe().srgb)
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            canvas: vec![0; (size.width * size.height * 4) as usize],
            render_map: true,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.queue.write_texture(
            output.texture.as_image_copy(),
            &self.canvas,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * self.size.width),
                rows_per_image: NonZeroU32::new(self.size.height),
            },
            output.texture.size(),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: false,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update(&mut self, game_state: &GameState) {
        let size = self.size;
        let pixels = (size.height * size.width) as usize;
        let mut pixel_offset = 0;

        // Clear Screen
        loop {
            // let x = (pixel_offset % size.width as usize) as f32 / size.width as f32;
            // let y = (size.height as usize - (pixel_offset / size.width as usize)) as f32
            //     / size.height as f32;

            let colour: u32 = 0; // per_pixel(x, y);

            // let pixel_offset = ((y * self.size.width + x) * 4) as usize;

            // todo: check the color format first
            let rgba_offset = pixel_offset * 4;
            self.canvas[rgba_offset + 0] = (colour >> 24) as u8;
            self.canvas[rgba_offset + 1] = (colour >> 16) as u8;
            self.canvas[rgba_offset + 2] = (colour >> 8) as u8;
            self.canvas[rgba_offset + 3] = colour as u8;

            pixel_offset += 1;
            if pixel_offset == pixels {
                break;
            }
        }
        if !self.render_map {
            self.update_map(game_state);
        } else {
            self.update_scene(game_state);
        }
    }

    fn update_map(&mut self, game_state: &GameState) {
        // Draw Level
        for sector in game_state.level().sectors() {
            for wall in sector.walls() {
                self.draw_line(&LineDescriptor {
                    start: wall.0,
                    end: wall.1,
                    color: Vec3::splat(1.0),
                    stroke: 1.0,
                });
            }
        }

        // Draw Player
        let look_at = game_state.pos() + (game_state.rot().xy() * 100.0);
        self.draw_line(&LineDescriptor {
            start: game_state.pos(),
            end: look_at,
            color: Vec3::new(1.0, 0.0, 0.0),
            stroke: 1.0,
        });
        self.draw_circle(&CricleDescriptor {
            centre: game_state.pos(),
            radius: 100.0,
            color: Vec3::splat(1.0),
        });
    }

    fn update_scene(&mut self, game_state: &GameState) {
        // we need to iterate through the walls in the scene, each one needs to be transformed into user-space
        //  First create the quaternion that will transform the wall points

        let transform = create_transform(game_state.pos(), game_state.rot());

        // for wall in game_state.level().walls() {
        //     self.draw_line(&LineDescriptor {
        //         start: transform.transform_point2(wall[0]),
        //         end: transform.transform_point2(wall[1]),
        //         color: Vec3::splat(1.0),
        //         stroke: 1.0,
        //     });
        // }

        let fov_y = PI / 2.0;
        let render_distance = 2.0;
        let half_canvas_height = self.size.height as f32 / 2.0;

        if let Some(current_sector) = game_state.find_current_sector() {
            for y in 0..(self.size.width) {
                let ray_angle = -((y as f32 / self.size.width as f32) * 2.0 - 1.0) * fov_y / 2.0;

                let ray = Mat3::from_axis_angle(Vec3::Z, ray_angle).transform_vector2(Vec2::Y);

                for wall in current_sector.walls() {
                    let start = transform.transform_point2(wall.0);
                    let end = transform.transform_point2(wall.1);
                    if let Some(distance) = intersection_distance(Vec2::ZERO, ray, start, end) {
                        let corrected_distance = (distance * ray_angle.cos()).max(0.0);
                        if corrected_distance > render_distance {
                            continue;
                        }
                        let perceived_height = current_sector.height() / corrected_distance * 100.0;
                        let perceived_base_height =
                            current_sector.base_height() / corrected_distance * 100.0;

                        let color = wall.2 * (1.0 - (corrected_distance / render_distance)).max(0.0);

                        self.draw_vertical_line(&VerticalLineDescriptor {
                            y,
                            top_x: (half_canvas_height - perceived_height - perceived_base_height)
                                as u32,
                            bottom_x: (half_canvas_height - perceived_base_height) as u32,
                            color: color.extend(1000.0 / corrected_distance),
                        })
                    }
                }
            }
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.canvas = vec![0; (new_size.width * new_size.height * 4) as usize];
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Tab),
                        ..
                    },
                ..
            } => {
                self.toggle_renderer();
                true
            }
            _ => false,
        }
    }

    fn toggle_renderer(&mut self) {
        self.render_map = !self.render_map;
    }

    fn draw_vertical_line(&mut self, line: &VerticalLineDescriptor) {
        let mut pixel_offset = line.top_x * self.size.width + line.y;
        for _ in line.top_x..line.bottom_x {
            // loop with if would be faster
            let rgba_offset = (pixel_offset * 4) as usize;

            let red_channel = (255.0 * line.color[0]) as u8;
            let green_channel = (255.0 * line.color[1]) as u8;
            let blue_channel = (255.0 * line.color[2]) as u8;
            let alpha_channel = (255.0 * line.color[3]) as u8;

            if self.canvas[rgba_offset + 3] < alpha_channel {
                self.canvas[rgba_offset + 0] = blue_channel;
                self.canvas[rgba_offset + 1] = green_channel;
                self.canvas[rgba_offset + 2] = red_channel;
                self.canvas[rgba_offset + 3] = alpha_channel;
                pixel_offset += self.size.width;
            }
        }
    }

    fn draw_circle(&mut self, circle: &CricleDescriptor) {
        let min_y = max((circle.centre.y - circle.radius) as i32, 0) as u32;
        let max_y = min((circle.centre.y + circle.radius) as u32, self.size.height);

        let min_x = max((circle.centre.x - circle.radius) as i32, 0) as u32;
        let max_x = min((circle.centre.x + circle.radius) as u32, self.size.width);

        for y in min_y..max_y {
            for x in min_x..max_x {
                let distance = circle.centre.distance(Vec2::new(x as f32, y as f32));
                let intensity = (circle.radius - distance) / circle.radius;
                if distance < circle.radius {
                    self.plot_with_opacity(x, y, 128, 128, 128, 0.5);
                }
            }
        }
    }

    fn draw_line(&mut self, line: &LineDescriptor) {
        let start = (line.start.x as isize, line.start.y as isize);
        let end = (line.end.x as isize, line.end.y as isize);

        for (x, y) in bresenham::Bresenham::new(start, end) {
            if 0 < x && x < self.size.width as isize && 0 < y && y < self.size.height as isize {
                self.plot(x as u32, y as u32, line.color);
            }
        }
    }

    fn plot(&mut self, x: u32, y: u32, color: Vec3) {
        let pixel_offset = (x + y * self.size.width) as usize;
        let rgba_offset = pixel_offset * 4;
        self.canvas[rgba_offset + 0] = (color[2] * 255.0) as u8;
        self.canvas[rgba_offset + 1] = (color[1] * 255.0) as u8;
        self.canvas[rgba_offset + 2] = (color[0] * 255.0) as u8;
    }

    fn plot_with_opacity(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, opacity: f32) {
        let pixel_offset = (x + y * self.size.width) as usize;
        let rgba_offset = pixel_offset * 4;

        self.canvas[rgba_offset + 0] = blend(self.canvas[rgba_offset + 0], b, opacity);
        self.canvas[rgba_offset + 1] = blend(self.canvas[rgba_offset + 1], g, opacity);
        self.canvas[rgba_offset + 2] = blend(self.canvas[rgba_offset + 2], r, opacity);
    }
}

fn blend(current: u8, new: u8, opacity: f32) -> u8 {
    let current = current as f32;
    let new = new as f32 * opacity;

    let hdr_color = current + new;

    255.0_f32.min(hdr_color) as u8
}

fn pos_to_offset(pos: glam::Vec2, size: PhysicalSize<u32>) -> Option<usize> {
    if pos.x < 0.0 || pos.y < 0.0 {
        return None;
    }
    if pos.x > size.width as f32 || pos.y > size.height as f32 {
        return None;
    }

    Some((pos.x + pos.y * size.width as f32) as usize)
}

fn per_pixel(x: f32, y: f32) -> u32 {
    let r = (x * 255.0) as u8;
    let g = (y * 255.0) as u8;
    let b = 0;

    u32::from_be_bytes([b, g, r, 0])
}

fn create_transform(pos: Vec2, rot: Vec3) -> Mat3 {
    let mut angle = rot.angle_between(glam::Vec3::NEG_Y);
    let angle_from_x = rot.angle_between(glam::Vec3::X);

    if angle_from_x < PI / 2.0 {
        angle = angle.neg()
    }

    let rot_transform = glam::Mat3::from_axis_angle(Vec3::Z, angle);
    let trans_transform = glam::Mat3::from_translation(pos.neg());
    rot_transform * trans_transform
}

fn intersection_distance(origin: Vec2, direction: Vec2, start: Vec2, end: Vec2) -> Option<f32> {
    let v1 = (origin - start).extend(0.0);
    let v2 = (end - start).extend(0.0);
    let v3 = Vec3::new(direction.y, direction.x, 0.0);

    let dot = v2.dot(v3);
    if dot.abs() < 0.000001 {
        return None;
    }

    let t1 = v2.cross(v1).z / dot;
    let t2 = v1.dot(v3) / dot;

    if t1 >= 0.0 && (t2 >= 0.0 && t2 <= 1.0) {
        return Some(t1);
    }

    None
}
