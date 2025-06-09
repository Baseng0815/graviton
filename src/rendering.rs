use bodies::BodyBuffers;
use bytemuck::Zeroable;
use cgmath::Point2;
use cgmath::prelude::*;
use generic::{
    Mesh,
    push_line,
};
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
use crate::simulation::quadtree::Quadtree;
use crate::simulation::{
    Body,
    Simulation,
};

pub mod bodies;
pub mod generic;
mod quadtree;

pub fn rgb(
    r: u8,
    g: u8,
    b: u8,
) -> Color {
    let f = 1.0 / 256.0;
    Color {
        r: r as f64 * f,
        g: g as f64 * f,
        b: b as f64 * f,
        a: 1.0,
    }
}

#[derive(Default)]
pub struct RenderSettings {
    pub draw_tree: bool,
}

impl RenderSettings {
    pub fn toggle_draw_tree(&mut self) {
        self.draw_tree = !self.draw_tree;
    }
}

pub struct RenderState {
    settings: RenderSettings,
    body_buffers: BodyBuffers,
}

impl RenderState {
    pub fn new(
        device: &Device,
        num_instances: usize,
    ) -> Self {
        let body_buffers = BodyBuffers::new(device, num_instances);

        Self {
            settings: Default::default(),
            body_buffers,
        }
    }

    pub fn settings_mut(&mut self) -> &mut RenderSettings {
        &mut self.settings
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

        if self.settings.draw_tree {
            let quadtree_mesh = generate_quadtree_mesh(simulation.quadtree());
            self.render_generic(pipeline, &mut render_pass, &quadtree_mesh.vertices, &quadtree_mesh.indices)?;
        }

        drop(render_pass);
        pipeline.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
