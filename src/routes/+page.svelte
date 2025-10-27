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

<div class="container">
  <header class="header">
    <h1>ClipForge</h1>
    <p class="subtitle">Fast Video Editor</p>
  </header>

  <main class="main-content">
    <div class="import-section">
      <ImportButton onImport={handleImport} />
    </div>

    <div class="workspace">
      <div class="library-section">
        <MediaLibrary {videos} onSelect={handleVideoSelect} />
      </div>

      <div class="preview-section">
        <h2 class="preview-title">Preview</h2>
        {#if selectedVideo}
          <VideoPlayer videoUrl={selectedVideo.url} metadata={null} />
        {:else}
          <div class="no-preview">
            <p>Select a video to preview</p>
          </div>
        {/if}
      </div>
    </div>
  </main>
</div>

<style>
  .container {
    min-height: 100vh;
    background: var(--bg-primary, #f6f6f6);
    padding: 1rem;
  }

  .header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .header h1 {
    margin: 0;
    font-size: 2.5rem;
    font-weight: 600;
    color: var(--text, #0f0f0f);
  }

  .subtitle {
    margin: 0.5rem 0 0 0;
    color: var(--text-secondary, #666);
    font-size: 1rem;
  }

  .main-content {
    max-width: 1400px;
    margin: 0 auto;
  }

  .import-section {
    margin-bottom: 2rem;
  }

  .workspace {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .library-section,
  .preview-section {
    min-height: 400px;
  }

  .preview-title {
    margin: 0 0 1rem 0;
    font-size: 1.2rem;
    font-weight: 600;
    color: var(--text, #0f0f0f);
  }

  .no-preview {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    background: var(--bg-secondary, #ffffff);
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .no-preview p {
    color: var(--text-secondary, #666);
    font-size: 1.1rem;
  }

  @media (max-width: 1024px) {
    .workspace {
      grid-template-columns: 1fr;
    }
  }

  @media (prefers-color-scheme: dark) {
    .container {
      background: var(--bg-primary, #1e1e1e);
    }

    .header h1 {
      color: var(--text, #f6f6f6);
    }

    .subtitle {
      color: var(--text-secondary, #ccc);
    }

    .preview-title {
      color: var(--text, #f6f6f6);
    }

    .no-preview {
      background: var(--bg-secondary, #1f1f1f);
    }

    .no-preview p {
      color: var(--text-secondary, #ccc);
    }
  }
</style>
