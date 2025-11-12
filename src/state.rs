use std::{iter, sync::Arc};
use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use winit::{event::*, event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{
  camera, instance, light, model, resources, texture, uniforms, pipeline_manager,
};

use crate::model::Vertex;
use crate::draw_traits::DrawMethod;
use crate::renderable_object::RenderableObject;

pub struct State {
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pipeline_manager: pipeline_manager::PipelineManager,

    objects: Vec<RenderableObject>,

    camera: camera::Camera,
    projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    camera_uniform: uniforms::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    light_uniform: light::LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,

    depth_texture: texture::Texture,
    is_surface_configured: bool,

    pub mouse_pressed: bool,
}

impl State {
  pub async fn new(window: Arc<Window>) -> anyhow::Result<State> {
    let size = window.inner_size();

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
      #[cfg(not(target_arch = "wasm32"))]
      backends: wgpu::Backends::PRIMARY,
      #[cfg(target_arch = "wasm32")]
      backends: wgpu::Backends::GL,
      ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        required_limits: if cfg!(target_arch = "wasm32") {
          wgpu::Limits::downlevel_webgl2_defaults()
        } else {
          wgpu::Limits::default()
        },
        memory_hints: Default::default(),
        trace: wgpu::Trace::Off,
      })
      .await?;

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
      .formats
      .iter()
      .copied()
      .find(|f| f.is_srgb())
      .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      desired_maximum_frame_latency: 2,
      view_formats: vec![],
    };

    surface.configure(&device, &config);

    let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 3,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
      label: Some("texture_bind_group_layout"),
    });

    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }
      ],
      label: Some("camera_bind_group_layout"),
    });

    let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
      label: Some("light_bind_group_layout"),
    });

    let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
    let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
    let camera_controller = camera::CameraController::new(4.0, 0.4);

    let mut camera_uniform = uniforms::CameraUniform::new();
    camera_uniform.update_view_proj(&camera, &projection);

    let camera_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      }
    );

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: camera_buffer.as_entire_binding(),
        }
      ],
      label: Some("camera_bind_group"),
    });

    let light_uniform = light::LightUniform::new(
      [2.0, 2.0, 2.0], 
      [1.0, 1.0, 1.0],
    );

    let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Light Buffer"),
      contents: bytemuck::cast_slice(&[light_uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &light_bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: light_buffer.as_entire_binding(),
      }],
      label: Some("light_bind_group"),
    });

    let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

    let mut pipeline_manager = pipeline_manager::PipelineManager::new();

    pipeline_manager.add_pipeline(
      &device,
      "main_pipeline".to_string(),
      include_str!("../shaders/shader.wgsl"),
      &[
        &texture_bind_group_layout,
        &camera_bind_group_layout,
        &light_bind_group_layout,
      ],
      &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
      config.format,
    );

    pipeline_manager.add_pipeline(
      &device,
      "light_pipeline".to_string(),
      include_str!("../shaders/light.wgsl"),
      &[
        &camera_bind_group_layout,
        &light_bind_group_layout,
      ],
      &[model::ModelVertex::desc()],
      config.format,
    );

    let obj_model = Arc::new(
      resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
        .await
        .unwrap()
    );

    let instances = instance::create_instances();

    // Create light object with single instance
    let light_instances = vec![instance::Instance {
      position: cgmath::Vector3::new(0.0, 0.0, 0.0),
      rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
    }];

    let mut objects = Vec::new();
    
    // Add main objects
    objects.push(RenderableObject::new(
      &device,
      obj_model.clone(),
      instances,
      None,
      DrawMethod::WithMaterial,
    ));
    
    // Add light object using light_pipeline
    objects.push(RenderableObject::new(
      &device,
      obj_model,
      light_instances,
      Some("light_pipeline".to_string()),
      DrawMethod::WithoutMaterial,
    ));


    Ok(Self {
      window,
      surface,
      device,
      queue,
      config,
      pipeline_manager,
      objects,
      camera,
      projection,
      camera_controller,
      camera_uniform,
      camera_buffer,
      camera_bind_group,
      light_uniform,
      light_buffer,
      light_bind_group,
      depth_texture,
      is_surface_configured: false,
      mouse_pressed: false,
    })
  }

  pub fn add_object(&mut self, model: Arc<model::Model>, instances: Vec<instance::Instance>, pipeline_name: Option<String>, draw_method: DrawMethod) {
    self.objects.push(RenderableObject::new(
      &self.device,
      model,
      instances,
      pipeline_name,
      draw_method,
    ));
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
      self.config.width = width;
      self.config.height = height;
      self.is_surface_configured = true;
      self.projection.resize(self.config.width, self.config.height);
      self.surface.configure(&self.device, &self.config);
      self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }
  }

  pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
    if !self.camera_controller.handle_key(key, pressed) {
      match (key, pressed) {
        (KeyCode::Escape, true) => event_loop.exit(),
        _ => {},
      }
    }
  }

  pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
    match button {
      MouseButton::Left => {
        self.mouse_pressed = pressed;
      }
      _ => {}
    }
  }

  pub fn handle_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
      self.camera_controller.handle_mouse_scroll(delta);
  }

  pub fn update(&mut self, dt: instant::Duration) {
    self.camera_controller.update_camera(&mut self.camera, dt);
    self.camera_uniform.update_view_proj(&self.camera, &self.projection);
    self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

    let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
    self.light_uniform.position = 
      (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32()))
        * old_position)
      .into();
    self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.window.request_redraw();

    if !self.is_surface_configured {
      return Ok(());
    }

    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.1,
              g: 0.2,
              b: 0.3,
              a: 1.0,
            }),
            store: wgpu::StoreOp::Store,
          },
          depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view: &self.depth_texture.view,
          depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
          }),
          stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      // Render all objects - draw method is encapsulated in the object
      for obj in &self.objects {
        let pipeline_name = obj.pipeline_name.as_deref().unwrap_or("main_pipeline");
        if let Some(pipeline) = self.pipeline_manager.get_by_name(pipeline_name) {
          render_pass.set_pipeline(pipeline);
          render_pass.set_vertex_buffer(1, obj.instance_buffer.slice(..));
          obj.draw(&mut render_pass, &self.camera_bind_group, &self.light_bind_group);
        }
      }
    }

    self.queue.submit(iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}
