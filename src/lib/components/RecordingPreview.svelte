<script lang="ts">
  interface Props {
    stream: MediaStream | null;
  }

  let { stream }: Props = $props();
  let videoElement = $state<HTMLVideoElement | null>(null);

  // Set video element srcObject when stream changes
  $effect(() => {
    if (videoElement && stream) {
      videoElement.srcObject = stream;
      videoElement.play().catch(err => {
        console.error("Error playing recording preview:", err);
      });
    }
  });

  // Cleanup: stop all tracks when component unmounts or stream changes
  $effect(() => {
    return () => {
      if (stream) {
        stream.getTracks().forEach(track => {
          track.stop();
        });
      }
      if (videoElement && videoElement.srcObject) {
        videoElement.srcObject = null;
      }
    };
  });
</script>

{#if stream}
  <div class="recording-preview">
    <div class="video-container">
      <video
        bind:this={videoElement}
        autoplay
        playsinline
        muted
        class="preview-video"
        aria-label="Live recording preview"
      ></video>
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

  .preview-video {
    width: 100%;
    height: 100%;
    object-fit: contain;
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

