#[macro_use]
extern crate vulkano;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer;
use vulkano::command_buffer::PrimaryCommandBufferBuilder;
use vulkano::command_buffer::Submission;
use vulkano::descriptor::pipeline_layout::EmptyPipeline;
use vulkano::device::Device;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::pipeline::ComputePipeline;

use std::sync::Arc;

fn main() {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create Vulkan instance");

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

    // Some little debug infos.
    println!("Using device: {} (type: {:?})",
             physical.name(),
             physical.ty());

    let queue = physical.queue_families()
        .find(|q| q.supports_graphics())
        .expect("couldn't find a graphical queue fmy");

    let (device, mut queues) = {
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        };

        Device::new(&physical,
                    physical.supported_features(),
                    &device_ext,
                    [(queue, 0.5)].iter().cloned())
            .expect("failed to create device")
    };

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
    // example we use only one queue, so we just retreive the first and only element of the
    // iterator and throw it away.
    let queue = queues.next().unwrap();

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = {
        #[derive(Debug, Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(&device,
                                       &BufferUsage::all(),
                                       Some(queue.family()),
                                       [Vertex { position: [-0.5, -0.25] },
                                        Vertex { position: [0.0, 0.5] },
                                        Vertex { position: [0.25, -0.1] }]
                                           .iter()
                                           .cloned())
            .expect("failed to create buffer")
    };
    mod cs {
        include!{concat!(env!("OUT_DIR"), "/shaders/src/compute.glsl")}
    }
    let cs = cs::Shader::load(&device).expect("failed to create shader module");


    let pipeline = ComputePipeline::new(&device,
                                        &EmptyPipeline::new(&device).unwrap(),
                                        &cs.main_entry_point(),
                                        &());

    let mut submissions: Vec<Arc<Submission>> = Vec::new();

    submissions.retain(|s| s.destroying_would_block());

    let command_buffer = PrimaryCommandBufferBuilder::new(&device, queue.family()).build();

    submissions.push(command_buffer::submit(&command_buffer, &queue).unwrap());
}