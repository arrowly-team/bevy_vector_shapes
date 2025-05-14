use bevy::{
    prelude::*,
    reflect::Reflect,
    render::render_resource::{ShaderRef, ShaderType},
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{Flags, ShapeComponent, ShapeData, LINE_HANDLE},
};

/// Component containing the data for drawing a polyline.
#[derive(Component, Reflect)]
pub struct PolylineComponent {
    pub alignment: Alignment,
    pub cap: Cap,

    /// Line strip positions to draw the polyline in world space relative to it's transform.
    pub strip: Vec<Vec3>,
}

impl PolylineComponent {
    pub fn new(config: &ShapeConfig, strip: Vec<Vec3>) -> Self {
        Self {
            alignment: config.alignment,
            cap: config.cap,

            strip,
        }
    }
}

impl Default for PolylineComponent {
    fn default() -> Self {
        Self {
            alignment: default(),
            cap: default(),

            strip: default(),
        }
    }
}

impl ShapeComponent for PolylineComponent {
    type Data = PolylineData;

    fn get_data(
        &self,
        tf: &GlobalTransform,
        fill: &ShapeFill,
    ) -> impl Iterator<Item = PolylineData> {
        let mut flags = Flags(0);
        let thickness = match fill.ty {
            FillType::Stroke(thickness, thickness_type) => {
                flags.set_thickness_type(thickness_type);
                flags.set_hollow(1);
                thickness
            }
            FillType::Fill => 1.0,
        };
        flags.set_alignment(self.alignment);
        flags.set_cap(self.cap);

        self.strip.windows(2).map(move |l| PolylineData {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: fill.color.to_linear().to_f32_array(),
            thickness,
            flags: flags.0,

            start: l[0],
            end: l[1],
        })
    }
}

/// Raw data sent to the line shader to draw a line
#[derive(Clone, Copy, Reflect, Default, Debug, ShaderType)]
#[repr(C)]
pub struct PolylineData {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    start: Vec3,
    end: Vec3,
}

impl PolylineData {
    pub fn new(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_cap(config.cap);

        PolylineData {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.to_linear().to_f32_array(),
            thickness: config.thickness,
            flags: flags.0,

            start,
            end,
        }
    }
}

impl ShapeData for PolylineData {
    type Component = PolylineComponent;

    fn vertex_layout() -> Vec<wgpu::VertexAttribute> {
        vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,

            4 => Float32x4,
            5 => Float32,
            6 => Uint32,
            7 => Float32x3,
            8 => Float32x3,
        ]
        .to_vec()
    }

    fn shader() -> ShaderRef {
        LINE_HANDLE.into()
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

/// Extension trait for [`ShapePainter`] to enable it to draw polylines.
pub trait PolylinePainter {
    fn polyline(&mut self, strip: Vec<Vec3>) -> &mut Self;
}

impl<'w, 's> PolylinePainter for ShapePainter<'w, 's> {
    fn polyline(&mut self, strip: Vec<Vec3>) -> &mut Self {
        for l in strip.windows(2) {
            self.send(PolylineData::new(self.config(), l[0], l[1]));
        }
        self
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of polyline bundles.
pub trait PolylineBundle {
    fn polyline(config: &ShapeConfig, strip: Vec<Vec3>) -> Self;
}

impl PolylineBundle for ShapeBundle<PolylineComponent> {
    fn polyline(config: &ShapeConfig, strip: Vec<Vec3>) -> Self {
        let mut bundle = Self::new(config, PolylineComponent::new(config, strip));
        bundle.fill.ty = FillType::Stroke(config.thickness, config.thickness_type);
        bundle
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of polyline entities.
pub trait PolylineSpawner<'w>: ShapeSpawner<'w> {
    fn polyline(&mut self, strip: Vec<Vec3>) -> ShapeEntityCommands;
}

impl<'w, T: ShapeSpawner<'w>> PolylineSpawner<'w> for T {
    fn polyline(&mut self, strip: Vec<Vec3>) -> ShapeEntityCommands {
        self.spawn_shape(ShapeBundle::polyline(self.config(), strip))
    }
}
