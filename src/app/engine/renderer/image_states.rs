use ash::vk;

use crate::app::engine::rendering_context::ImageLayoutState;

pub struct ImageStates {
    pub undefined_image_state: ImageLayoutState,
    pub renderable_image_state: ImageLayoutState,
    pub present_image_state: ImageLayoutState,
    pub depth_attach_state: ImageLayoutState,
}

impl Default for ImageStates {
    fn default() -> Self {
        let undefined_image_state = ImageLayoutState {
            layout: vk::ImageLayout::UNDEFINED,
            access_mask: vk::AccessFlags::empty(),
            stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        };
        let renderable_image_state = ImageLayoutState {
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        };

        let present_image_state = ImageLayoutState {
            layout: vk::ImageLayout::PRESENT_SRC_KHR,
            access_mask: vk::AccessFlags::empty(),
            stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        };
        let depth_attach_state = ImageLayoutState {
            layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        };

        ImageStates {
            undefined_image_state,
            renderable_image_state,
            present_image_state,
            depth_attach_state,
        }
    }
}
