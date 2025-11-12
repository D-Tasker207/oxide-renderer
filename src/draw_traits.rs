use std::ops::Range;
use crate::model::{Mesh, Material, Model};

pub enum DrawMethod {
  WithMaterial,
  WithoutMaterial,
}

pub trait DrawWithMaterial<'a> {
  #[allow(unused)]
  fn draw_mesh(
    &mut self, 
    mesh: &'a Mesh, 
    material: &'a Material, 
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  );
  fn draw_mesh_instanced(
    &mut self, 
    mesh: &'a Mesh, 
    material: &'a Material, 
    instances: Range<u32>, 
    camera_bind_group: &'a wgpu::BindGroup, 
    light_bind_group: &'a wgpu::BindGroup,
  );

  #[allow(unused)]
  fn draw_model(
    &mut self, 
    model: &'a Model, 
    camera_bind_group: &'a wgpu::BindGroup, 
    light_bind_group: &'a wgpu::BindGroup,
  );
  fn draw_model_instanced(
    &mut self, 
    model: &'a Model, 
    instances: Range<u32>, 
    camera_bind_group: &'a wgpu::BindGroup, 
    light_bind_group: &'a wgpu::BindGroup,
  );
}

impl<'a, 'b> DrawWithMaterial<'b> for wgpu::RenderPass<'a>
where
  'b: 'a,
{
  fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, camera_bind_group: &'b wgpu::BindGroup, light_bind_group: &'b wgpu::BindGroup) {
    DrawWithMaterial::draw_mesh_instanced(self, mesh, material, 0..1, camera_bind_group, light_bind_group);
  }

  fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, material: &'b Material, instances: Range<u32>, camera_bind_group: &'b wgpu::BindGroup, light_bind_group: &'b wgpu::BindGroup) {
    self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    self.set_bind_group(0, &material.bind_group, &[]);
    self.set_bind_group(1, camera_bind_group, &[]);
    self.set_bind_group(2, light_bind_group, &[]);
    self.draw_indexed(0..mesh.num_elements, 0, instances);
  }

  fn draw_model(&mut self, model: &'b Model, camera_bind_group: &'b wgpu::BindGroup, light_bind_group: &'b wgpu::BindGroup) {
    DrawWithMaterial::draw_model_instanced(self, model, 0..1, camera_bind_group, light_bind_group);
  }

  fn draw_model_instanced(&mut self, model: &'b Model, instances: Range<u32>, camera_bind_group: &'b wgpu::BindGroup, light_bind_group: &'b wgpu::BindGroup) {
    for mesh in &model.meshes {
      let material = &model.materials[mesh.material];
      DrawWithMaterial::draw_mesh_instanced(self, mesh, material, instances.clone(), camera_bind_group, light_bind_group);
    }
  }
}

pub trait DrawWithoutMaterial<'a> {
  #[allow(unused)]
  fn draw_mesh(
    &mut self,
    mesh: &'a Mesh,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  );
  fn draw_mesh_instanced(
    &mut self,
    mesh: &'a Mesh,
    instances: Range<u32>,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  );

  fn draw_model(
    &mut self,
    model: &'a Model,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  );
  fn draw_model_instanced(
    &mut self,
    model: &'a Model,
    instances: Range<u32>,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  );
}

impl<'a, 'b> DrawWithoutMaterial<'b> for wgpu::RenderPass<'a>
where
  'b: 'a,
{
  fn draw_mesh(
      &mut self,
      mesh: &'b Mesh,
      camera_bind_group: &'b wgpu::BindGroup,
      light_bind_group: &'b wgpu::BindGroup,
    ) {
      DrawWithoutMaterial::draw_mesh_instanced(self, mesh, 0..1, camera_bind_group, light_bind_group);
  }

  fn draw_mesh_instanced(
      &mut self,
      mesh: &'b Mesh,
      instances: Range<u32>,
      camera_bind_group: &'b wgpu::BindGroup,
      light_bind_group: &'b wgpu::BindGroup,
    ) {
      self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
      self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
      self.set_bind_group(0, camera_bind_group, &[]);
      self.set_bind_group(1, light_bind_group, &[]);
      self.draw_indexed(0..mesh.num_elements, 0, instances);
  }

  fn draw_model(
      &mut self,
      model: &'b Model,
      camera_bind_group: &'b wgpu::BindGroup,
      light_bind_group: &'b wgpu::BindGroup,
    ) {
      DrawWithoutMaterial::draw_model_instanced(self, model, 0..1, camera_bind_group, light_bind_group);
  }

  fn draw_model_instanced(
      &mut self,
      model: &'b Model,
      instances: Range<u32>,
      camera_bind_group: &'b wgpu::BindGroup,
      light_bind_group: &'b wgpu::BindGroup,
    ) {
      for mesh in &model.meshes {
        DrawWithoutMaterial::draw_mesh_instanced(self, mesh, instances.clone(), camera_bind_group, light_bind_group);
      }
  }
}
