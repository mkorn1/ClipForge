<script lang="ts">
  import ImportButton from "$lib/components/ImportButton.svelte";
  import MediaLibrary from "$lib/components/MediaLibrary.svelte";
  import VideoPlayer from "$lib/components/VideoPlayer.svelte";

  interface VideoFile {
    path: string;
    name: string;
    url: string;
    size?: number;
  }

  let videos = $state<VideoFile[]>([]);
  let selectedVideo = $state<VideoFile | null>(null);

  function handleImport(importedVideos: VideoFile[]) {
    // Filter out duplicates based on path
    const existingPaths = new Set(videos.map(v => v.path));
    const newVideos = importedVideos.filter(v => !existingPaths.has(v.path));
    videos = [...videos, ...newVideos];
  }

  function handleVideoSelect(video: VideoFile) {
    selectedVideo = video;
  }

  function formatFileSize(bytes: number): string {
    if (!bytes) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + " " + sizes[i];
  }
</script>

<div class="app-container">
  <div class="top-section">
    <div class="left-panel">
      <h2 class="panel-title">Media Library</h2>
      <div class="import-container">
        <ImportButton onImport={handleImport} />
      </div>
      <MediaLibrary {videos} onSelect={handleVideoSelect} />
    </div>

    <div class="right-panel">
      <h2 class="panel-title">Preview</h2>
      {#if selectedVideo}
        <VideoPlayer 
          videoUrl={selectedVideo.url} 
          videoPath={selectedVideo.path}
          videoName={selectedVideo.name}
          metadata={null} 
        />
      {:else}
        <div class="no-preview">
          <p>Select a video to preview</p>
        </div>
      {/if}
    </div>
  </div>

  <div class="bottom-section">
    <div class="timeline-placeholder">
      <p>Timeline - Coming Soon</p>
    </div>
  </div>
</div>

<style>
  .app-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-primary, #1a1a1a);
  }

  .top-section {
    height: 50%;
    display: flex;
    gap: 0;
    overflow: hidden;
  }

  .left-panel,
  .right-panel {
    width: 50%;
    display: flex;
    flex-direction: column;
    padding: 1rem;
    overflow-y: auto;
    background: var(--bg-secondary, #2a2a2a);
  }

  .left-panel {
    border-right: 1px solid var(--border, #3a3a3a);
  }

  .panel-title {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text, #f6f6f6);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .import-container {
    margin-bottom: 1rem;
  }

  .no-preview {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    background: var(--bg-tertiary, #1a1a1a);
    border-radius: 8px;
    border: 2px dashed var(--border, #3a3a3a);
  }

  .no-preview p {
    color: var(--text-secondary, #888);
    font-size: 1rem;
  }

  .bottom-section {
    height: 50%;
    background: var(--bg-primary, #1a1a1a);
    border-top: 1px solid var(--border, #3a3a3a);
  }

  .timeline-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .timeline-placeholder p {
    color: var(--text-secondary, #888);
    font-size: 1.1rem;
  }

  /* Scrollbar styling */
  .left-panel::-webkit-scrollbar,
  .right-panel::-webkit-scrollbar {
    width: 8px;
  }

  .left-panel::-webkit-scrollbar-track,
  .right-panel::-webkit-scrollbar-track {
    background: var(--bg-primary, #1a1a1a);
  }

  .left-panel::-webkit-scrollbar-thumb,
  .right-panel::-webkit-scrollbar-thumb {
    background: var(--border, #3a3a3a);
    border-radius: 4px;
  }

  .left-panel::-webkit-scrollbar-thumb:hover,
  .right-panel::-webkit-scrollbar-thumb:hover {
    background: var(--text-secondary, #555);
  }

  @media (prefers-color-scheme: light) {
    .app-container {
      background: var(--bg-primary, #f0f0f0);
    }

    .left-panel,
    .right-panel {
      background: var(--bg-secondary, #ffffff);
      border-color: var(--border, #e0e0e0);
    }

    .panel-title {
      color: var(--text, #0f0f0f);
    }

    .no-preview {
      background: var(--bg-tertiary, #f9f9f9);
      border-color: var(--border, #e0e0e0);
    }

    .no-preview p {
      color: var(--text-secondary, #666);
    }

    .bottom-section {
      background: var(--bg-primary, #f0f0f0);
      border-top-color: var(--border, #e0e0e0);
    }

    .timeline-placeholder p {
      color: var(--text-secondary, #666);
    }

    .left-panel::-webkit-scrollbar-track,
    .right-panel::-webkit-scrollbar-track {
      background: var(--bg-primary, #f0f0f0);
    }

    .left-panel::-webkit-scrollbar-thumb,
    .right-panel::-webkit-scrollbar-thumb {
      background: var(--border, #e0e0e0);
    }

    .left-panel::-webkit-scrollbar-thumb:hover,
    .right-panel::-webkit-scrollbar-thumb:hover {
      background: var(--text-secondary, #999);
    }
  }
</style>
