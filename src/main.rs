#[macro_use]
extern crate vulkano;

use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer;
use vulkano::command_buffer::PrimaryCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::pipeline::ComputePipeline; 
use vulkano::instance::Limits;

use std::time::Duration;

mod cs {
    include!{concat!(env!("OUT_DIR"), "/shaders/src/compute.glsl")}
}

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

    println!("Max Compute Workers: {:?}", physical.limits().max_compute_work_group_count());
    println!("Max Compute Group Size: {:?}", physical.limits().max_compute_work_group_size());

    // Enumerating memory heaps.
    for heap in physical.memory_heaps() {
        println!("Heap #{:?} has a capacity of {:?} bytes",
                 heap.id(),
                 heap.size());
    }

    let queue = queues.next().unwrap();

    let storage_buffer = CpuAccessibleBuffer::<cs::ty::Data>::from_data(&device,
                                                        &vulkano::buffer::BufferUsage::all(),
                                                        Some(queue.family()),
                                                        cs::ty::Data {asdf: 12i32, arr: [11i32, 22i32]})
        .expect("failed to create buffer");


    let descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);
    mod pipeline_layout {
        pipeline_layout!{
            set0: {
                buffer: StorageBuffer<::cs::ty::Data>  
            }
        }
    }

    let cs = cs::Shader::load(&device).expect("failed to create shader module");

    println!("initial buffer val: {}",
             storage_buffer.read(Duration::new(1, 0)).unwrap().asdf);

    let pipeline_layout = pipeline_layout::CustomPipeline::new(&device).unwrap();
    let set = pipeline_layout::set0::Set::new(&descriptor_pool,
                                              &pipeline_layout,
                                              &pipeline_layout::set0::Descriptors {
                                                  buffer: &storage_buffer,
                                              });

    let pipeline = ComputePipeline::new(&device, &pipeline_layout, &cs.main_entry_point(), &())
        .unwrap();

    let command_buffer = PrimaryCommandBufferBuilder::new(&device, queue.family())
        .dispatch(&pipeline, &set, [10, 1, 1], &())
        .build();

    command_buffer::submit(&command_buffer, &queue).unwrap().wait(Duration::new(1, 0)).unwrap();

    println!("final buffer val: {}",
             storage_buffer.read(Duration::new(1, 0)).unwrap().asdf);

    command_buffer::submit(&command_buffer, &queue).unwrap().wait(Duration::new(1, 0)).unwrap();

    println!("final buffer val: {}",
             storage_buffer.read(Duration::new(1, 0)).unwrap().asdf);

    println!("arr buffer val: {}",
             storage_buffer.read(Duration::new(1, 0)).unwrap().arr[0]);
}