use rgx::core;
use rgx::core::*;
use rgx::kit::ZDepth;
use rgx::math::{Matrix4, Vector2, Vector3};

pub struct Pipeline {
    pipeline: core::Pipeline,

    pub cursor_binding: Option<core::BindingGroup>,
    pub framebuffer_binding: Option<core::BindingGroup>,

    ortho_buffer: core::UniformBuffer,
    ortho_binding: core::BindingGroup,
}

impl<'a> AbstractPipeline<'a> for Pipeline {
    type PrepareContext = Matrix4<f32>;
    type Uniforms = Matrix4<f32>;

    fn description() -> PipelineDescription<'a> {
        core::PipelineDescription {
            vertex_layout: &[VertexFormat::Float3, VertexFormat::Float2],
            pipeline_layout: &[
                Set(&[Binding {
                    // Ortho matrix.
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
                Set(&[
                    // Cursor texture.
                    Binding {
                        binding: BindingType::SampledTexture,
                        stage: ShaderStage::Fragment,
                    },
                    Binding {
                        binding: BindingType::Sampler,
                        stage: ShaderStage::Fragment,
                    },
                ]),
                Set(&[
                    // Screen framebuffer.
                    Binding {
                        binding: BindingType::SampledTexture,
                        stage: ShaderStage::Fragment,
                    },
                ]),
            ],
            // TODO: Use `env("CARGO_MANIFEST_DIR")`
            vertex_shader: include_bytes!("data/cursor.vert.spv"),
            fragment_shader: include_bytes!("data/cursor.frag.spv"),
        }
    }

    fn setup(pipeline: core::Pipeline, dev: &core::Device) -> Self {
        let m: Matrix4<f32> = Matrix4::identity();
        let ortho_buffer = dev.create_uniform_buffer(&[m]);
        let ortho_binding = dev.create_binding_group(&pipeline.layout.sets[0], &[&ortho_buffer]);
        let framebuffer_binding = None;
        let cursor_binding = None;

        Self {
            pipeline,
            ortho_buffer,
            ortho_binding,
            framebuffer_binding,
            cursor_binding,
        }
    }

    fn apply(&self, pass: &mut Pass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_binding(&self.ortho_binding, &[]);
    }

    fn prepare(
        &'a self,
        ortho: Matrix4<f32>,
    ) -> Option<(&'a core::UniformBuffer, Vec<Matrix4<f32>>)> {
        Some((&self.ortho_buffer, vec![ortho]))
    }
}

impl Pipeline {
    pub fn set_cursor(&mut self, texture: &Texture, sampler: &Sampler, r: &Renderer) {
        self.cursor_binding = Some(
            r.device
                .create_binding_group(&self.pipeline.layout.sets[1], &[texture, sampler]),
        );
    }

    pub fn set_framebuffer(&mut self, fb: &Framebuffer, r: &Renderer) {
        self.framebuffer_binding = Some(
            r.device
                .create_binding_group(&self.pipeline.layout.sets[2], &[fb]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex(Vector3<f32>, Vector2<f32>);

pub struct Sprite {
    w: u32,
    h: u32,
    buf: Vec<Vertex>,
}

impl Sprite {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            buf: Vec::with_capacity(6),
        }
    }

    pub fn set(&mut self, src: Rect<f32>, dst: Rect<f32>, z: ZDepth) {
        let ZDepth(z) = z;

        // Relative texture coordinates
        let rx1: f32 = src.x1 / self.w as f32;
        let ry1: f32 = src.y1 / self.h as f32;
        let rx2: f32 = src.x2 / self.w as f32;
        let ry2: f32 = src.y2 / self.h as f32;

        self.buf.extend_from_slice(&[
            Vertex(Vector3::new(dst.x1, dst.y1, z), Vector2::new(rx1, ry2)),
            Vertex(Vector3::new(dst.x2, dst.y1, z), Vector2::new(rx2, ry2)),
            Vertex(Vector3::new(dst.x2, dst.y2, z), Vector2::new(rx2, ry1)),
            Vertex(Vector3::new(dst.x1, dst.y1, z), Vector2::new(rx1, ry2)),
            Vertex(Vector3::new(dst.x1, dst.y2, z), Vector2::new(rx1, ry1)),
            Vertex(Vector3::new(dst.x2, dst.y2, z), Vector2::new(rx2, ry1)),
        ]);
    }

    pub fn finish(self, r: &Renderer) -> core::VertexBuffer {
        r.device.create_buffer(self.buf.as_slice())
    }
}