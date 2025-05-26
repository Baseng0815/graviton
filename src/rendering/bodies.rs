use bytemuck::Zeroable;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt}, Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, CommandEncoder, Device, IndexFormat, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceError, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode
};

use crate::{pipeline::Pipeline, simulation::Body};

use super::RenderState;

const QUAD_VERTICES: &[CircleVertex] = &[
    CircleVertex {
        position: [-0.5, 0.5],
    },
    CircleVertex {
        position: [-0.5, -0.5],
    },
    CircleVertex {
        position: [0.5, -0.5],
    },
    CircleVertex {
        position: [0.5, 0.5],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 3, 1, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleVertex {
    position: [f32; 2],
}

impl CircleVertex {
    pub fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x2,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BodyInstance {
    position: [f32; 2],
    color: [f32; 4],
    radius: f32,
}

impl BodyInstance {
    pub fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<BodyInstance>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as u64,
                    shader_location: 3,
                    format: VertexFormat::Float32,
                },
            ],
        }
    }
}

pub(super) struct BodyBuffers {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    instance_buffer: Buffer,
    // must be the same for every render call
    num_instances: u32,
    instances: Vec<BodyInstance>,
}

impl BodyBuffers {
    pub(super) fn new(device: &Device, num_instances: usize) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: BufferUsages::INDEX,
        });

        let num_indices = QUAD_INDICES.len() as u32;

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: BufferAddress::try_from(num_instances * std::mem::size_of::<BodyInstance>())
                .unwrap(),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instances = vec![BodyInstance::zeroed(); num_instances];

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            instance_buffer,
            num_instances: u32::try_from(num_instances).unwrap(),
            instances,
        }
    }
}

impl RenderState {
    pub(super) fn render_bodies(
        &mut self,
        pipeline: &mut Pipeline,
        render_pass: &mut RenderPass,
        bodies: &[Body],
    ) -> Result<(), SurfaceError> {
        let bufs = &mut self.body_buffers;

        assert!(
            bodies.len() == bufs.num_instances as usize,
            "Number of bodies must not change across rendering calls"
        );

        render_pass.set_pipeline(&pipeline.circle_pipeline);

        for (instance, body) in bufs.instances.iter_mut().zip(bodies.iter()) {
            *instance = BodyInstance {
                position: [body.position.x, body.position.y],
                color: [
                    body.color.r as f32,
                    body.color.g as f32,
                    body.color.b as f32,
                    body.color.a as f32,
                ],
                radius: body.radius,
            }
        }

        pipeline.queue.write_buffer(
            &bufs.instance_buffer,
            0,
            bytemuck::cast_slice(&bufs.instances),
        );

        render_pass.set_vertex_buffer(0, bufs.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, bufs.instance_buffer.slice(..));
        render_pass.set_index_buffer(bufs.index_buffer.slice(..), IndexFormat::Uint16);

        render_pass.draw_indexed(0..bufs.num_indices, 0, 0..bufs.num_instances);

        Ok(())
    }
}
