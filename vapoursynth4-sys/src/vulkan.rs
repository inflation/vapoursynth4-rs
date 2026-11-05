/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! Vulkan GPU filtering API from R80's `VSVulkan4.h`.
//!
//! Keep every frame referenced by submitted work alive until completion. Shared
//! queues require their queue lock around submission. Wait for each plane's
//! producer semaphore/value before reading it, and publish a producer pair after
//! queueing writes. The exec-pool API manages these dependencies and retentions.

use std::ffi::{c_char, c_int, c_void};

use crate::{VSCore, VSFrame, VSVideoFormat, opaque_struct};
pub use ash::vk;

/// Required Vulkan version: 1.4.
pub const VS_VULKAN_API_VERSION: u32 = vk::make_api_version(0, 1, 4, 0);

/// Vulkan entry points loaded by the core for its device.
///
/// All entries are required by Vulkan 1.4 except debug-utils entries, which
/// may be null when the extension is disabled. Promoted 1.4 functions use
/// their ABI-compatible KHR pointer types from Ash.
#[allow(non_snake_case)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanFunctions {
    pub vkEnumerateInstanceVersion: Option<vk::PFN_vkEnumerateInstanceVersion>,
    pub vkEnumerateInstanceExtensionProperties:
        Option<vk::PFN_vkEnumerateInstanceExtensionProperties>,
    pub vkEnumerateInstanceLayerProperties: Option<vk::PFN_vkEnumerateInstanceLayerProperties>,
    pub vkCreateInstance: Option<vk::PFN_vkCreateInstance>,
    pub vkDestroyInstance: Option<vk::PFN_vkDestroyInstance>,
    pub vkEnumeratePhysicalDevices: Option<vk::PFN_vkEnumeratePhysicalDevices>,
    pub vkGetPhysicalDeviceProperties2: Option<vk::PFN_vkGetPhysicalDeviceProperties2>,
    pub vkGetPhysicalDeviceFeatures2: Option<vk::PFN_vkGetPhysicalDeviceFeatures2>,
    pub vkGetPhysicalDeviceMemoryProperties2: Option<vk::PFN_vkGetPhysicalDeviceMemoryProperties2>,
    pub vkGetPhysicalDeviceQueueFamilyProperties2:
        Option<vk::PFN_vkGetPhysicalDeviceQueueFamilyProperties2>,
    pub vkGetPhysicalDeviceFormatProperties2: Option<vk::PFN_vkGetPhysicalDeviceFormatProperties2>,
    pub vkGetPhysicalDeviceImageFormatProperties2:
        Option<vk::PFN_vkGetPhysicalDeviceImageFormatProperties2>,
    pub vkEnumerateDeviceExtensionProperties: Option<vk::PFN_vkEnumerateDeviceExtensionProperties>,
    pub vkCreateDevice: Option<vk::PFN_vkCreateDevice>,
    pub vkGetDeviceProcAddr: Option<vk::PFN_vkGetDeviceProcAddr>,
    /// Optional entry point; check for null before calling.
    pub vkCreateDebugUtilsMessengerEXT: Option<vk::PFN_vkCreateDebugUtilsMessengerEXT>,
    /// Optional entry point; check for null before calling.
    pub vkDestroyDebugUtilsMessengerEXT: Option<vk::PFN_vkDestroyDebugUtilsMessengerEXT>,
    pub vkDestroyDevice: Option<vk::PFN_vkDestroyDevice>,
    pub vkDeviceWaitIdle: Option<vk::PFN_vkDeviceWaitIdle>,
    pub vkGetDeviceQueue2: Option<vk::PFN_vkGetDeviceQueue2>,
    pub vkQueueSubmit2: Option<vk::PFN_vkQueueSubmit2>,
    pub vkQueueWaitIdle: Option<vk::PFN_vkQueueWaitIdle>,
    pub vkAllocateMemory: Option<vk::PFN_vkAllocateMemory>,
    pub vkFreeMemory: Option<vk::PFN_vkFreeMemory>,
    pub vkMapMemory2: Option<vk::PFN_vkMapMemory2KHR>,
    pub vkUnmapMemory2: Option<vk::PFN_vkUnmapMemory2KHR>,
    pub vkFlushMappedMemoryRanges: Option<vk::PFN_vkFlushMappedMemoryRanges>,
    pub vkInvalidateMappedMemoryRanges: Option<vk::PFN_vkInvalidateMappedMemoryRanges>,
    pub vkCreateBuffer: Option<vk::PFN_vkCreateBuffer>,
    pub vkDestroyBuffer: Option<vk::PFN_vkDestroyBuffer>,
    pub vkGetBufferMemoryRequirements2: Option<vk::PFN_vkGetBufferMemoryRequirements2>,
    pub vkBindBufferMemory2: Option<vk::PFN_vkBindBufferMemory2>,
    pub vkGetBufferDeviceAddress: Option<vk::PFN_vkGetBufferDeviceAddress>,
    pub vkGetDeviceBufferMemoryRequirements: Option<vk::PFN_vkGetDeviceBufferMemoryRequirements>,
    pub vkCreateImage: Option<vk::PFN_vkCreateImage>,
    pub vkDestroyImage: Option<vk::PFN_vkDestroyImage>,
    pub vkGetImageMemoryRequirements2: Option<vk::PFN_vkGetImageMemoryRequirements2>,
    pub vkBindImageMemory2: Option<vk::PFN_vkBindImageMemory2>,
    pub vkCreateImageView: Option<vk::PFN_vkCreateImageView>,
    pub vkDestroyImageView: Option<vk::PFN_vkDestroyImageView>,
    pub vkGetImageSubresourceLayout2: Option<vk::PFN_vkGetImageSubresourceLayout2KHR>,
    pub vkGetDeviceImageMemoryRequirements: Option<vk::PFN_vkGetDeviceImageMemoryRequirements>,
    pub vkCreateSampler: Option<vk::PFN_vkCreateSampler>,
    pub vkDestroySampler: Option<vk::PFN_vkDestroySampler>,
    pub vkCreateSamplerYcbcrConversion: Option<vk::PFN_vkCreateSamplerYcbcrConversion>,
    pub vkDestroySamplerYcbcrConversion: Option<vk::PFN_vkDestroySamplerYcbcrConversion>,
    pub vkCreateDescriptorSetLayout: Option<vk::PFN_vkCreateDescriptorSetLayout>,
    pub vkDestroyDescriptorSetLayout: Option<vk::PFN_vkDestroyDescriptorSetLayout>,
    pub vkCreateDescriptorPool: Option<vk::PFN_vkCreateDescriptorPool>,
    pub vkDestroyDescriptorPool: Option<vk::PFN_vkDestroyDescriptorPool>,
    pub vkResetDescriptorPool: Option<vk::PFN_vkResetDescriptorPool>,
    pub vkAllocateDescriptorSets: Option<vk::PFN_vkAllocateDescriptorSets>,
    pub vkFreeDescriptorSets: Option<vk::PFN_vkFreeDescriptorSets>,
    pub vkUpdateDescriptorSets: Option<vk::PFN_vkUpdateDescriptorSets>,
    pub vkCreateDescriptorUpdateTemplate: Option<vk::PFN_vkCreateDescriptorUpdateTemplate>,
    pub vkDestroyDescriptorUpdateTemplate: Option<vk::PFN_vkDestroyDescriptorUpdateTemplate>,
    pub vkUpdateDescriptorSetWithTemplate: Option<vk::PFN_vkUpdateDescriptorSetWithTemplate>,
    pub vkGetDescriptorSetLayoutSupport: Option<vk::PFN_vkGetDescriptorSetLayoutSupport>,
    pub vkCreatePipelineLayout: Option<vk::PFN_vkCreatePipelineLayout>,
    pub vkDestroyPipelineLayout: Option<vk::PFN_vkDestroyPipelineLayout>,
    pub vkCreateComputePipelines: Option<vk::PFN_vkCreateComputePipelines>,
    pub vkDestroyPipeline: Option<vk::PFN_vkDestroyPipeline>,
    pub vkCreatePipelineCache: Option<vk::PFN_vkCreatePipelineCache>,
    pub vkDestroyPipelineCache: Option<vk::PFN_vkDestroyPipelineCache>,
    pub vkGetPipelineCacheData: Option<vk::PFN_vkGetPipelineCacheData>,
    pub vkMergePipelineCaches: Option<vk::PFN_vkMergePipelineCaches>,
    pub vkCreateShaderModule: Option<vk::PFN_vkCreateShaderModule>,
    pub vkDestroyShaderModule: Option<vk::PFN_vkDestroyShaderModule>,
    pub vkCreateCommandPool: Option<vk::PFN_vkCreateCommandPool>,
    pub vkDestroyCommandPool: Option<vk::PFN_vkDestroyCommandPool>,
    pub vkResetCommandPool: Option<vk::PFN_vkResetCommandPool>,
    pub vkTrimCommandPool: Option<vk::PFN_vkTrimCommandPool>,
    pub vkAllocateCommandBuffers: Option<vk::PFN_vkAllocateCommandBuffers>,
    pub vkFreeCommandBuffers: Option<vk::PFN_vkFreeCommandBuffers>,
    pub vkBeginCommandBuffer: Option<vk::PFN_vkBeginCommandBuffer>,
    pub vkEndCommandBuffer: Option<vk::PFN_vkEndCommandBuffer>,
    pub vkResetCommandBuffer: Option<vk::PFN_vkResetCommandBuffer>,
    pub vkCmdExecuteCommands: Option<vk::PFN_vkCmdExecuteCommands>,
    pub vkCmdBindPipeline: Option<vk::PFN_vkCmdBindPipeline>,
    pub vkCmdBindDescriptorSets2: Option<vk::PFN_vkCmdBindDescriptorSets2KHR>,
    pub vkCmdPushConstants2: Option<vk::PFN_vkCmdPushConstants2KHR>,
    pub vkCmdPushDescriptorSet: Option<vk::PFN_vkCmdPushDescriptorSetKHR>,
    pub vkCmdPushDescriptorSetWithTemplate: Option<vk::PFN_vkCmdPushDescriptorSetWithTemplateKHR>,
    pub vkCmdDispatch: Option<vk::PFN_vkCmdDispatch>,
    pub vkCmdDispatchBase: Option<vk::PFN_vkCmdDispatchBase>,
    pub vkCmdDispatchIndirect: Option<vk::PFN_vkCmdDispatchIndirect>,
    pub vkCmdPipelineBarrier2: Option<vk::PFN_vkCmdPipelineBarrier2>,
    pub vkCmdCopyBuffer2: Option<vk::PFN_vkCmdCopyBuffer2>,
    pub vkCmdCopyImage2: Option<vk::PFN_vkCmdCopyImage2>,
    pub vkCmdCopyBufferToImage2: Option<vk::PFN_vkCmdCopyBufferToImage2>,
    pub vkCmdCopyImageToBuffer2: Option<vk::PFN_vkCmdCopyImageToBuffer2>,
    pub vkCmdBlitImage2: Option<vk::PFN_vkCmdBlitImage2>,
    pub vkCmdFillBuffer: Option<vk::PFN_vkCmdFillBuffer>,
    pub vkCmdUpdateBuffer: Option<vk::PFN_vkCmdUpdateBuffer>,
    pub vkCmdClearColorImage: Option<vk::PFN_vkCmdClearColorImage>,
    pub vkCreateSemaphore: Option<vk::PFN_vkCreateSemaphore>,
    pub vkDestroySemaphore: Option<vk::PFN_vkDestroySemaphore>,
    pub vkWaitSemaphores: Option<vk::PFN_vkWaitSemaphores>,
    pub vkSignalSemaphore: Option<vk::PFN_vkSignalSemaphore>,
    pub vkGetSemaphoreCounterValue: Option<vk::PFN_vkGetSemaphoreCounterValue>,
    pub vkCreateFence: Option<vk::PFN_vkCreateFence>,
    pub vkDestroyFence: Option<vk::PFN_vkDestroyFence>,
    pub vkResetFences: Option<vk::PFN_vkResetFences>,
    pub vkWaitForFences: Option<vk::PFN_vkWaitForFences>,
    pub vkGetFenceStatus: Option<vk::PFN_vkGetFenceStatus>,
    pub vkCreateEvent: Option<vk::PFN_vkCreateEvent>,
    pub vkDestroyEvent: Option<vk::PFN_vkDestroyEvent>,
    pub vkCmdSetEvent2: Option<vk::PFN_vkCmdSetEvent2>,
    pub vkCmdResetEvent2: Option<vk::PFN_vkCmdResetEvent2>,
    pub vkCmdWaitEvents2: Option<vk::PFN_vkCmdWaitEvents2>,
    pub vkCreateQueryPool: Option<vk::PFN_vkCreateQueryPool>,
    pub vkDestroyQueryPool: Option<vk::PFN_vkDestroyQueryPool>,
    pub vkGetQueryPoolResults: Option<vk::PFN_vkGetQueryPoolResults>,
    pub vkResetQueryPool: Option<vk::PFN_vkResetQueryPool>,
    pub vkCmdResetQueryPool: Option<vk::PFN_vkCmdResetQueryPool>,
    pub vkCmdWriteTimestamp2: Option<vk::PFN_vkCmdWriteTimestamp2>,
    pub vkCmdCopyQueryPoolResults: Option<vk::PFN_vkCmdCopyQueryPoolResults>,
    pub vkCmdBeginQuery: Option<vk::PFN_vkCmdBeginQuery>,
    pub vkCmdEndQuery: Option<vk::PFN_vkCmdEndQuery>,
    /// Optional entry point; check for null before calling.
    pub vkSetDebugUtilsObjectNameEXT: Option<vk::PFN_vkSetDebugUtilsObjectNameEXT>,
    /// Optional entry point; check for null before calling.
    pub vkCmdBeginDebugUtilsLabelEXT: Option<vk::PFN_vkCmdBeginDebugUtilsLabelEXT>,
    /// Optional entry point; check for null before calling.
    pub vkCmdEndDebugUtilsLabelEXT: Option<vk::PFN_vkCmdEndDebugUtilsLabelEXT>,
}

/// Scope used to resolve Vulkan function pointers.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VSVulkanLevel {
    Global = 0,
    Instance = 1,
    Device = 2,
}

/// Whether a Vulkan dispatch entry must be present.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VSVulkanRequirement {
    Required = 0,
    Optional = 1,
}

/// Queue selection. Compute accepts all commands; Transfer accepts copies only.
/// The queues may share the same non-recursive lock.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VSVulkanQueueType {
    Compute = 0,
    Transfer = 1,
}

/// Outcome of a host GPU wait. A successful wait is limited by driver reset
/// reporting; applications requiring certainty must validate the output itself.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VSGPUDrainResult {
    /// Work completed; a pool drain also completed its retention callbacks.
    Drained = 0,
    /// Completion is unknown; submitted resources must remain alive.
    Incomplete = 1,
    /// The device was lost or reset, so submitted resources may be destroyed.
    DeviceLost = 2,
}

/// Device handles and queue indices for work submitted on the core device.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanCoreHandles {
    pub instance: vk::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: vk::Device,
    /// Resolve entry points outside the provided dispatch table.
    pub get_instance_proc_addr: Option<vk::PFN_vkGetInstanceProcAddr>,
    pub compute_queue_family: u32,
    pub compute_queue_index: u32,
    pub transfer_queue_family: u32,
    pub transfer_queue_index: u32,
}

/// A GPU plane stored as a pitched linear buffer. Stride and dimensions use
/// the normal frame API. Wait for the producer pair before GPU reads.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanPlaneInfo {
    pub buffer: vk::Buffer,
    /// Size in bytes, including row padding.
    pub buffer_size: vk::DeviceSize,
    /// Wait on this semaphore and `ready_value` before reading; null means ready.
    pub ready_semaphore: vk::Semaphore,
    /// Timeline value associated with `ready_semaphore`.
    pub ready_value: u64,
}

opaque_struct!(
    /// A reference-counted producer timeline. Published planes retain it, so a
    /// filter may release its reference without waiting for consumers.
    VSGPUTimeline
);

opaque_struct!(
    /// A scratch buffer allocated and budgeted by the core.
    VSGPUBuffer
);

/// Details of scratch memory owned by a `VSGPUBuffer`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanBufferInfo {
    pub buffer: vk::Buffer,
    /// Device address; nonzero only when requested with `SHADER_DEVICE_ADDRESS`.
    pub address: vk::DeviceAddress,
    /// Persistent mapping, or null when the chosen memory is not host visible.
    pub mapped: *mut c_void,
    pub size: vk::DeviceSize,
    /// Properties of the memory type actually selected.
    pub memory_flags: vk::MemoryPropertyFlags,
}

opaque_struct!(
    /// A pooled VRAM region for resources created by the caller.
    VSGPUMemory
);

/// A suballocated region for binding caller-created Vulkan resources.
/// Bind at `offset`; never free the shared `memory` handle directly.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanMemoryInfo {
    pub memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
    /// Persistent mapping, or null when the chosen memory is not host visible.
    pub mapped: *mut c_void,
    /// Properties of the memory type actually selected.
    pub memory_flags: vk::MemoryPropertyFlags,
}

opaque_struct!(
    /// A budget reservation for memory allocated outside the core; owns no memory.
    VSGPUMemoryReservation
);

/// An exported plane allocation. Cache imports by `memory_id`, which is never
/// reused, and import the whole `memory_size` at `memory_type_index`. Each call
/// creates a new handle: close NT handles after import; successful POSIX imports
/// consume their fd. Hold the frame and synchronize access through producer
/// semaphores or `waitGPUFrame` before using imported memory.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanExportedMemory {
    /// Stable allocation identity; never reused during the process lifetime.
    pub memory_id: u64,
    /// Full allocation size, required unchanged when importing into Vulkan.
    pub memory_size: vk::DeviceSize,
    /// Original memory type index, required unchanged for Vulkan imports.
    pub memory_type_index: u32,
    pub offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
    /// External-handle type, using the corresponding Vulkan flag-bit value.
    pub handle_type: c_int,
    /// Owned NT handle on Windows, or file descriptor on POSIX.
    pub handle: isize,
}

opaque_struct!(
    /// A per-filter execution pool with a timeline and reusable command buffers.
    VSGPUExecPool
);

opaque_struct!(
    /// A recording owned by the acquiring thread until submit or abandon.
    VSGPUExecContext
);

/// Callback invoked when retained GPU work releases an object.
pub type VSGPUReleaseFunc = Option<unsafe extern "system-unwind" fn(object: *mut c_void)>;

opaque_struct!(
    /// Compiled SPIR-V words, valid independently of the originating core.
    VSGPUShader
);

/// Language accepted by the runtime compute shader compiler.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VSGPUShaderLanguage {
    /// GLSL 460 compute, targeting Vulkan 1.4 and SPIR-V 1.6.
    GLSL = 0,
}

/// An exported timeline semaphore. Each export creates a new owned OS handle.
/// Cache imports by the semaphore while holding its frame. Close NT handles
/// after importing; a successful POSIX import consumes its fd.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanExportedSemaphore {
    /// External-handle type, using the corresponding Vulkan flag-bit value.
    pub handle_type: c_int,
    /// Owned NT handle on Windows, or file descriptor on POSIX.
    pub handle: isize,
}

/// Device identity, current memory use, export capabilities and budget.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanCoreInfo {
    pub device_name: [c_char; 256],
    /// Largest device-local heap, in bytes.
    pub device_memory: i64,
    /// Driver-reported memory budget available to this process.
    pub budget: i64,
    /// Current core VRAM use, including buffers, reservations and device-local staging.
    pub allocated: i64,
    /// VRAM eviction limit controlled by setMaxVRAMUse.
    pub limit: i64,
    pub device_uuid: [u8; 16],
    pub device_luid: [u8; 8],
    pub device_node_mask: u32,
    /// Nonzero when the device LUID may be used for adapter matching.
    pub device_luid_valid: c_int,
    /// Memory export type, or zero if plane export is unavailable.
    pub export_handle_type: c_int,
    /// Semaphore export type, or zero if export is unavailable.
    pub semaphore_export_handle_type: c_int,
    /// Nonzero for shared host/device memory; the VRAM limit is capped alongside host use.
    pub unified_memory: c_int,
}

/// An enumerated device. Its position is the index accepted by setVulkanDevice.
/// Unusable devices are included with an explanatory reason.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VSVulkanDeviceListEntry {
    pub device_name: [c_char; 256],
    pub api_version: u32,
    pub device_type: c_int,
    /// Largest device-local heap, in bytes.
    pub device_memory: i64,
    /// Nonzero if this device meets the Vulkan 1.4 and required-feature baseline.
    pub usable: c_int,
    /// Explanation of why the device cannot be selected.
    pub unusable_reason: [c_char; 256],
    pub device_uuid: [u8; 16],
    pub device_luid: [u8; 8],
    pub device_node_mask: u32,
    /// Nonzero when the device LUID may be used for adapter matching.
    pub device_luid_valid: c_int,
}

/// GPU API table obtained from [`crate::VSAPI::getVulkanAPI`].
///
/// Status functions return zero on success. Pointer returns are null on failure.
/// Error buffers receive the reason when provided. This table follows the core
/// API version; its layout must match the R80 header exactly.
#[allow(non_snake_case)]
#[repr(C)]
pub struct VSVULKANAPI {
    /// Lists devices using a temporary instance without a core. Returns the total
    /// count, even when greater than `maxEntries`, or -1 on error. A null entries
    /// pointer with zero capacity counts devices only.
    pub enumerateVulkanDevices: unsafe extern "system-unwind" fn(
        entries: *mut VSVulkanDeviceListEntry,
        maxEntries: c_int,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Selects a device before its first use on this core. Pass -1 for automatic
    /// selection of the most powerful device.
    pub setVulkanDevice: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        deviceIndex: c_int,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Initializes the device if necessary and fills its identity and VRAM budget.
    pub getVulkanCoreInfo: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        info: *mut VSVulkanCoreInfo,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Sets the VRAM eviction limit and returns the new limit, in bytes.
    pub setMaxVRAMUse: unsafe extern "system-unwind" fn(bytes: i64, core: *mut VSCore) -> i64,
    /// Initializes the device if necessary and obtains its handles and queue indices.
    pub getVulkanHandles: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        handles: *mut VSVulkanCoreHandles,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Returns the loaded Vulkan dispatch table, valid for the core lifetime.
    /// Initializes the device on first use. Resolve other functions through
    /// `get_instance_proc_addr` in `VSVulkanCoreHandles`.
    pub getVulkanFunctions: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> *const VSVulkanFunctions,
    /// Locks the selected shared queue for submission. Use a `VSVulkanQueueType`
    /// value. Hold this leaf lock only around Vulkan submission; call no core API
    /// inside it. Never lock both queues: they may be the same lock.
    pub lockVulkanQueue: unsafe extern "system-unwind" fn(core: *mut VSCore, queue: c_int),
    /// Releases the queue lock acquired with `lockVulkanQueue`.
    pub unlockVulkanQueue: unsafe extern "system-unwind" fn(core: *mut VSCore, queue: c_int),
    /// Allocates GPU output with the semantics of newVideoFrame. Returns null
    /// when no device is available. newVideoFrame2 also supports GPU planes.
    pub newGPUVideoFrame: unsafe extern "system-unwind" fn(
        format: *const VSVideoFormat,
        width: c_int,
        height: c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    /// Fills a plane buffer and producer pair. Fails for CPU frames or absent planes.
    pub getGPUPlane: unsafe extern "system-unwind" fn(
        frame: *const VSFrame,
        plane: c_int,
        info: *mut VSVulkanPlaneInfo,
    ) -> c_int,
    /// Publishes a producer timeline/value and retains its timeline. Null marks
    /// host-ready content. The plane must be unshared. Queue the signal before
    /// publishing it; a signal requiring future host work can deadlock the core.
    pub setGPUPlaneProducer: unsafe extern "system-unwind" fn(
        frame: *mut VSFrame,
        plane: c_int,
        timeline: *mut VSGPUTimeline,
        value: u64,
    ),
    /// Creates an exportable timeline with value zero and one owned reference.
    pub createGPUTimeline: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> *mut VSGPUTimeline,
    /// Releases one timeline reference. Published planes retain their own references.
    pub freeGPUTimeline: unsafe extern "system-unwind" fn(timeline: *mut VSGPUTimeline),
    /// Adds one reference, requiring a matching freeGPUTimeline.
    pub addGPUTimelineRef: unsafe extern "system-unwind" fn(timeline: *mut VSGPUTimeline),
    /// Borrows the raw semaphore for the lifetime of a timeline reference.
    pub getGPUTimelineSemaphore:
        unsafe extern "system-unwind" fn(timeline: *mut VSGPUTimeline) -> vk::Semaphore,
    /// Waits for a timeline value and returns a `VSGPUDrainResult`, distinguishing
    /// completion from detected device loss. Prefer this to a bare Vulkan wait.
    /// An exec-pool timeline value must already have been submitted.
    pub gpuTimelineWaitValue: unsafe extern "system-unwind" fn(
        timeline: *mut VSGPUTimeline,
        value: u64,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Allocates scratch VRAM counted against the core budget. Host-visible
    /// allocations are persistently mapped. Destroy only after GPU use completes,
    /// or transfer ownership to gpuExecUsesBuffer.
    pub createGPUBuffer: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        requiredFlags: vk::MemoryPropertyFlags,
        preferredFlags: vk::MemoryPropertyFlags,
        info: *mut VSVulkanBufferInfo,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> *mut VSGPUBuffer,
    /// Destroys a scratch buffer after its submitted uses have completed.
    pub destroyGPUBuffer: unsafe extern "system-unwind" fn(buffer: *mut VSGPUBuffer),
    /// Suballocates pooled memory matching Vulkan requirements. Bind resources
    /// at the returned offset. External-memory resources and resources requiring
    /// dedicated allocation cannot use this pool. Free only after GPU use ends.
    pub allocateGPUMemory: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        requirements: *const vk::MemoryRequirements,
        requiredFlags: vk::MemoryPropertyFlags,
        preferredFlags: vk::MemoryPropertyFlags,
        info: *mut VSVulkanMemoryInfo,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> *mut VSGPUMemory,
    /// Returns the region to the pool immediately. All GPU uses must be complete.
    pub freeGPUMemory: unsafe extern "system-unwind" fn(memory: *mut VSGPUMemory),
    /// Accounts for external allocations on this same physical device. Reserve
    /// before allocating so the core can evict cached frames. Never include memory
    /// already counted by the core. Returns an owned reservation.
    pub reserveGPUMemory: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        bytes: i64,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> *mut VSGPUMemoryReservation,
    /// Replaces the reservation total, in bytes; negative means zero. Concurrent
    /// updates are allowed. Increasing the total triggers memory reclamation.
    pub updateGPUMemoryReservation:
        unsafe extern "system-unwind" fn(reservation: *mut VSGPUMemoryReservation, bytes: i64),
    /// Destroys the reservation, subtracting its total. Accepts null. Must not
    /// overlap another operation on the same reservation.
    pub releaseGPUMemoryReservation:
        unsafe extern "system-unwind" fn(reservation: *mut VSGPUMemoryReservation),
    /// Compiles compute source through glslang without using a GPU. Source is
    /// cached per core; no include handler or optimizer is provided. Returns null
    /// with a compiler log on failure. Use a `VSGPUShaderLanguage` value.
    pub compileGPUShader: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        language: c_int,
        source: *const c_char,
        errorLog: *mut c_char,
        errorLogSize: c_int,
    ) -> *mut VSGPUShader,
    /// Borrows the compiled SPIR-V words and writes their byte length.
    pub getGPUShaderCode: unsafe extern "system-unwind" fn(
        shader: *const VSGPUShader,
        sizeInBytes: *mut usize,
    ) -> *const u32,
    /// Releases compiled shader words.
    pub freeGPUShader: unsafe extern "system-unwind" fn(shader: *mut VSGPUShader),
    /// Creates a pool for one filter instance on a `VSVulkanQueueType` queue.
    /// Transfer pools may record only copies. The core sizes the recording ring
    /// and bounds in-flight memory. Free the pool in the filter free callback.
    pub createGPUExecPool: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        queue: c_int,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> *mut VSGPUExecPool,
    /// Drains submitted GPU work and runs all retention callbacks before returning.
    /// No thread may still hold a recording context.
    pub freeGPUExecPool: unsafe extern "system-unwind" fn(pool: *mut VSGPUExecPool),
    /// Drains submitted work and retention callbacks, returning `VSGPUDrainResult`.
    /// Call during setup or teardown with no concurrent pool users and no held
    /// context. Recordings that have not been submitted are not drained.
    pub gpuExecPoolWaitIdle: unsafe extern "system-unwind" fn(
        pool: *mut VSGPUExecPool,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Waits for a successfully submitted value on this pool, returning
    /// `VSGPUDrainResult`. Incomplete work retains its resources; device loss
    /// permits destruction. A value beyond all submitted work is fatal.
    pub gpuExecWaitValue: unsafe extern "system-unwind" fn(
        pool: *mut VSGPUExecPool,
        value: u64,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Borrows the pool timeline. Never signal it directly or publish a value
    /// that has not been submitted by this pool.
    pub gpuExecPoolTimeline:
        unsafe extern "system-unwind" fn(pool: *mut VSGPUExecPool) -> *mut VSGPUTimeline,
    /// Claims and begins a recording on this thread. End it exactly once with
    /// submit or abandon on the same thread. Do not retain the handle afterwards.
    /// Hold at most one recording per thread; a nested acquisition may be fatal.
    pub gpuExecAcquire: unsafe extern "system-unwind" fn(
        pool: *mut VSGPUExecPool,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> *mut VSGPUExecContext,
    /// Borrows the command buffer currently being recorded.
    pub gpuExecCommandBuffer:
        unsafe extern "system-unwind" fn(context: *mut VSGPUExecContext) -> vk::CommandBuffer,
    /// Adds device waits for every producer plane and retains its own frame
    /// reference until completion. The caller still owns its original reference.
    pub gpuExecReadsFrame:
        unsafe extern "system-unwind" fn(context: *mut VSGPUExecContext, frame: *const VSFrame),
    /// Declares an output plane and retains the frame until its producer pair
    /// is published at submission. Shared planes are fatal: allocate new planes
    /// for writes rather than copying a frame with shared GPU planes.
    pub gpuExecWritesPlane: unsafe extern "system-unwind" fn(
        context: *mut VSGPUExecContext,
        frame: *mut VSFrame,
        plane: c_int,
    ),
    /// Transfers scratch-buffer ownership to the recording for deferred destruction.
    pub gpuExecUsesBuffer:
        unsafe extern "system-unwind" fn(context: *mut VSGPUExecContext, buffer: *mut VSGPUBuffer),
    /// Transfers pooled-memory ownership for release after submission completes.
    /// The caller remains responsible for destroying resources bound to it.
    pub gpuExecUsesMemory:
        unsafe extern "system-unwind" fn(context: *mut VSGPUExecContext, memory: *mut VSGPUMemory),
    /// Registers thread-safe cleanup after completion, or immediately on abandon.
    /// `bytes` counts pinned VRAM; do not double-count typed retentions. The callback
    /// may free its objects, but must not acquire contexts, allocate GPU memory,
    /// or create, free or wait on pools. Avoid locks the callback may reenter.
    pub gpuExecRetain: unsafe extern "system-unwind" fn(
        context: *mut VSGPUExecContext,
        release: VSGPUReleaseFunc,
        object: *mut c_void,
        bytes: vk::DeviceSize,
    ),
    /// Consumes the recording on success or failure, submits its commands and
    /// publishes producer pairs. Optionally returns the signaled pool value for
    /// gpuExecWaitValue. Never reuse the context afterwards.
    pub gpuExecSubmit: unsafe extern "system-unwind" fn(
        context: *mut VSGPUExecContext,
        signaledValue: *mut u64,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Consumes a recording without submission and releases its retentions immediately.
    pub gpuExecAbandon: unsafe extern "system-unwind" fn(context: *mut VSGPUExecContext),
    /// Exports a plane allocation when `export_handle_type` is nonzero. Fails for
    /// CPU frames, absent planes or devices without export support.
    pub exportGPUPlane: unsafe extern "system-unwind" fn(
        frame: *const VSFrame,
        plane: c_int,
        out: *mut VSVulkanExportedMemory,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Exports an exportable timeline when `semaphore_export_handle_type` is
    /// nonzero. External waits carry memory visibility; if a third-party timeline
    /// cannot be exported, use waitGPUFrame instead.
    pub exportGPUSemaphore: unsafe extern "system-unwind" fn(
        core: *mut VSCore,
        semaphore: vk::Semaphore,
        out: *mut VSVulkanExportedSemaphore,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
    /// Waits for all plane producers and makes completed writes visible outside
    /// this device. Use before reading exported memory without an external
    /// semaphore wait; same-device Vulkan consumers use producer pairs instead.
    pub waitGPUFrame: unsafe extern "system-unwind" fn(
        frame: *const VSFrame,
        errorMessage: *mut c_char,
        errorMessageSize: c_int,
    ) -> c_int,
}

/// Whether a drain result permits destroying submitted resources.
#[must_use]
pub const fn vs_gpu_drain_safe_to_destroy(result: c_int) -> bool {
    result == VSGPUDrainResult::Drained as c_int || result == VSGPUDrainResult::DeviceLost as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn table_layout() {
        let pointer = size_of::<*const ()>();
        assert_eq!(size_of::<VSVULKANAPI>(), 43 * pointer);
        assert_eq!(offset_of!(VSVULKANAPI, gpuExecRetain), 37 * pointer);
        assert_eq!(offset_of!(VSVULKANAPI, waitGPUFrame), 42 * pointer);
        assert_eq!(size_of::<VSVulkanFunctions>(), 122 * pointer);
        assert_eq!(
            offset_of!(VSVulkanFunctions, vkCmdEndDebugUtilsLabelEXT),
            121 * pointer
        );
    }

    #[test]
    fn drain_results() {
        assert!(vs_gpu_drain_safe_to_destroy(
            VSGPUDrainResult::Drained as c_int
        ));
        assert!(vs_gpu_drain_safe_to_destroy(
            VSGPUDrainResult::DeviceLost as c_int
        ));
        assert!(!vs_gpu_drain_safe_to_destroy(
            VSGPUDrainResult::Incomplete as c_int
        ));
        assert!(!vs_gpu_drain_safe_to_destroy(3));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn data_layout() {
        assert_eq!(size_of::<VSVulkanCoreHandles>(), 48);
        assert_eq!(size_of::<VSVulkanPlaneInfo>(), 32);
        assert_eq!(size_of::<VSVulkanBufferInfo>(), 40);
        assert_eq!(size_of::<VSVulkanMemoryInfo>(), 40);
        assert_eq!(size_of::<VSVulkanExportedMemory>(), 56);
        assert_eq!(size_of::<VSVulkanExportedSemaphore>(), 16);
        assert_eq!(size_of::<VSVulkanCoreInfo>(), 336);
        assert_eq!(size_of::<VSVulkanDeviceListEntry>(), 568);
        assert_eq!(offset_of!(VSVulkanExportedMemory, offset), 24);
        assert_eq!(offset_of!(VSVulkanCoreInfo, device_uuid), 288);
        assert_eq!(offset_of!(VSVulkanDeviceListEntry, device_uuid), 532);
    }

    #[cfg(feature = "link-vs")]
    #[test]
    fn gpu_frame_and_submission() {
        use crate::{VAPOURSYNTH_API_VERSION, VSColorFamily, VSSampleType, getVapourSynthAPI};
        use std::{ffi::CStr, mem::MaybeUninit, ptr::null};
        unsafe {
            let api = getVapourSynthAPI(VAPOURSYNTH_API_VERSION);
            assert!(!api.is_null());
            let api = &*api;
            let vkapi = (api.getVulkanAPI)();
            assert!(!vkapi.is_null());
            let vkapi = &*vkapi;
            let core = (api.createCore)(0);
            let mut error = [0; 512];
            let mut info = MaybeUninit::uninit();
            if (vkapi.getVulkanCoreInfo)(core, info.as_mut_ptr(), error.as_mut_ptr(), 512) != 0 {
                eprintln!(
                    "GPU unavailable: {}",
                    CStr::from_ptr(error.as_ptr()).to_string_lossy()
                );
                (api.freeCore)(core);
                return;
            }
            let mut format = MaybeUninit::uninit();
            assert_ne!(
                (api.queryVideoFormat)(
                    format.as_mut_ptr(),
                    VSColorFamily::Gray,
                    VSSampleType::Integer,
                    8,
                    0,
                    0,
                    core
                ),
                0
            );
            let frame = (vkapi.newGPUVideoFrame)(format.as_ptr(), 16, 16, null(), core);
            assert!(!frame.is_null());
            assert_eq!((api.getFrameResidency)(frame), crate::VSNodeResidency::GPU);
            let mut plane = MaybeUninit::<VSVulkanPlaneInfo>::uninit();
            assert_eq!((vkapi.getGPUPlane)(frame, 0, plane.as_mut_ptr()), 0);
            assert!(plane.assume_init().buffer_size >= 256);
            let pool = (vkapi.createGPUExecPool)(
                core,
                VSVulkanQueueType::Compute as c_int,
                error.as_mut_ptr(),
                512,
            );
            assert!(!pool.is_null());
            let context = (vkapi.gpuExecAcquire)(pool, error.as_mut_ptr(), 512);
            assert!(!context.is_null());
            (vkapi.gpuExecWritesPlane)(context, frame, 0);
            let mut value = 0;
            assert_eq!(
                (vkapi.gpuExecSubmit)(context, &raw mut value, error.as_mut_ptr(), 512),
                0
            );
            assert_eq!(
                (vkapi.gpuExecWaitValue)(pool, value, error.as_mut_ptr(), 512),
                VSGPUDrainResult::Drained as c_int
            );
            let shader = (vkapi.compileGPUShader)(
                core,
                VSGPUShaderLanguage::GLSL as c_int,
                c"#version 460\nlayout(local_size_x = 1) in; void main() {}".as_ptr(),
                error.as_mut_ptr(),
                512,
            );
            assert!(
                !shader.is_null(),
                "{}",
                CStr::from_ptr(error.as_ptr()).to_string_lossy()
            );
            let mut bytes = 0;
            assert!(!(vkapi.getGPUShaderCode)(shader, &raw mut bytes).is_null());
            assert!(bytes > 0);
            (vkapi.freeGPUShader)(shader);
            (api.freeFrame)(frame);
            (vkapi.freeGPUExecPool)(pool);
            (api.freeCore)(core);
        }
    }
}
