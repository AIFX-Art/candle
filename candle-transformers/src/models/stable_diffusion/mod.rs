pub mod attention;
pub mod clip;
pub mod ddim;
pub mod ddpm;
pub mod embeddings;
pub mod euler_ancestral_discrete;
pub mod resnet;
pub mod schedulers;
pub mod unet_2d;
pub mod unet_2d_blocks;
pub mod utils;
pub mod vae;

use std::sync::Arc;

use candle::{DType, Device, Result};
use candle_nn as nn;

use self::schedulers::{Scheduler, SchedulerConfig};

#[derive(Clone, Debug)]
pub struct StableDiffusionConfig {
    pub width: usize,
    pub height: usize,
    pub clip: clip::Config,
    pub clip2: Option<clip::Config>,
    pub autoencoder: vae::AutoEncoderKLConfig,
    pub unet: unet_2d::UNet2DConditionModelConfig,
    scheduler: Arc<dyn SchedulerConfig>,
}

impl StableDiffusionConfig {
    pub fn v1_5(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/runwayml/stable-diffusion-v1-5/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, Some(1), 8),
                bc(640, Some(1), 8),
                bc(1280, Some(1), 8),
                bc(1280, None, 8),
            ],
            center_input_sample: false,
            cross_attention_dim: 768,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: false,
        };
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
            use_quant_conv: true,
            use_post_quant_conv: true,
        };
        let height = if let Some(height) = height {
            assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            512
        };

        let width = if let Some(width) = width {
            assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            512
        };

        let scheduler = Arc::new(ddim::DDIMSchedulerConfig {
            prediction_type: schedulers::PredictionType::Epsilon,
            ..Default::default()
        });

        StableDiffusionConfig {
            width,
            height,
            clip: clip::Config::v1_5(),
            clip2: None,
            autoencoder,
            scheduler,
            unet,
        }
    }

    fn v2_1_(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
        prediction_type: schedulers::PredictionType,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-2-1/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, Some(1), 5),
                bc(640, Some(1), 10),
                bc(1280, Some(1), 20),
                bc(1280, None, 20),
            ],
            center_input_sample: false,
            cross_attention_dim: 1024,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: true,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-2-1/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
            use_quant_conv: true,
            use_post_quant_conv: true,
        };
        let scheduler = Arc::new(ddim::DDIMSchedulerConfig {
            prediction_type,
            ..Default::default()
        });

        let height = if let Some(height) = height {
            assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            768
        };

        let width = if let Some(width) = width {
            assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            768
        };

        StableDiffusionConfig {
            width,
            height,
            clip: clip::Config::v2_1(),
            clip2: None,
            autoencoder,
            scheduler,
            unet,
        }
    }

    pub fn v2_1(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        // https://huggingface.co/stabilityai/stable-diffusion-2-1/blob/main/scheduler/scheduler_config.json
        Self::v2_1_(
            sliced_attention_size,
            height,
            width,
            schedulers::PredictionType::VPrediction,
        )
    }

    fn sdxl_(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
        prediction_type: schedulers::PredictionType,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, None, 5),
                bc(640, Some(2), 10),
                bc(1280, Some(10), 20),
            ],
            center_input_sample: false,
            cross_attention_dim: 2048,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: true,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
            use_quant_conv: true,
            use_post_quant_conv: true,
        };
        let scheduler = Arc::new(ddim::DDIMSchedulerConfig {
            prediction_type,
            ..Default::default()
        });

        let height = if let Some(height) = height {
            //   assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            1024
        };

        let width = if let Some(width) = width {
            // assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            1024
        };

        StableDiffusionConfig {
            width,
            height,
            clip: clip::Config::sdxl(),
            clip2: Some(clip::Config::sdxl2()),
            autoencoder,
            scheduler,
            unet,
        }
    }

    fn sdxl_turbo_(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
        prediction_type: schedulers::PredictionType,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/stabilityai/sdxl-turbo/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, None, 5),
                bc(640, Some(2), 10),
                bc(1280, Some(10), 20),
            ],
            center_input_sample: false,
            cross_attention_dim: 2048,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: true,
        };
        // https://huggingface.co/stabilityai/sdxl-turbo/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
            use_quant_conv: true,
            use_post_quant_conv: true,
        };
        /* let scheduler = Arc::new(
            euler_ancestral_discrete::EulerAncestralDiscreteSchedulerConfig {
                prediction_type,
                timestep_spacing: schedulers::TimestepSpacing::Trailing,
                ..Default::default()
            },
        ); */

        let scheduler = Arc::new(
            euler_ancestral_discrete::EulerAncestralDiscreteSchedulerConfig {
                prediction_type,
                timestep_spacing: schedulers::TimestepSpacing::Trailing,
                ..Default::default()
            },
        );

        let height = if let Some(height) = height {
            assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            512
        };

        let width = if let Some(width) = width {
            assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            512
        };

        Self {
            width,
            height,
            clip: clip::Config::sdxl(),
            clip2: Some(clip::Config::sdxl2()),
            autoencoder,
            scheduler,
            unet,
        }
    }

    fn sdxl_aifx_(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
        eta: Option<f64>,
        prediction_type: schedulers::PredictionType,
        sampler: Option<String>,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, None, 5),
                bc(640, Some(2), 10),
                bc(1280, Some(10), 20),
            ],
            center_input_sample: false,
            cross_attention_dim: 2048,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: true,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
            use_quant_conv: true,
            use_post_quant_conv: true,
        };
       /*  let scheduler = Arc::new(ddim::DDIMSchedulerConfig {
            prediction_type,
            eta: eta.unwrap_or(0.),
            ..Default::default()
        });
 */
        let height = if let Some(height) = height {
            // assert_eq!(height % 8, 0, "height has to be divisible by 8");
            if height % 8 != 0 {
                let t = height / 8;
                (t + 1) * 8
            } else {
                height
            }
        } else {
            1024
        };

        let width = if let Some(width) = width {
            //  assert_eq!(width % 8, 0, "width has to be divisible by 8");
            if width % 8 != 0 {
                let t = width / 8;
                (t + 1) * 8
            } else {
                width
            }
        } else {
            1024
        };

        println!("height, width: {} {}", height, width);

        match sampler {
            Some(sampler) => {
                match sampler.as_str() {
                    "DDPM" => {
                        let scheduler = Arc::new(ddpm::DDPMSchedulerConfig {
                            prediction_type,
                            eta: eta.unwrap_or(0.),
                            ..Default::default()
                        });
                        StableDiffusionConfig {
                            width,
                            height,
                            clip: clip::Config::sdxl(),
                            clip2: Some(clip::Config::sdxl2()),
                            autoencoder,
                            scheduler,
                            unet,
                        }
                    }
                    "EULER_ANCESTRAL" => {
                        let scheduler = Arc::new(
                            euler_ancestral_discrete::EulerAncestralDiscreteSchedulerConfig {
                                prediction_type,
                                eta: eta.unwrap_or(0.),
                                timestep_spacing: schedulers::TimestepSpacing::Leading,
                                ..Default::default()
                            },
                        );
                        StableDiffusionConfig {
                            width,
                            height,
                            clip: clip::Config::sdxl(),
                            clip2: Some(clip::Config::sdxl2()),
                            autoencoder,
                            scheduler,
                            unet,
                        }
                    }
                    "DDIM" | &_ => {
                        let scheduler = Arc::new(ddim::DDIMSchedulerConfig {
                            prediction_type,
                            eta: eta.unwrap_or(0.),
                            ..Default::default()
                        });
                        StableDiffusionConfig {
                            width,
                            height,
                            clip: clip::Config::sdxl(),
                            clip2: Some(clip::Config::sdxl2()),
                            autoencoder,
                            scheduler,
                            unet,
                        }
                    }
                }
            }
            None => {
                let scheduler = Arc::new(ddim::DDIMSchedulerConfig {
                    prediction_type,
                    eta: eta.unwrap_or(0.),
                    ..Default::default()
                });
                StableDiffusionConfig {
                    width,
                    height,
                    clip: clip::Config::sdxl(),
                    clip2: Some(clip::Config::sdxl2()),
                    autoencoder,
                    scheduler,
                    unet,
                }
            }
        }

        /*    let scheduler = Arc::new(ddpm::DDPMSchedulerConfig {
            prediction_type,
            //eta: eta.unwrap_or(0.),
            ..Default::default()
        }); */
    }

    pub fn sdxl(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
        eta: Option<f64>,
        sampler: Option<String>,
    ) -> Self {
        Self::sdxl_aifx_(
            sliced_attention_size,
            height,
            width,
            eta,
            // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/scheduler/scheduler_config.json
            //schedulers::PredictionType::Epsilon, //og
            schedulers::PredictionType::Epsilon,
            sampler,
        )
    }

    pub fn sdxl_turbo(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        Self::sdxl_turbo_(
            sliced_attention_size,
            height,
            width,
            // https://huggingface.co/stabilityai/sdxl-turbo/blob/main/scheduler/scheduler_config.json
            schedulers::PredictionType::Epsilon,
        )
    }

    pub fn ssd1b(
        sliced_attention_size: Option<usize>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        let bc = |out_channels, use_cross_attn, attention_head_dim| unet_2d::BlockConfig {
            out_channels,
            use_cross_attn,
            attention_head_dim,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/unet/config.json
        let unet = unet_2d::UNet2DConditionModelConfig {
            blocks: vec![
                bc(320, None, 5),
                bc(640, Some(2), 10),
                bc(1280, Some(10), 20),
            ],
            center_input_sample: false,
            cross_attention_dim: 2048,
            downsample_padding: 1,
            flip_sin_to_cos: true,
            freq_shift: 0.,
            layers_per_block: 2,
            mid_block_scale_factor: 1.,
            norm_eps: 1e-5,
            norm_num_groups: 32,
            sliced_attention_size,
            use_linear_projection: true,
        };
        // https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKLConfig {
            block_out_channels: vec![128, 256, 512, 512],
            layers_per_block: 2,
            latent_channels: 4,
            norm_num_groups: 32,
            use_quant_conv: true,
            use_post_quant_conv: true,
        };
        let scheduler = Arc::new(ddim::DDIMSchedulerConfig {
            ..Default::default()
        });

        let height = if let Some(height) = height {
            assert_eq!(height % 8, 0, "height has to be divisible by 8");
            height
        } else {
            1024
        };

        let width = if let Some(width) = width {
            assert_eq!(width % 8, 0, "width has to be divisible by 8");
            width
        } else {
            1024
        };

        Self {
            width,
            height,
            clip: clip::Config::ssd1b(),
            clip2: Some(clip::Config::ssd1b2()),
            autoencoder,
            scheduler,
            unet,
        }
    }

    pub fn build_vae<P: AsRef<std::path::Path>>(
        &self,
        vae_weights: P,
        device: &Device,
        dtype: DType,
    ) -> Result<vae::AutoEncoderKL> {
        let vs_ae =
            unsafe { nn::VarBuilder::from_mmaped_safetensors(&[vae_weights], dtype, device)? };
        // https://huggingface.co/runwayml/stable-diffusion-v1-5/blob/main/vae/config.json
        let autoencoder = vae::AutoEncoderKL::new(vs_ae, 3, 3, self.autoencoder.clone())?;
        Ok(autoencoder)
    }

    pub fn build_unet<P: AsRef<std::path::Path>>(
        &self,
        unet_weights: P,
        device: &Device,
        in_channels: usize,
        use_flash_attn: bool,
        dtype: DType,
    ) -> Result<unet_2d::UNet2DConditionModel> {
        let vs_unet =
            unsafe { nn::VarBuilder::from_mmaped_safetensors(&[unet_weights], dtype, device)? };
        let unet = unet_2d::UNet2DConditionModel::new(
            vs_unet,
            in_channels,
            4,
            use_flash_attn,
            self.unet.clone(),
        )?;
        Ok(unet)
    }

    pub fn build_scheduler(&self, n_steps: usize) -> Result<Box<dyn Scheduler>> {
        self.scheduler.build(n_steps)
    }
}

pub fn build_clip_transformer<P: AsRef<std::path::Path>>(
    clip: &clip::Config,
    clip_weights: P,
    device: &Device,
    dtype: DType,
) -> Result<clip::ClipTextTransformer> {
    let vs = unsafe { nn::VarBuilder::from_mmaped_safetensors(&[clip_weights], dtype, device)? };
    let text_model = clip::ClipTextTransformer::new(vs, clip)?;
    Ok(text_model)
}
