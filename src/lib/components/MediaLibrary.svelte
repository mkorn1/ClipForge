<script lang="ts">
  interface Video {
    path: string;
    name: string;
    url: string;
    size?: number;
  }

  interface Props {
    videos: Video[];
    onSelect?: (video: Video) => void;
  }

  let { videos, onSelect }: Props = $props();

  let draggedVideo = $state<Video | null>(null);

  function handleClick(video: Video) {
    onSelect?.(video);
  }

  function handleDragStart(video: Video, e: DragEvent) {
    if (e.dataTransfer) {
      draggedVideo = video;
      const dragData = {
        type: 'video',
        video: video
      };
      console.log('Drag start - setting data:', dragData);
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData('text/plain', JSON.stringify(dragData));
    }
  }

  function handleDragEnd() {
    draggedVideo = null;
  }

  function formatFileSize(bytes: number): string {
    if (!bytes) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + " " + sizes[i];
  }
</script>

<div class="media-library">
  <h2 class="library-title">Media Library</h2>
  
  {#if videos.length === 0}
    <div class="empty-state">
      <p>No videos imported yet</p>
      <p class="empty-hint">Click "Import" to add video files</p>
    </div>
  {:else}
    <div class="video-grid">
      {#each videos as video (video.path)}
        <button 
          type="button" 
          class="video-card" 
          class:dragging={draggedVideo?.path === video.path}
          onclick={() => handleClick(video)}
          draggable="true"
          ondragstart={(e) => handleDragStart(video, e)}
          ondragend={handleDragEnd}
        >
          <div class="video-thumbnail">
            <video src={video.url} preload="metadata" muted></video>
            <div class="video-overlay">
              <span class="play-icon">▶</span>
            </div>
          </div>
          <div class="video-info">
            <h3 class="video-name" title={video.name}>{video.name}</h3>
            <p class="video-size">{formatFileSize(video.size || 0)}</p>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .media-library {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .library-title {
    display: none;
  }

  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    color: var(--text-secondary, #666);
  }

  .empty-hint {
    font-size: 0.9rem;
    margin-top: 0.5rem;
  }

  .video-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
    overflow-y: auto;
    flex: 1;
  }

  .video-card {
    cursor: pointer;
    background: var(--bg-card, #f9f9f9);
    border-radius: 8px;
    overflow: hidden;
    transition: transform 0.2s, box-shadow 0.2s;
    border: none;
    padding: 0;
    text-align: left;
  }

  .video-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .video-card.dragging {
    opacity: 0.5;
    cursor: grabbing;
  }

  .video-thumbnail {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 9;
    background: black;
    overflow: hidden;
  }

  .video-thumbnail video {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .video-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .video-card:hover .video-overlay {
    opacity: 1;
  }

  .play-icon {
    color: white;
    font-size: 2rem;
  }

  .video-info {
    padding: 0.75rem;
  }

  .video-name {
    margin: 0 0 0.25rem 0;
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text, #0f0f0f);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .video-size {
    margin: 0;
    font-size: 0.8rem;
    color: var(--text-secondary, #666);
  }

  @media (prefers-color-scheme: dark) {
    .media-library {
      background: var(--bg-secondary, #1f1f1f);
    }

    .library-title {
      color: var(--text, #f6f6f6);
    }

    .video-card {
      background: var(--bg-card, #2f2f2f);
    }

    .video-name {
      color: var(--text, #f6f6f6);
    }

    .video-size {
      color: var(--text-secondary, #ccc);
    }
  }
</style>
