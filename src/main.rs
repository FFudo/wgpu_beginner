use glfw::{fail_on_errors, Action, Context, Key, Window};
mod renderer_backend;
use renderer_backend::{bind_group_layout, material::Material, mesh_builder, pipeline};

struct State<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (i32, i32),
    window: &'a mut Window,
    render_pipeline: wgpu::RenderPipeline,
    triangle_mesh: wgpu::Buffer,
    quad_mesh: mesh_builder::Mesh,
    triangle_material: Material,
    quad_material: Material,
}

impl<'a> State<'a> {
    async fn new(window: &'a mut Window) -> Self {
        let size = window.get_framebuffer_size();
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);

        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }.unwrap();
        let surface = unsafe { instance.create_surface_unsafe(target) }.unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            memory_hints: wgpu::MemoryHints::Performance,
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let triangle_mesh = mesh_builder::make_triangle(&device);
        let quad_mesh = mesh_builder::make_quad(&device);

        let material_bind_group_layout: wgpu::BindGroupLayout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("Material Bind Group Layout");
        }

        let render_pipeline: wgpu::RenderPipeline;
        {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("shaders/shader.wgsl", "vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_bind_group_layout(&material_bind_group_layout);
            render_pipeline = builder.build_pipeline("Render Pipeline");
        }

        let triangle_material = Material::new(
            "img/satin.jpg",
            &device,
            &queue,
            &material_bind_group_layout,
        );
        let quad_material = Material::new(
            "img/grunge.jpg",
            &device,
            &queue,
            &material_bind_group_layout,
        );

        Self {
            instance,
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            triangle_mesh,
            quad_mesh,
            triangle_material,
            quad_material,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let drawable = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };

        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.2,
                    g: 0.4,
                    b: 0.6,
                    a: 0.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        {
            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            renderpass.set_pipeline(&self.render_pipeline);

            renderpass.set_bind_group(0, &self.quad_material.bind_group, &[]);
            renderpass.set_vertex_buffer(0, self.quad_mesh.vertex_buffer.slice(..));
            renderpass.set_index_buffer(
                self.quad_mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            renderpass.draw_indexed(0..6, 0, 0..1);

            renderpass.set_bind_group(0, &self.triangle_material.bind_group, &[]);
            renderpass.set_vertex_buffer(0, self.triangle_mesh.slice(..));
            renderpass.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(command_encoder.finish()));

        drawable.present();

        Ok(())
    }

    fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0 as u32;
            self.config.height = new_size.1 as u32;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

async fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    let (mut window, events) = glfw
        .create_window(800, 600, "It's Graphics Time", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_key_polling(true); //set to all polling if you want  all events
    window.set_size_polling(true);

    window.make_current();

    let mut state = State::new(&mut window).await;

    while !state.window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    state.window.set_should_close(true);
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    state.resize((width, height));
                }
                _ => {}
            }
        }

        match state.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.resize(state.size);
            }
            Err(e) => eprint!("{:?}", e),
        }
        state.window.swap_buffers();
    }
}

fn main() {
    pollster::block_on(run())
}
