use std::sync::Arc;
use crate::{instance, model};
use crate::draw_traits::{DrawWithMaterial, DrawWithoutMaterial, DrawMethod};

pub struct RenderableObject {
  pub model: Arc<model::Model>,
  pub instances: Vec<instance::Instance>,
  pub instance_buffer: wgpu::Buffer,
  pub pipeline_name: Option<String>,
  pub draw_method: DrawMethod,
}

impl RenderableObject {
  pub fn new(
    device: &wgpu::Device,
    model: Arc<model::Model>,
    instances: Vec<instance::Instance>,
    pipeline_name: Option<String>,
    draw_method: DrawMethod,
  ) -> Self {
    use wgpu::util::DeviceExt;
    
    let instance_data = instances
      .iter()
      .map(instance::Instance::to_raw)
      .collect::<Vec<_>>();
    
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    Self {
      model,
      instances,
      instance_buffer,
      pipeline_name,
      draw_method,
    }
  }

  pub fn draw<'a>(
    &'a self,
    render_pass: &mut wgpu::RenderPass<'a>,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  ) {
    let instances = 0..self.instances.len() as u32;
    match self.draw_method {
      DrawMethod::WithMaterial => {
        DrawWithMaterial::draw_model_instanced(
          render_pass,
          &self.model,
          instances,
          camera_bind_group,
          light_bind_group,
        );
      }
      DrawMethod::WithoutMaterial => {
        DrawWithoutMaterial::draw_model_instanced(
          render_pass,
          &self.model,
          instances,
          camera_bind_group,
          light_bind_group,
        );
      }
    }
  }

  #[allow(dead_code)]
  pub fn update_instances(&mut self, queue: &wgpu::Queue) {
    let instance_data = self.instances
      .iter()
      .map(instance::Instance::to_raw)
      .collect::<Vec<_>>();
    queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
  }
}
