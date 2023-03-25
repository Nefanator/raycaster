use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::Path,
};

use glam::{Vec2, Vec3};

pub type SectorId = usize;

#[derive(Default, Serialize, Deserialize)]
pub struct LevelState {
    sectors: Vec<Sector>,
}

impl LevelState {
    pub fn demo() -> Self {
        Self {
            sectors: vec![Sector {
                points: vec![
                    Vec2::new(-1.0, -1.0),
                    Vec2::new(-1.0, 0.5),
                    Vec2::new(-0.5, 1.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(1.0, -1.0),
                ],
                lines: vec![
                    Line {
                        wall_type: Wall::Solid(Vec3::new(1.0, 0.0, 0.0)),
                        point_1_id: 0,
                        point_2_id: 1,
                    },
                    Line {
                        wall_type: Wall::Solid(Vec3::new(1.0, 1.0, 0.0)),
                        point_1_id: 1,
                        point_2_id: 2,
                    },
                    Line {
                        wall_type: Wall::Solid(Vec3::new(0.0, 1.0, 1.0)),
                        point_1_id: 2,
                        point_2_id: 3,
                    },
                    Line {
                        wall_type: Wall::Solid(Vec3::new(1.0, 0.0, 1.0)),
                        point_1_id: 3,
                        point_2_id: 4,
                    },
                    Line {
                        wall_type: Wall::Solid(Vec3::new(0.0, 0.0, 1.0)),
                        point_1_id: 4,
                        point_2_id: 0,
                    },
                ],
                base_height: 0.0,
                height: 2.5,
            }],
        }
    }

    pub fn sectors(&self) -> &Vec<Sector> {
        &self.sectors
    }

    pub fn _load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let level = serde_json::from_reader(reader)?;

        Ok(level)
    }

    pub fn _save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn find_current_sector(&self, pos: Vec2) -> Option<&Sector> {
        self.sectors().iter().find(|&sector| sector.contains(pos))
    }
}

/// A collection of lines that create a convex shape, some lines will be
/// treated as walls, others as portals. The portals will point to the next
/// sector which should be rendered in the window created by the portal. It is
/// assumed that the next sector is in the correct location, there will be no
/// checks that portals have matching dimensions.
#[derive(Debug, Serialize, Deserialize)]
pub struct Sector {
    points: Vec<Vec2>,
    lines: Vec<Line>,
    base_height: f32,
    height: f32,
}

impl Sector {
    pub fn walls(&self) -> Vec<(Vec2, Vec2, Vec3)> {
        self.lines
            .iter()
            .filter(|line| matches!(line.wall_type, Wall::Solid(_)))
            .map(|line| {
                if let Wall::Solid(color) = line.wall_type {
                    (
                        self.points[line.point_1_id],
                        self.points[line.point_2_id],
                        color,
                    )
                } else {
                    panic!() // todo: yeah...
                }
            })
            .collect()
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn base_height(&self) -> f32 {
        self.base_height
    }

    fn contains(&self, pos: Vec2) -> bool {
        for wall in self.walls() {
            let seg = (wall.0 - wall.1).extend(0.0).normalize();
            let point = (pos - wall.1).extend(0.0).normalize();

            if seg.cross(point).z < 0.0 {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    wall_type: Wall,
    point_1_id: usize,
    point_2_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Wall {
    Solid(Vec3),
    Portal(SectorId),
}
