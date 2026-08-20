import os
import sys
import argparse
import time
import io
import base64
import torch
import numpy as np
from PIL import Image
from diffusers import StableDiffusionXLPipeline, DPMSolverMultistepScheduler

# Fast SDXL Latent to RGB conversion matrix (approximate linear projection in 0.2ms)
# Allows visualizing the latent denoising state in real-time with zero VAE decoding overhead
SDXL_LATENT_RGB_FACTORS = torch.tensor([
    [ 0.298,  0.207, -0.064],
    [-0.142,  0.209,  0.219],
    [ 0.177,  0.137,  0.071],
    [-0.231, -0.198, -0.222]
], dtype=torch.float16)

def latent_to_b64(latents, factors):
    with torch.no_grad():
        # latents: [1, 4, H/8, W/8]
        lat = latents[0].permute(1, 2, 0) # [H/8, W/8, 4]
        rgb = torch.matmul(lat, factors) # [H/8, W/8, 3]
        rgb = (rgb + 0.5).clamp(0, 1)
        rgb_np = (rgb.cpu().numpy() * 255.0).astype(np.uint8)
        img = Image.fromarray(rgb_np).resize((512, 512), resample=Image.NEAREST)
        buf = io.BytesIO()
        img.save(buf, format='JPEG', quality=75)
        return base64.b64encode(buf.getvalue()).decode('utf-8')

def main():
    parser = argparse.ArgumentParser(description='SDXL Real Inference Runner with Live Latent Stream')
    parser.add_argument('--ckpt', type=str, required=True, help='Path to .safetensors checkpoint')
    parser.add_argument('--prompt', type=str, required=True, help='Positive prompt')
    parser.add_argument('--neg_prompt', type=str, default='ugly, deformed, disfigured, poor details, bad anatomy, bad eyes, blurry, watermark, low quality, cartoon, 3d render, extra limbs', help='Negative prompt')
    parser.add_argument('--steps', type=int, default=25, help='Number of inference steps')
    parser.add_argument('--cfg', type=float, default=6.0, help='Guidance scale')
    parser.add_argument('--width', type=int, default=1024, help='Image width')
    parser.add_argument('--height', type=int, default=1024, help='Image height')
    parser.add_argument('--seed', type=int, default=-1, help='Seed (-1 for random)')
    parser.add_argument('--output', type=str, required=True, help='Output image file path')
    args = parser.parse_args()

    print(f'[SDXL Engine] Loading checkpoint {args.ckpt} on CUDA (fp16)...', flush=True)
    start_load = time.time()
    
    pipe = StableDiffusionXLPipeline.from_single_file(
        args.ckpt,
        torch_dtype=torch.float16,
        use_safetensors=True
    )
    pipe.scheduler = DPMSolverMultistepScheduler.from_config(pipe.scheduler.config, use_karras_sigmas=True)
    pipe.to('cuda')
    pipe.enable_attention_slicing()
    print(f'[SDXL Engine] Checkpoint loaded in {time.time() - start_load:.2f}s', flush=True)

    generator = None
    if args.seed >= 0:
        generator = torch.Generator('cuda').manual_seed(args.seed)

    factors = SDXL_LATENT_RGB_FACTORS.to('cuda')

    def step_callback(pipe, step_index, timestep, callback_kwargs):
        latents = callback_kwargs.get('latents')
        if latents is not None:
            try:
                b64 = latent_to_b64(latents, factors)
                # Output machine-readable live latent stream tag
                print(f'__LATENT_PREVIEW__:{step_index + 1}/{args.steps}:{b64}', flush=True)
            except Exception as e:
                pass
        return callback_kwargs

    print(f'[SDXL Engine] Denoising ({args.steps} steps, CFG {args.cfg}, {args.width}x{args.height})...', flush=True)
    start_gen = time.time()

    image = pipe(
        prompt=args.prompt,
        negative_prompt=args.neg_prompt,
        num_inference_steps=args.steps,
        guidance_scale=args.cfg,
        width=args.width,
        height=args.height,
        generator=generator,
        callback_on_step_end=step_callback,
        callback_on_step_end_tensor_inputs=['latents']
    ).images[0]

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    image.save(args.output)
    print(f'[SDXL Engine] ✓ Image saved to {args.output} in {time.time() - start_gen:.2f}s', flush=True)

if __name__ == '__main__':
    main()
