use bytemuck::Zeroable;
use cgmath::prelude::*;
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, Device, IndexFormat, LoadOp,
    Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceError,
    TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
    util::{self, BufferInitDescriptor, DeviceExt},
};

use crate::{pipeline::Pipeline, simulation::Body};

const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5],
    },
    Vertex {
        position: [-0.5, -0.5],
    },
    Vertex {
        position: [0.5, -0.5],
    },
    Vertex {
        position: [0.5, 0.5],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 3, 1, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
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

pub struct RenderState {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    instance_buffer: Buffer,
    // must be the same for every render call
    num_instances: u32,
    instances: Vec<BodyInstance>,
}

impl RenderState {
    pub fn new(device: &Device, num_instances: usize) -> Self {
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

    pub fn render_bodies(
        &mut self,
        pipeline: &mut Pipeline,
        bodies: &Vec<Body>,
    ) -> Result<(), SurfaceError> {
        assert!(
            bodies.len() == self.num_instances as usize,
            "Number of bodies must not change across rendering calls"
        );

        let output = pipeline.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // create command encoder and render pass
        let mut encoder = pipeline.start_encoder();
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&pipeline.render_pipeline);

        // update instance buffer
        for (instance, body) in self.instances.iter_mut().zip(bodies.iter()) {
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
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );

        // set all buffers
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

        // draw calls
        render_pass.draw_indexed(0..self.num_indices, 0, 0..self.num_instances);

        // finish
        drop(render_pass);
        pipeline.finish_encoder(encoder);
        output.present();

        Ok(())
    }
}
