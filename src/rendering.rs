use bodies::BodyBuffers;
use bytemuck::Zeroable;
use cgmath::Point2;
use cgmath::prelude::*;
use generic::push_line;
use generic::Mesh;
use quadtree::generate_quadtree_mesh;
use wgpu::util::{
    self,
    BufferInitDescriptor,
    DeviceExt,
};
use wgpu::{
    Buffer,
    BufferAddress,
    BufferDescriptor,
    BufferUsages,
    Color,
    Device,
    IndexFormat,
    LoadOp,
    Operations,
    RenderPassColorAttachment,
    RenderPassDescriptor,
    StoreOp,
    SurfaceError,
    TextureViewDescriptor,
    VertexAttribute,
    VertexBufferLayout,
    VertexFormat,
    VertexStepMode,
};

use crate::pipeline::Pipeline;
use crate::simulation::Body;
use crate::simulation::quadtree::Quadtree;
use crate::simulation::Simulation;

pub mod bodies;
pub mod generic;
mod quadtree;

pub struct RenderState {
    body_buffers: BodyBuffers,
}

impl RenderState {
    pub fn new(
        device: &Device,
        num_instances: usize,
    ) -> Self {
        let body_buffers = BodyBuffers::new(device, num_instances);

        Self { body_buffers }
    }

    pub fn render(
        &mut self,
        pipeline: &mut Pipeline,
        simulation: &Simulation,
    ) -> Result<(), SurfaceError> {
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
                        r: 0.001,
                        g: 0.001,
                        b: 0.002,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.render_bodies(pipeline, &mut render_pass, simulation.bodies())?;

        let quadtree_mesh = generate_quadtree_mesh(simulation.quadtree());
        self.render_generic(pipeline, &mut render_pass, &quadtree_mesh.vertices, &quadtree_mesh.indices)?;

        drop(render_pass);
        pipeline.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
