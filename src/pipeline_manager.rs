use std::collections::HashMap;
use crate::{texture, pipeline};

pub struct PipelineManager {
  pipelines: Vec<wgpu::RenderPipeline>,
  pipeline_map: HashMap<String, usize>,
}

impl PipelineManager {
  pub fn new() -> Self {
    Self {
      pipelines: Vec::new(),
      pipeline_map: HashMap::new(),
    }
  }

  pub fn add_pipeline(
    &mut self,
    device: &wgpu::Device,
    name: String,
    shader_source: &str,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_layouts: &[wgpu::VertexBufferLayout],
    surface_format: wgpu::TextureFormat,
  ) -> usize {
    if let Some(&index) = self.pipeline_map.get(&name) {
      return index;
    }

    let render_pipeline_layout = 
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{} Pipeline Layout", name)),
        bind_group_layouts: bind_group_layouts,
        push_constant_ranges: &[],
      });

    let shader = wgpu::ShaderModuleDescriptor {
      label: Some(&format!("{} Shader", name)),
      source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    };

    let render_pipeline = pipeline::create_render_pipeline(
      device,
      &render_pipeline_layout,
      surface_format,
      Some(texture::Texture::DEPTH_FORMAT),
      vertex_layouts,
      shader,
    );

    let index = self.pipelines.len();
    self.pipelines.push(render_pipeline);
    self.pipeline_map.insert(name, index);
    index
  }


  fn get(&self, index: usize) -> Option<&wgpu::RenderPipeline> {
    self.pipelines.get(index)
  }

  pub fn get_by_name(&self, name: &str) -> Option<&wgpu::RenderPipeline> {
    self.pipeline_map.get(name).and_then(|&i| self.get(i))
  }
}

impl Default for PipelineManager {
  fn default() -> Self {
    Self::new()
  }
}