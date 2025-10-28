<script lang="ts">
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    videoUrl?: string;
    videoPath?: string;
    videoName?: string;
    metadata?: {
      width: number;
      height: number;
      duration: number;
      fileSize: string;
    } | null;
  }

  let { videoUrl = "", videoPath, videoName = "export", metadata = null }: Props = $props();

  let videoElement = $state<HTMLVideoElement | null>(null);
  let isPlaying = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);
  let isDragging = $state(false);
  let errorMessage = $state<string | null>(null);
  let isLoading = $state(true);
  let isExporting = $state(false);
  let exportMessage = $state<string | null>(null);

  // Reset loading state when video URL changes
  $effect(() => {
    videoUrl;
    isLoading = true;
    errorMessage = null;
  });

  onMount(() => {
    if (videoElement) {
      videoElement.addEventListener("loadstart", () => {
        isLoading = true;
      });

      videoElement.addEventListener("loadedmetadata", () => {
        duration = videoElement!.duration;
        isLoading = false;
      });

      videoElement.addEventListener("timeupdate", () => {
        if (!isDragging) {
          currentTime = videoElement!.currentTime;
        }
      });

      videoElement.addEventListener("error", (e) => {
        console.error("Video error:", e);
        console.error("Video URL:", videoUrl);
        console.error("Video element error:", videoElement?.error);
        console.error("Network state:", videoElement?.networkState);
        console.error("Ready state:", videoElement?.readyState);
        
        const error = videoElement?.error;
        if (error) {
          errorMessage = `Error loading video: ${error.message || 'Unknown error'}`;
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

  async function handleExport() {
    if (!videoPath) {
      exportMessage = "Error: No video path available";
      return;
    }

    isExporting = true;
    exportMessage = null;

    try {
      // Open save dialog
      const destination = await save({
        title: "Export Video",
        defaultPath: `${videoName}_export.mp4`,
        filters: [
          {
            name: "Video",
            extensions: ["mp4", "mov", "webm"],
          },
        ],
      });

      if (!destination) {
        isExporting = false;
        return; // User cancelled
      }

      // Call the Rust backend to export the video
      const result = await invoke("export_video", {
        sourcePath: videoPath,
        destinationPath: destination,
      });

      if (result && typeof result === "object" && "success" in result) {
        const exportResult = result as { success: boolean; message: string; output_path?: string };
        if (exportResult.success) {
          exportMessage = `✓ ${exportResult.message}`;
        } else {
          exportMessage = `Error: ${exportResult.message}`;
        }
      }
    } catch (error) {
      console.error("Export error:", error);
      exportMessage = `Error: ${error instanceof Error ? error.message : "Unknown error"}`;
    } finally {
      isExporting = false;
      // Clear message after 3 seconds
      setTimeout(() => {
        exportMessage = null;
      }, 3000);
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
      {#if errorMessage}
        <div class="error-message">
          <p>{errorMessage}</p>
          <p class="error-details">Check the console for more details</p>
        </div>
      {:else if isLoading}
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

    {#if isExporting}
      <div class="export-status">
        <div class="export-loading">Exporting video...</div>
      </div>
    {:else if exportMessage}
      <div class="export-status">
        <div class="export-message">{exportMessage}</div>
      </div>
    {/if}

    {#if videoPath}
      <div class="export-section">
        <button class="export-button" onclick={handleExport} disabled={isExporting}>
          {#if isExporting}
            Exporting...
          {:else}
            💾 Export Video
          {/if}
        </button>
      </div>
    {/if}

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

  .error-message {
    position: absolute;
    color: white;
    text-align: center;
    padding: 1rem;
  }

  .error-message p {
    margin: 0.5rem 0;
    font-size: 1rem;
  }

  .error-details {
    font-size: 0.8rem !important;
    opacity: 0.8;
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

  .export-section {
    padding: 0.5rem;
  }

  .export-button {
    width: 100%;
    background: var(--accent, #396cd8);
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.75rem 1rem;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .export-button:hover:not(:disabled) {
    background: var(--accent-hover, #295bb8);
    transform: translateY(-1px);
  }

  .export-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .export-status {
    padding: 0.5rem;
  }

  .export-loading {
    text-align: center;
    color: var(--accent, #396cd8);
    font-size: 0.9rem;
  }

  .export-message {
    text-align: center;
    color: var(--text, #0f0f0f);
    font-size: 0.9rem;
    padding: 0.5rem;
    border-radius: 4px;
    background: var(--bg-tertiary, #f5f5f5);
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

    .export-message {
      color: var(--text, #f6f6f6);
      background: var(--bg-tertiary, #2f2f2f);
    }
  }
</style>
