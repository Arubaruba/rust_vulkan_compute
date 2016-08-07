#[macro_use]
extern crate vulkano;

use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer;
use vulkano::command_buffer::PrimaryCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::pipeline::ComputePipeline;

use std::sync::Arc;
use std::time::Duration;

fn main() {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .expect("failed to create Vulkan instance");

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

    // Some little debug infos.
    println!("Using device: {} (type: {:?})",
             physical.name(),
             physical.ty());

    let queue = physical.queue_families()
        .find(|q| q.supports_compute())
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

    let queue = queues.next().unwrap();

    mod cs {
        include!{concat!(env!("OUT_DIR"), "/shaders/src/compute.glsl")}
    }
    let cs = cs::Shader::load(&device).expect("failed to create shader module");

    let storage_buffer = CpuAccessibleBuffer::from_data(&device,
                                                        &vulkano::buffer::BufferUsage::all(),
                                                        Some(queue.family()),
                                                        1234usize)
        .expect("failed to create buffer");


    let descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);
    mod pipeline_layout {
        pipeline_layout!{
            set0: {
                asdf: StorageBuffer<usize>
            }
        }
    }

    println!("initial buffer val: {}",
             *storage_buffer.read(Duration::new(1, 0)).unwrap());

    let pipeline_layout = pipeline_layout::CustomPipeline::new(&device).unwrap();
    let set = pipeline_layout::set0::Set::new(&descriptor_pool,
                                              &pipeline_layout,
                                              &pipeline_layout::set0::Descriptors {
                                                  asdf: &storage_buffer,
                                              });

    let pipeline = ComputePipeline::new(&device, &pipeline_layout, &cs.main_entry_point(), &())
        .unwrap();

    let command_buffer = PrimaryCommandBufferBuilder::new(&device, queue.family())
        .dispatch(&pipeline, &set, [2, 1, 1], &())
        .build();

    command_buffer::submit(&command_buffer, &queue).unwrap().wait(Duration::new(1, 0)).unwrap();

    println!("final buffer val: {}",
             *storage_buffer.read(Duration::new(1, 0)).unwrap());
}