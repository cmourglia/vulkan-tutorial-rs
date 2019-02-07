# Vulkan tutorial

Vulkan [tutorials][0] written in Rust using [Ash][1].

## Introduction

This repository will follow the structure of the original tutorial. Each 
commit will correspond to one page or on section of the page for 
long chapters.

Sometimes an 'extra' commit will be added with some refactoring or commenting.

## Commits

### 1.1.1: Base code

Application setup. We don't setup the window system now as it's done in 
the original tutorial.

### 1.1.2: Instance

Create and destroy the Vulkan instance with required surface extensions.

### 1.1.3: Validation layers

Add `VK_LAYER_LUNARG_standard_validation` at instance creation and creates
a debug report callback function after checking that it is available. 
Since we are using the `log` crate, we log the message with the proper log level.
The callback is detroyed at application termination.

### 1.1.4: Physical devices and queue families

Find a physical device with at least a queue family supporting graphics.

### 1.1.5: Logical device and queues

Create the logical device interfacing with the physical device. Then create
the graphics queue from the device.

### 1.1.extra: Refactoring and comments

- Update the readme with explanations on the structure of the repository. 
- Move validation layers related code to its own module.
- Disabled validation layers on release build.

### 1.2.1: Window surface

Create the window, the window surface and the presentation queue.
Update the physical device creation to get a device with presentation support.
At that point, the code will only work on Windows.

### 1.2.2: Swapchain

Checks for swapchain support and enable device extension for swapchain. Then
query the swapchain details and choose the right settings. Then create the 
swapchain and retrieve the swapchain images.

### 1.2.3: Image views

Create the image views to the swapchain images.

### 1.2.extra: Refactoring swapchain creation

Add `SwapchainProperties` to hold the format, present mode and extent of our swapchain.
Add a method to build the best properties to `SwapchainSupportDetails`.
Move these two struct into the `swapchain` module.

### 1.3.2: Shader module

Create the vertex and fragment shaders GLSL source and add a `compile.bat` script
to compile it into SPIR-V bytecode using `glslangValidator`.
Load the compiled SPIR-V and create a `ShaderModule` from it.

### 1.3.3: Fixed functions

This one is huge so it will be split across several commits.

#### 1.3.3.1: Vertex input and input assembly

Create the vertex input and input assembly info for the pipeline.

#### 1.3.3.2: Viewports and scissors

Create the viewport and scissor info for the pipeline.

#### 1.3.3.3: Rasterizer

Create the rasterizer info for the pipeline.

#### 1.3.3.4: Multisampling

Create the multisampling info for the pipeline.

## Run it

With validation layers:

```sh
RUST_LOG=vulkan_tutorial_ash=debug cargo run
```

or without:

```sh
RUST_LOG=vulkan_tutorial_ash=debug cargo run --release
```

[0]: https://vulkan-tutorial.com/Introduction
[1]: https://github.com/MaikKlein/ash
