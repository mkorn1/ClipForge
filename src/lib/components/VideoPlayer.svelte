<script lang="ts">
  import { onMount } from "svelte";

  interface Props {
    videoUrl?: string;
    metadata?: {
      width: number;
      height: number;
      duration: number;
      fileSize: string;
    } | null;
  }

  let { videoUrl = "", metadata = null }: Props = $props();

  let videoElement = $state<HTMLVideoElement | null>(null);
  let isPlaying = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);
  let isDragging = $state(false);

  onMount(() => {
    if (videoElement) {
      videoElement.addEventListener("loadedmetadata", () => {
        duration = videoElement!.duration;
      });

      videoElement.addEventListener("timeupdate", () => {
        if (!isDragging) {
          currentTime = videoElement!.currentTime;
        }
      });
    }
  });

  function togglePlay() {
    if (videoElement) {
      if (isPlaying) {
        videoElement.pause();
      } else {
        videoElement.play();
      }
      isPlaying = !isPlaying;
    }
  }

  function formatTime(seconds: number): string {
    if (!seconds || isNaN(seconds)) return "0:00";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  function handleSeek(e: MouseEvent) {
    if (videoElement && !isDragging && e.currentTarget) {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const percent = (e.clientX - rect.left) / rect.width;
      const newTime = percent * duration;
      videoElement.currentTime = newTime;
      currentTime = newTime;
    }
  }

  function handleMouseDown() {
    isDragging = true;
  }

  function handleMouseUp(e: MouseEvent) {
    isDragging = false;
    handleSeek(e);
  }

  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === " " || e.key === "Spacebar") {
      e.preventDefault();
      togglePlay();
    }
  }
</script>

<svelte:window onkeypress={handleKeyPress} />

{#if videoUrl}
  <div class="video-player">
    <div class="video-container">
      <video
        bind:this={videoElement}
        src={videoUrl}
        class="video"
        onclick={togglePlay}
        aria-label="Video player"
      ></video>
      {#if !metadata}
        <div class="loading">Loading video...</div>
      {/if}
    </div>

    <div class="controls">
      <button class="play-button" onclick={togglePlay}>
        {#if isPlaying} ⏸ {:else} ▶ {/if}
      </button>

      <div class="seek-container">
        <button type="button" class="seek-bar-button" aria-label="Seek video" onmousedown={handleMouseDown} onmouseup={handleMouseUp} onclick={handleSeek}>
          <div class="seek-bar">
            <div class="seek-progress" style="width: {duration ? (currentTime / duration) * 100 : 0}%"></div>
          </div>
        </button>
      </div>

      <div class="time-display">
        <span>{formatTime(currentTime)}</span>
        <span>/</span>
        <span>{formatTime(duration)}</span>
      </div>
    </div>

    {#if metadata}
      <div class="metadata">
        <div class="metadata-item">
          <span class="label">Resolution:</span>
          <span class="value">{metadata.width}x{metadata.height}</span>
        </div>
        <div class="metadata-item">
          <span class="label">Duration:</span>
          <span class="value">{formatTime(metadata.duration)}</span>
        </div>
        <div class="metadata-item">
          <span class="label">File Size:</span>
          <span class="value">{metadata.fileSize}</span>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .video-player {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
    background: var(--bg-secondary, #ffffff);
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
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

  .video {
    width: 100%;
    height: 100%;
    object-fit: contain;
    cursor: pointer;
  }

  .loading {
    position: absolute;
    color: white;
    font-size: 1.2rem;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .play-button {
    background: var(--accent, #396cd8);
    color: white;
    border: none;
    border-radius: 50%;
    width: 40px;
    height: 40px;
    font-size: 1rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s;
  }

  .play-button:hover {
    transform: scale(1.1);
  }

  .seek-container {
    flex: 1;
  }

  .seek-bar-button {
    border: none;
    background: none;
    padding: 0;
    width: 100%;
    cursor: pointer;
  }

  .seek-bar {
    position: relative;
    width: 100%;
    height: 8px;
    background: var(--border, #e0e0e0);
    border-radius: 4px;
  }

  .seek-progress {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: var(--accent, #396cd8);
    border-radius: 4px;
    transition: width 0.1s linear;
  }

  .time-display {
    display: flex;
    gap: 0.25rem;
    font-size: 0.9rem;
    color: var(--text-secondary, #666);
  }

  .metadata {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
    background: var(--bg-tertiary, #f5f5f5);
    border-radius: 4px;
  }

  .metadata-item {
    display: flex;
    gap: 0.5rem;
  }

  .label {
    font-weight: 600;
    color: var(--text-secondary, #666);
  }

  .value {
    color: var(--text, #0f0f0f);
  }

  @media (prefers-color-scheme: dark) {
    .video-player {
      background: var(--bg-secondary, #1f1f1f);
    }

    .metadata {
      background: var(--bg-tertiary, #2f2f2f);
    }

    .label, .value {
      color: var(--text, #f6f6f6);
    }

    .time-display {
      color: var(--text-secondary, #ccc);
    }
  }
</style>
