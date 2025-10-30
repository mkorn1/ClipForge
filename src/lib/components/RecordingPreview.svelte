<script lang="ts">
  interface Props {
    stream: MediaStream | null;
    webcamStream?: MediaStream | null; // Optional webcam stream for PiP mode
    pipPosition?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';
    pipSize?: { width: number; height: number };
  }

  let { 
    stream, 
    webcamStream = null,
    pipPosition = 'bottom-right',
    pipSize = { width: 320, height: 240 }
  }: Props = $props();
  
  let videoElement = $state<HTMLVideoElement | null>(null);
  let webcamVideoElement = $state<HTMLVideoElement | null>(null);
  let canvasElement = $state<HTMLCanvasElement | null>(null);
  let animationFrameId = $state<number | null>(null);
  let isPiPMode = $derived(webcamStream !== null && stream !== null);

  // Set video element srcObject when stream changes
  $effect(() => {
    if (videoElement && stream) {
      videoElement.srcObject = stream;
      videoElement.play().catch(err => {
        console.error("Error playing recording preview:", err);
      });
    }
  });

  // Set webcam video element srcObject when webcam stream changes
  $effect(() => {
    if (webcamVideoElement && webcamStream) {
      webcamVideoElement.srcObject = webcamStream;
      webcamVideoElement.play().catch(err => {
        console.error("Error playing webcam preview:", err);
      });
    }
  });

  // Composite streams using Canvas for PiP mode
  $effect(() => {
    if (!isPiPMode || !canvasElement || !videoElement || !webcamVideoElement) {
      // Cancel any ongoing animation frame
      if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
      }
      return;
    }

    const canvas = canvasElement;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Helper function to draw rounded rectangle (with fallback for older browsers)
    const drawRoundedRect = (ctx: CanvasRenderingContext2D, x: number, y: number, width: number, height: number, radius: number) => {
      if (typeof ctx.roundRect === 'function') {
        ctx.roundRect(x, y, width, height, radius);
      } else {
        // Fallback for browsers without roundRect support
        ctx.beginPath();
        ctx.moveTo(x + radius, y);
        ctx.lineTo(x + width - radius, y);
        ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
        ctx.lineTo(x + width, y + height - radius);
        ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
        ctx.lineTo(x + radius, y + height);
        ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
        ctx.lineTo(x, y + radius);
        ctx.quadraticCurveTo(x, y, x + radius, y);
        ctx.closePath();
      }
    };

    // Set canvas size to match video
    const updateCanvasSize = () => {
      const video = videoElement;
      if (video && video.videoWidth > 0 && video.videoHeight > 0) {
        canvas.width = video.videoWidth;
        canvas.height = video.videoHeight;
      }
    };

    // Initial size update
    updateCanvasSize();

    // Watch for video dimensions to be available
    const handleLoadedMetadata = () => {
      updateCanvasSize();
    };
    videoElement.addEventListener('loadedmetadata', handleLoadedMetadata);

    // Animation loop to composite frames
    const drawFrame = () => {
      if (!videoElement || !webcamVideoElement || !canvas || !ctx) {
        animationFrameId = null;
        return;
      }

      const screenVideo = videoElement;
      const webcamVideo = webcamVideoElement;

      // Only draw if both videos are ready
      if (screenVideo.readyState >= 2 && webcamVideo.readyState >= 2) {
        // Clear canvas
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        // Draw screen video (background)
        ctx.drawImage(screenVideo, 0, 0, canvas.width, canvas.height);

        // Calculate webcam overlay position and size
        const pipWidth = Math.min(pipSize.width, canvas.width * 0.3);
        const pipHeight = Math.min(pipSize.height, canvas.height * 0.3);
        const aspectRatio = webcamVideo.videoWidth / webcamVideo.videoHeight;
        const scaledWidth = pipHeight * aspectRatio;
        const scaledHeight = pipWidth / aspectRatio;
        
        // Use the smaller dimension to maintain aspect ratio
        const finalWidth = scaledWidth <= pipWidth ? scaledWidth : pipWidth;
        const finalHeight = scaledWidth <= pipWidth ? pipHeight : scaledHeight;

        // Calculate position based on pipPosition
        let x = 0;
        let y = 0;
        const margin = 10;

        switch (pipPosition) {
          case 'bottom-right':
            x = canvas.width - finalWidth - margin;
            y = canvas.height - finalHeight - margin;
            break;
          case 'bottom-left':
            x = margin;
            y = canvas.height - finalHeight - margin;
            break;
          case 'top-right':
            x = canvas.width - finalWidth - margin;
            y = margin;
            break;
          case 'top-left':
            x = margin;
            y = margin;
            break;
        }

        // Draw rounded rectangle background for webcam
        ctx.save();
        const cornerRadius = 8;
        ctx.beginPath();
        drawRoundedRect(ctx, x - 2, y - 2, finalWidth + 4, finalHeight + 4, cornerRadius);
        ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
        ctx.fill();

        // Draw webcam video with rounded corners
        ctx.beginPath();
        drawRoundedRect(ctx, x, y, finalWidth, finalHeight, cornerRadius);
        ctx.clip();
        ctx.drawImage(webcamVideo, x, y, finalWidth, finalHeight);
        ctx.restore();
      }

      animationFrameId = requestAnimationFrame(drawFrame);
    };

    // Start the animation loop
    animationFrameId = requestAnimationFrame(drawFrame);

    // Cleanup
    return () => {
      videoElement.removeEventListener('loadedmetadata', handleLoadedMetadata);
      if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
      }
    };
  });

  // Cleanup: stop all tracks when component unmounts or stream changes
  $effect(() => {
    return () => {
      if (stream) {
        stream.getTracks().forEach(track => {
          track.stop();
        });
      }
      if (webcamStream) {
        webcamStream.getTracks().forEach(track => {
          track.stop();
        });
      }
      if (videoElement && videoElement.srcObject) {
        videoElement.srcObject = null;
      }
      if (webcamVideoElement && webcamVideoElement.srcObject) {
        webcamVideoElement.srcObject = null;
      }
      if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
      }
    };
  });
</script>

{#if stream}
  <div class="recording-preview">
    <div class="video-container">
      {#if isPiPMode}
        <!-- PiP Mode: Use Canvas for compositing -->
        <canvas
          bind:this={canvasElement}
          class="preview-canvas"
          aria-label="Live recording preview with picture-in-picture"
        ></canvas>
        <!-- Hidden video elements for Canvas -->
        <video
          bind:this={videoElement}
          autoplay
          playsinline
          muted
          class="hidden-video"
          aria-hidden="true"
        ></video>
        <video
          bind:this={webcamVideoElement}
          autoplay
          playsinline
          muted
          class="hidden-video"
          aria-hidden="true"
        ></video>
      {:else}
        <!-- Single stream mode: Use video element directly -->
        <video
          bind:this={videoElement}
          autoplay
          playsinline
          muted
          class="preview-video"
          aria-label="Live recording preview"
        ></video>
      {/if}
      <div class="recording-overlay">
        <div class="recording-badge">
          <span class="recording-dot"></span>
          <span class="recording-text">RECORDING</span>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .recording-preview {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    flex: 1;
    overflow: hidden;
  }

  .video-container {
    position: relative;
    width: 100%;
    background: black;
    border-radius: 4px;
    overflow: hidden;
    aspect-ratio: 16 / 9;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .preview-video,
  .preview-canvas {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .hidden-video {
    display: none;
    position: absolute;
    width: 0;
    height: 0;
  }

  .recording-overlay {
    position: absolute;
    top: 1rem;
    left: 1rem;
    z-index: 10;
  }

  .recording-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(220, 38, 38, 0.9);
    color: white;
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .recording-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: white;
    animation: blink 1s infinite;
  }

  @keyframes blink {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }

  .recording-text {
    text-transform: uppercase;
  }
</style>

