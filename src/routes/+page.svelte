<script lang="ts">
  import VideoPlayer from "$lib/components/VideoPlayer.svelte";
  import Timeline from "$lib/components/Timeline.svelte";
  import ImportButton from "$lib/components/ImportButton.svelte";
  import RecordButton from "$lib/components/RecordButton.svelte";
  import RecordingPreview from "$lib/components/RecordingPreview.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { readFile } from "@tauri-apps/plugin-fs";

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

  // Recording state
  let isRecording = $state(false);
  let recordingStream = $state<MediaStream | null>(null);
  let recordingProcessId = $state<number | null>(null);
  let recordingTime = $state(0);

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

  function handleClipDelete(clipId: string) {
    // Remove the clip from the timeline
    const updatedClips = timelineClips.filter(clip => clip.id !== clipId);
    
    // Update clip positions to be contiguous
    let currentTime = 0;
    timelineClips = updatedClips.map(clip => {
      const duration = clip.endTime - clip.startTime;
      const result = {
        ...clip,
        startTime: currentTime,
        endTime: currentTime + duration
      };
      currentTime = result.endTime;
      return result;
    });
    
    // Clear selection if the deleted clip was selected
    if (selectedClip && selectedClip.id === clipId) {
      selectedClip = null;
    }
    
    console.log(`Deleted clip ${clipId}. Remaining clips:`, timelineClips.map(c => c.name));
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

  async function handleRecordStart() {
    try {
      // Clear selected clip so preview shows recording
      selectedClip = null;

      // Try to use getDisplayMedia for live preview if available
      let stream: MediaStream | null = null;
      let hasLivePreview = false;

      if (navigator.mediaDevices && navigator.mediaDevices.getDisplayMedia) {
        try {
          // Request screen capture for live preview
          stream = await navigator.mediaDevices.getDisplayMedia({
            video: true,
            audio: false,
          });
          recordingStream = stream;
          hasLivePreview = true;

          // Handle stream ending (e.g., user stops from browser dialog)
          stream.getVideoTracks()[0].addEventListener("ended", () => {
            if (isRecording) {
              handleRecordStop();
            }
          });
        } catch (previewError) {
          console.warn("Live preview not available, proceeding with backend recording only:", previewError);
          // Continue with backend-only recording if preview fails
          stream = null;
          hasLivePreview = false;
        }
      } else {
        console.warn("getDisplayMedia API not available, using backend-only recording");
        // Continue with backend-only recording
        stream = null;
        hasLivePreview = false;
      }

      // Start backend recording (backend will generate temp file path)
      const result = await invoke<{ process_id: number; output_path: string }>(
        "start_screen_recording",
        { outputPath: null }
      );

      recordingProcessId = result.process_id;
      isRecording = true;

      // If no live preview, show a placeholder message
      if (!hasLivePreview) {
        recordingStream = null; // Clear stream since we don't have live preview
      }
    } catch (error) {
      console.error("Error starting recording:", error);
      
      // Reset state on error
      if (recordingStream) {
        recordingStream.getTracks().forEach(track => track.stop());
        recordingStream = null;
      }
      isRecording = false;
      recordingProcessId = null;

      // Show user-friendly error message
      const errorMessage = error instanceof Error ? error.message : "Unknown error";
      if (errorMessage.includes("NotAllowedError") || errorMessage.includes("Permission")) {
        alert("Screen recording permission was denied. Please enable screen recording in System Preferences > Security & Privacy > Privacy > Screen Recording.");
      } else if (errorMessage.includes("NotFoundError") || errorMessage.includes("not found")) {
        alert("Could not access screen. Please ensure no other application is blocking screen recording.");
      } else {
        alert(`Failed to start recording: ${errorMessage}`);
      }
    }
  }

  async function handleRecordStop() {
    try {
      // Stop frontend stream
      if (recordingStream) {
        recordingStream.getTracks().forEach(track => track.stop());
        recordingStream = null;
      }

      // Stop backend recording
      if (recordingProcessId !== null) {
        let result: { success: boolean; file_path: string; message: string };
        
        try {
          result = await invoke<{ success: boolean; file_path: string; message: string }>(
            "stop_screen_recording",
            { processId: recordingProcessId }
          );
        } catch (invokeError) {
          throw new Error(
            invokeError instanceof Error 
              ? invokeError.message 
              : `Failed to stop recording: ${String(invokeError)}`
          );
        }

        if (result && result.success) {
          // Read the file and create blob URL
          const fileData = await readFile(result.file_path);
          const blob = new Blob([fileData], { type: "video/mp4" });
          const blobUrl = URL.createObjectURL(blob);

          // Extract filename
          const filename = result.file_path.split(/[/\\]/).pop() || `screen-recording-${Date.now()}.mp4`;

          // Get video duration
          const duration = await getVideoDuration(blobUrl);

          // Calculate start time
          let startTime = 0;
          if (timelineClips.length > 0) {
            const lastClip = timelineClips[timelineClips.length - 1];
            startTime = lastClip.endTime;
          }

          // Create new clip
          const newClip: TimelineClip = {
            id: `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
            videoUrl: blobUrl,
            videoPath: result.file_path,
            name: filename,
            duration: duration,
            startTime: startTime,
            endTime: startTime + duration,
          };

          // Add to timeline and select
          timelineClips = [...timelineClips, newClip];
          selectedClip = newClip;
        } else {
          throw new Error(result.message || "Failed to stop recording");
        }
      }
    } catch (error) {
      console.error("Error stopping recording:", error);
      alert(`Failed to stop recording: ${error instanceof Error ? error.message : "Unknown error"}`);
    } finally {
      // Reset recording state
      isRecording = false;
      recordingProcessId = null;
      recordingTime = 0;
    }
  }
</script>

<div class="app-container" ondrop={handleFileDrop} ondragover={handleDragOver} role="application">
  <div class="top-section">
    <div class="preview-panel">
      <div class="panel-header">
        <h2 class="panel-title">Preview</h2>
        <div class="header-actions">
          <RecordButton 
            isRecording={isRecording}
            recordingTime={recordingTime}
            onRecordStart={handleRecordStart}
            onRecordStop={handleRecordStop}
          />
          <ImportButton onImport={handleImport} />
        </div>
      </div>
      {#if isRecording}
        {#if recordingStream}
          <RecordingPreview stream={recordingStream} />
        {:else}
          <div class="recording-placeholder">
            <div class="recording-status">
              <span class="recording-indicator"></span>
              <h3>Recording Screen...</h3>
              <p>Recording duration: {Math.floor(recordingTime / 60)}:{(recordingTime % 60).toString().padStart(2, '0')}</p>
              <p class="recording-hint">Screen capture is in progress. Click "Stop Recording" when finished.</p>
            </div>
          </div>
        {/if}
      {:else if selectedClip}
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
      onClipDelete={handleClipDelete}
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

  .header-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .recording-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    background: var(--bg-tertiary, #1a1a1a);
    border-radius: 8px;
    border: 2px solid var(--border, #3a3a3a);
  }

  .recording-status {
    text-align: center;
    padding: 2rem;
  }

  .recording-status h3 {
    margin: 1rem 0 0.5rem 0;
    color: var(--text, #f6f6f6);
    font-size: 1.2rem;
  }

  .recording-status p {
    margin: 0.5rem 0;
    color: var(--text-secondary, #888);
    font-size: 0.9rem;
  }

  .recording-hint {
    font-size: 0.8rem !important;
    opacity: 0.7;
    margin-top: 1rem !important;
  }

  .recording-indicator {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #dc2626;
    display: inline-block;
    animation: blink 1s infinite;
    margin-bottom: 1rem;
  }

  @keyframes blink {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
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
