use cgmath::{
    InnerSpace,
    Point2,
    Vector2,
};
use wgpu::util::{
    BufferInitDescriptor,
    DeviceExt,
};
use wgpu::wgc::pipeline::{
    self,
    VertexStep,
};
use wgpu::{
    BufferAddress, BufferUsages, Color, IndexFormat, RenderPass, SurfaceError, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode
};

use crate::pipeline::Pipeline;

use super::RenderState;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GenericVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl GenericVertex {
    pub fn new(
        position: [f32; 2],
        color: [f32; 4],
    ) -> Self {
        Self { position, color }
    }

    pub fn from_point_and_color(
        position: Point2<f32>,
        color: Color,
    ) -> Self {
        Self::new([position.x, position.y], [color.r as f32, color.g as f32, color.b as f32, color.a as f32])
    }

    pub fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<GenericVertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(
        vertices: Vec<GenericVertex>,
        indices: Vec<u32>,
    ) -> Self {
        Self { vertices, indices }
    }
}

impl RenderState {
    pub(super) fn render_generic(
        &mut self,
        pipeline: &mut Pipeline,
        render_pass: &mut RenderPass,
        vertices: &[GenericVertex],
        indices: &[u32],
    ) -> Result<(), SurfaceError> {
        if indices.is_empty() {
            log::warn!("Skipping rendering of empty index list");
            return Ok(());
        }

        render_pass.set_pipeline(&pipeline.generic_pipeline);

        // this is very slow. too bad!
        let vertex_buffer = pipeline.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Generic Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = pipeline.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Generic Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);

        render_pass.draw_indexed(0..u32::try_from(indices.len()).unwrap(), 0, 0..1);

        Ok(())
    }
}

pub fn push_line(
    mesh: &mut Mesh,
    from: Point2<f32>,
    to: Point2<f32>,
    width: f32,
    color: Color,
) {
    let direction = (to - from).normalize();
    let direction_perp = Vector2::new(-direction.y, direction.x) * 0.5 * width;

    let p0 = from + direction_perp;
    let p1 = from - direction_perp;
    let p2 = to - direction_perp;
    let p3 = to + direction_perp;

    let vertices = vec![
        GenericVertex::from_point_and_color(p0, color),
        GenericVertex::from_point_and_color(p1, color),
        GenericVertex::from_point_and_color(p2, color),
        GenericVertex::from_point_and_color(p3, color),
    ];

    let indices = vec![
        0, 1, 2,
        0, 2, 3
    ];

    let index_offset = u32::try_from(mesh.vertices.len()).unwrap();

    mesh.vertices.extend(vertices.into_iter());
    mesh.indices.extend(indices.into_iter().map(|index| index + index_offset));
}
