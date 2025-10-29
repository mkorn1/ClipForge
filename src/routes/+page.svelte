<script lang="ts">
  import VideoPlayer from "$lib/components/VideoPlayer.svelte";
  import Timeline from "$lib/components/Timeline.svelte";
  import ImportButton from "$lib/components/ImportButton.svelte";

  interface TimelineClip {
    id: string;
    videoUrl: string;
    videoPath: string;
    name: string;
    duration: number;
    startTime: number;
    endTime: number;
  }

  let timelineClips = $state<TimelineClip[]>([]);
  let selectedClip = $state<TimelineClip | null>(null);
  let currentTime = $state(0);
  let isScrubbing = $state(false);
  let wasPlayingBeforeScrub = $state(false);

  function getMimeType(filename: string): string {
    const ext = filename.split(".").pop()?.toLowerCase();
    const mimeTypes: Record<string, string> = {
      "mp4": "video/mp4",
      "mov": "video/quicktime",
      "avi": "video/x-msvideo",
      "mkv": "video/x-matroska",
      "webm": "video/webm",
    };
    return mimeTypes[ext || ""] || "video/mp4";
  }

  function getVideoDuration(videoUrl: string): Promise<number> {
    return new Promise((resolve) => {
      const video = document.createElement('video');
      video.src = videoUrl;
      video.addEventListener('loadedmetadata', () => {
        resolve(video.duration || 10);
      });
      video.addEventListener('error', () => {
        resolve(10); // Fallback to 10 seconds on error
      });
    });
  }

  interface VideoFile {
    path: string;
    name: string;
    url: string;
    size?: number;
  }

  async function handleImport(videos: VideoFile[]) {
    for (const video of videos) {
      const duration = await getVideoDuration(video.url);
      
      // Calculate the end position of the last clip
      let startTime = 0;
      if (timelineClips.length > 0) {
        const lastClip = timelineClips[timelineClips.length - 1];
        startTime = lastClip.endTime;
      }
      
      const newClip: TimelineClip = {
        id: `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        videoUrl: video.url,
        videoPath: video.path,
        name: video.name,
        duration: duration,
        startTime: startTime,
        endTime: startTime + duration
      };

      timelineClips = [...timelineClips, newClip];
    }
  }

  async function handleFileImport(files: File[]) {
    for (const file of files) {
      const videoUrl = URL.createObjectURL(file);
      
      const duration = await getVideoDuration(videoUrl);
      
      // Calculate the end position of the last clip
      let startTime = 0;
      if (timelineClips.length > 0) {
        const lastClip = timelineClips[timelineClips.length - 1];
        startTime = lastClip.endTime;
      }
      
      const newClip: TimelineClip = {
        id: `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        videoUrl: videoUrl,
        videoPath: file.name,
        name: file.name,
        duration: duration,
        startTime: startTime,
        endTime: startTime + duration
      };

      timelineClips = [...timelineClips, newClip];
    }
  }

  async function handleVideoDrop(dropData: any) {
    console.log('handleVideoDrop called with:', dropData);
    
    if (dropData.type === 'video' && dropData.video) {
      const video = dropData.video;
      console.log('Adding video to timeline:', video);
      
      // Calculate the end position of the last clip
      let startTime = 0;
      if (timelineClips.length > 0) {
        const lastClip = timelineClips[timelineClips.length - 1];
        startTime = lastClip.endTime;
      }

      const duration = await getVideoDuration(video.url);
      
      const newClip: TimelineClip = {
        id: `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        videoUrl: video.url,
        videoPath: video.path,
        name: video.name,
        duration: duration,
        startTime: startTime,
        endTime: startTime + duration
      };

      timelineClips = [...timelineClips, newClip];
    }
  }

  function handleClipSelect(clip: TimelineClip) {
    console.log('Clip selected:', clip);
    selectedClip = clip;
  }

  function handleClipUpdate(clipId: string, startTime: number, endTime: number) {
    // Update the clip in the timeline
    timelineClips = timelineClips.map(clip => {
      if (clip.id === clipId) {
        return { ...clip, startTime, endTime };
      }
      return clip;
    });
    
    console.log(`Updated clip ${clipId} to ${startTime}s - ${endTime}s`);
  }

  function handleClipsReorder(reorderedClips: TimelineClip[]) {
    // Update the timeline with reordered clips
    timelineClips = reorderedClips;
    console.log(`Reordered clips. New order:`, reorderedClips.map(c => c.name));
  }

  function handleTimeSeek(time: number) {
    console.log('Timeline seeking to time:', time);
    
    // Track that we're scrubbing
    if (!isScrubbing) {
      wasPlayingBeforeScrub = false; // TODO: track actual play state
      isScrubbing = true;
    }
    
    currentTime = time;
  }

  function handleTimeUpdate(time: number) {
    if (!isScrubbing && selectedClip) {
      // Convert video playback time to timeline absolute time
      currentTime = selectedClip.startTime + time;
    }
  }

  function handleScrubEnd() {
    isScrubbing = false;
    wasPlayingBeforeScrub = false;
  }

  function handleFileDrop(event: DragEvent) {
    event.preventDefault();
    const files = Array.from(event.dataTransfer?.files || []);
    const videoFiles = files.filter(file => file.type.startsWith('video/'));
    if (videoFiles.length > 0) {
      handleFileImport(videoFiles);
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
  }
</script>

<div class="app-container" ondrop={handleFileDrop} ondragover={handleDragOver} role="application">
  <div class="top-section">
    <div class="preview-panel">
      <div class="panel-header">
        <h2 class="panel-title">Preview</h2>
        <ImportButton onImport={handleImport} />
      </div>
      {#if selectedClip}
        <VideoPlayer 
          videoUrl={selectedClip.videoUrl} 
          videoPath={selectedClip.videoPath}
          videoName={selectedClip.name}
          metadata={null}
          currentTime={currentTime}
          onTimeUpdate={handleTimeUpdate}
        />
      {:else}
        <div class="no-preview">
          <p>Click a clip to preview</p>
        </div>
      {/if}
    </div>
  </div>

  <div class="bottom-section">
    <Timeline 
      clips={timelineClips} 
      onClipSelect={handleClipSelect}
      onDrop={handleVideoDrop}
      onClipUpdate={handleClipUpdate}
      onClipsReorder={handleClipsReorder}
      currentTime={currentTime}
      onTimeSeek={handleTimeSeek}
      onScrubEnd={handleScrubEnd}
    />
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
    overflow: hidden;
  }

  .preview-panel {
    width: 100%;
    display: flex;
    flex-direction: column;
    padding: 1rem;
    overflow-y: auto;
    background: var(--bg-secondary, #2a2a2a);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .panel-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text, #f6f6f6);
    text-transform: uppercase;
    letter-spacing: 0.05em;
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


  /* Scrollbar styling */
  .preview-panel::-webkit-scrollbar {
    width: 8px;
  }

  .preview-panel::-webkit-scrollbar-track {
    background: var(--bg-primary, #1a1a1a);
  }

  .preview-panel::-webkit-scrollbar-thumb {
    background: var(--border, #3a3a3a);
    border-radius: 4px;
  }

  .preview-panel::-webkit-scrollbar-thumb:hover {
    background: var(--text-secondary, #555);
  }

  @media (prefers-color-scheme: light) {
    .app-container {
      background: var(--bg-primary, #f0f0f0);
    }

    .preview-panel {
      background: var(--bg-secondary, #ffffff);
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

    .preview-panel::-webkit-scrollbar-track {
      background: var(--bg-primary, #f0f0f0);
    }

    .preview-panel::-webkit-scrollbar-thumb {
      background: var(--border, #e0e0e0);
    }

    .preview-panel::-webkit-scrollbar-thumb:hover {
      background: var(--text-secondary, #999);
    }
  }
</style>
