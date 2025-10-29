<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  interface TimelineClip {
    id: string;
    videoUrl: string;
    videoPath: string;
    name: string;
    duration: number;
    startTime: number;
    endTime: number;
  }

  interface Props {
    clips?: TimelineClip[];
    onClipSelect?: (clip: TimelineClip) => void;
    onDrop?: (clipData: any) => void;
    onClipUpdate?: (clipId: string, startTime: number, endTime: number) => void;
    currentTime?: number;
    onTimeSeek?: (time: number) => void;
    onScrubEnd?: () => void;
  }

  let { clips = [], onClipSelect, onDrop, onClipUpdate, currentTime = 0, onTimeSeek, onScrubEnd } = $props();

  const MIN_PIXELS_PER_SECOND = 10;
  const MAX_PIXELS_PER_SECOND = 500;
  const TRACK_HEIGHT = 80;
  const HEADER_HEIGHT = 30;
  const CLIP_TOP_PADDING = 8;
  const CLIP_HEIGHT = 64;
  const CLIP_COLOR = "#3a3a3a";
  const CLIP_BORDER_COLOR = "#4a4a4a";
  const CLIP_HOVER_COLOR = "#4a4a4a";
  const PLAYHEAD_COLOR = "#4a9eff";
  const PLAYHEAD_WIDTH = 2;
  const SCRUBBER_COLOR = "#4a9eff";
  const SCRUBBER_WIDTH = 2;
  const HEADER_BG = "#2a2a2a";
  const TRACK_BG = "#1a1a1a";
  const BORDER_COLOR = "#3a3a3a";
  const TEXT_COLOR = "#ffffff";

  // Trim constants
  const TRIM_HANDLE_WIDTH = 4;
  const TRIM_HANDLE_COLOR = "#4a9eff";
  const MIN_CLIP_DURATION = 0.5; // Minimum clip duration in seconds

  let canvasElement = $state<HTMLCanvasElement | null>(null);
  let canvasContext = $state<CanvasRenderingContext2D | null>(null);
  let overlayElement = $state<HTMLDivElement | null>(null);
  let containerElement = $state<HTMLDivElement | null>(null);
  let wrapperElement = $state<HTMLDivElement | null>(null);
  let isDraggingOver = $state(false);
  let hoveredClip = $state<TimelineClip | null>(null);
  let selectedClip = $state<TimelineClip | null>(null);
  
  // Trim state
  let trimHandle = $state<'left' | 'right' | null>(null);
  let isTrimming = $state(false);
  let trimmingClip = $state<TimelineClip | null>(null);
  let originalStartTime = $state(0);
  let originalEndTime = $state(0);
  let justFinishedTrimming = $state(false);
  
  // Scrubbing state
  let isDraggingScrubber = $state(false);
  let justFinishedScrubbing = $state(false);

  const totalDuration = $derived(clips.length > 0 
    ? Math.max(...clips.map(c => c.endTime))
    : 0);

  // Calculate pixels per second based on timeline wrapper visible width
  let visibleWidth = $derived(wrapperElement && wrapperElement.clientWidth > 0 
    ? wrapperElement.clientWidth 
    : 1920);
  
  // Use a fixed base zoom level - don't auto-fit to window size
  // This ensures the timeline is always scrollable
  const BASE_PIXELS_PER_SECOND = 30; // Show 30 seconds per 900px
  const PIXELS_PER_SECOND = $derived(
    totalDuration > 0 
      ? BASE_PIXELS_PER_SECOND
      : BASE_PIXELS_PER_SECOND
  );

  const timelineWidth = $derived(totalDuration * PIXELS_PER_SECOND);
  const contentWidth = $derived(Math.max(visibleWidth, timelineWidth, MIN_PIXELS_PER_SECOND * 10));

  function timeToX(time: number): number {
    return time * PIXELS_PER_SECOND;
  }

  function xToTime(x: number): number {
    return x / PIXELS_PER_SECOND;
  }

  function drawTimeMarkers(ctx: CanvasRenderingContext2D, width: number) {
    if (totalDuration === 0) return;
    
    // Determine interval based on zoom level
    const secondsPerPixel = 1 / PIXELS_PER_SECOND;
    let markerInterval: number;
    
    if (PIXELS_PER_SECOND > 100) {
      // Very zoomed in - show seconds
      markerInterval = 1;
    } else if (PIXELS_PER_SECOND > 50) {
      // Zoomed in - show every 2 seconds
      markerInterval = 2;
    } else if (PIXELS_PER_SECOND > 20) {
      // Medium zoom - show every 5 seconds
      markerInterval = 5;
    } else {
      // Zoomed out - show every 10 seconds
      markerInterval = 10;
    }
    
    ctx.strokeStyle = BORDER_COLOR;
    ctx.lineWidth = 1;
    ctx.fillStyle = TEXT_COLOR;
    ctx.font = "10px system-ui, -apple-system, sans-serif";
    ctx.textBaseline = "top";
    
    // Draw markers
    for (let time = 0; time <= totalDuration; time += markerInterval) {
      const x = timeToX(time);
      if (x > width) break;
      
      // Draw tick line
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, HEADER_HEIGHT);
      ctx.stroke();
      
      // Draw time label
      const minutes = Math.floor(time / 60);
      const seconds = Math.floor(time % 60);
      const label = `${minutes}:${seconds.toString().padStart(2, '0')}`;
      
      // Center the label
      const metrics = ctx.measureText(label);
      ctx.fillText(label, x - metrics.width / 2, 4);
    }
  }

  function drawTimeline() {
    if (!canvasContext || !canvasElement) return;

    const ctx: CanvasRenderingContext2D = canvasContext;
    const width = canvasElement.width;
    const height = canvasElement.height;

    ctx.clearRect(0, 0, width, height);

    ctx.fillStyle = HEADER_BG;
    ctx.fillRect(0, 0, width, HEADER_HEIGHT);
    
    // Draw time markers in header
    drawTimeMarkers(ctx, width);
    
    ctx.strokeStyle = BORDER_COLOR;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, HEADER_HEIGHT);
    ctx.lineTo(width, HEADER_HEIGHT);
    ctx.stroke();

    ctx.fillStyle = TRACK_BG;
    ctx.fillRect(0, HEADER_HEIGHT, width, TRACK_HEIGHT);

    clips.forEach(clip => {
      const x = timeToX(clip.startTime);
      const clipWidth = timeToX(clip.endTime - clip.startTime);
      const y = HEADER_HEIGHT + CLIP_TOP_PADDING;
      
      const isHovered = hoveredClip?.id === clip.id;
      const isSelected = selectedClip?.id === clip.id;
      
      ctx.fillStyle = isHovered || isSelected ? CLIP_HOVER_COLOR : CLIP_COLOR;
      ctx.fillRect(x, y, clipWidth, CLIP_HEIGHT);
      
      ctx.strokeStyle = CLIP_BORDER_COLOR;
      ctx.lineWidth = 1;
      ctx.strokeRect(x, y, clipWidth, CLIP_HEIGHT);

      const thumbnailWidth = 60;
      if (clipWidth > thumbnailWidth) {
        ctx.fillStyle = "#000000";
        ctx.fillRect(x, y, Math.min(thumbnailWidth, clipWidth), CLIP_HEIGHT);
      }

      if (clipWidth > thumbnailWidth + 40) {
        ctx.fillStyle = TEXT_COLOR;
        ctx.font = "12px system-ui, -apple-system, sans-serif";
        ctx.textBaseline = "middle";
        
        const textX = x + thumbnailWidth + 12;
        const textY = y + CLIP_HEIGHT / 2;
        const maxWidth = clipWidth - thumbnailWidth - 24;
        
        let text = clip.name;
        const metrics = ctx.measureText(text);
        if (metrics.width > maxWidth) {
          text = text.substring(0, Math.floor(text.length * maxWidth / metrics.width)) + "...";
        }
        
        ctx.fillText(text, textX, textY);
      }

      // Draw trim handles for selected or hovered clips
      if ((isHovered || isSelected) && clipWidth > 40) {
        ctx.fillStyle = TRIM_HANDLE_COLOR;
        
        // Left trim handle
        ctx.fillRect(x, y, TRIM_HANDLE_WIDTH, CLIP_HEIGHT);
        
        // Right trim handle
        ctx.fillRect(x + clipWidth - TRIM_HANDLE_WIDTH, y, TRIM_HANDLE_WIDTH, CLIP_HEIGHT);
      }
    });
    
    // Draw scrubber line (playhead)
    if (currentTime > 0 && currentTime <= totalDuration) {
      const scrubberX = timeToX(currentTime);
      ctx.strokeStyle = SCRUBBER_COLOR;
      ctx.lineWidth = SCRUBBER_WIDTH;
      ctx.beginPath();
      ctx.moveTo(scrubberX, 0);
      ctx.lineTo(scrubberX, HEADER_HEIGHT + TRACK_HEIGHT);
      ctx.stroke();
      
      // Draw scrubber handle (circular indicator)
      ctx.fillStyle = SCRUBBER_COLOR;
      ctx.beginPath();
      ctx.arc(scrubberX, HEADER_HEIGHT + TRACK_HEIGHT / 2, 8, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  function getClipAtPosition(x: number, y: number): TimelineClip | null {
    if (y < HEADER_HEIGHT || y > HEADER_HEIGHT + TRACK_HEIGHT) {
      return null;
    }
    
    for (const clip of clips) {
      const clipX = timeToX(clip.startTime);
      const clipWidth = timeToX(clip.endTime - clip.startTime);
      
      if (x >= clipX && x <= clipX + clipWidth) {
        return clip;
      }
    }
    
    return null;
  }

  function isClickingScrubberHandle(x: number, y: number): boolean {
    if (currentTime <= 0 || currentTime > totalDuration) return false;
    
    const scrubberX = timeToX(currentTime);
    const handleRadius = 12; // Make it easier to grab
    const handleY = HEADER_HEIGHT + TRACK_HEIGHT / 2;
    
    const dx = x - scrubberX;
    const dy = y - handleY;
    const distance = Math.sqrt(dx * dx + dy * dy);
    
    return distance <= handleRadius;
  }

  function getTrimHandleAtPosition(x: number, y: number, clip: TimelineClip): 'left' | 'right' | null {
    if (y < HEADER_HEIGHT || y > HEADER_HEIGHT + TRACK_HEIGHT) {
      return null;
    }
    
    const clipX = timeToX(clip.startTime);
    const clipWidth = timeToX(clip.endTime - clip.startTime);
    
    // Check left trim handle
    if (x >= clipX && x <= clipX + TRIM_HANDLE_WIDTH) {
      return 'left';
    }
    
    // Check right trim handle
    if (x >= clipX + clipWidth - TRIM_HANDLE_WIDTH && x <= clipX + clipWidth) {
      return 'right';
    }
    
    return null;
  }

  function handleCanvasMouseMove(e: MouseEvent) {
    if (!overlayElement || !wrapperElement) return;
    
    const wrapperRect = wrapperElement.getBoundingClientRect();
    const scrollLeft = wrapperElement.scrollLeft;
    const x = e.clientX - wrapperRect.left + scrollLeft;
    const y = e.clientY - wrapperRect.top;
    
    const clip = getClipAtPosition(x, y);
    hoveredClip = clip;
    
    // Check if hovering over scrubber handle first
    if (isClickingScrubberHandle(x, y)) {
      trimHandle = null;
      overlayElement.style.cursor = "grab";
      drawTimeline();
      return;
    }
    
    // Check if hovering over trim handle
    let handle = null;
    if (clip) {
      handle = getTrimHandleAtPosition(x, y, clip);
    }
    
    if (handle) {
      trimHandle = handle;
      overlayElement.style.cursor = "ew-resize";
    } else if (clip) {
      trimHandle = null;
      overlayElement.style.cursor = "pointer";
    } else if (y >= HEADER_HEIGHT && y < HEADER_HEIGHT + TRACK_HEIGHT) {
      trimHandle = null;
      overlayElement.style.cursor = "pointer";
    } else {
      trimHandle = null;
      overlayElement.style.cursor = "default";
    }
    
    drawTimeline();
  }

  function handleCanvasMouseDown(e: MouseEvent) {
    if (!overlayElement || !wrapperElement) return;
    
    const wrapperRect = wrapperElement.getBoundingClientRect();
    const scrollLeft = wrapperElement.scrollLeft;
    const x = e.clientX - wrapperRect.left + scrollLeft;
    const y = e.clientY - wrapperRect.top;
    
    console.log(`Timeline mousedown: raw mouse x=${e.clientX - wrapperRect.left}px, scrollLeft=${scrollLeft}px, adjusted x=${x.toFixed(2)}px`);
    
    // Check if clicking on scrubber handle
    if (isClickingScrubberHandle(x, y)) {
      isDraggingScrubber = true;
      overlayElement.style.cursor = "grabbing";
      e.preventDefault();
      e.stopPropagation();
      drawTimeline();
      return;
    }
    
    const clip = getClipAtPosition(x, y);
    
    if (clip) {
      // Check if clicking on trim handle
      const handle = getTrimHandleAtPosition(x, y, clip);
      if (handle) {
        // Start trimming
        isTrimming = true;
        trimHandle = handle;
        trimmingClip = clip;
        selectedClip = clip;
        
        // Save original values
        originalStartTime = clip.startTime;
        originalEndTime = clip.endTime;
        
        if (onClipSelect) {
          onClipSelect(clip);
        }
        
        console.log(`Started trimming ${handle} edge of clip "${clip.name}"`);
        drawTimeline();
        return; // Don't continue with click handler
      }
    }
    // If not trimming, let the click handler handle selection
  }

  function handleCanvasClick(e: MouseEvent) {
    // Skip if we just finished trimming
    if (justFinishedTrimming) {
      justFinishedTrimming = false;
      return;
    }
    
    // Skip if we just finished scrubbing
    if (justFinishedScrubbing) {
      justFinishedScrubbing = false;
      return;
    }
    
    if (!overlayElement || !wrapperElement) return;
    
    const wrapperRect = wrapperElement.getBoundingClientRect();
    const scrollLeft = wrapperElement.scrollLeft;
    const x = e.clientX - wrapperRect.left + scrollLeft;
    const y = e.clientY - wrapperRect.top;
    
    console.log(`Timeline click identifier: raw mouse x=${e.clientX - wrapperRect.left}px, scrollLeft=${scrollLeft}px, adjusted x=${x.toFixed(2)}px`);
    console.log(`Pixels per second: ${PIXELS_PER_SECOND.toFixed(2)}`);
    
    // Check if clicking on timeline header (seek to time)
    if (y >= 0 && y < HEADER_HEIGHT) {
      if (onTimeSeek) {
        const seekTime = xToTime(x);
        const constrainedTime = Math.max(0, Math.min(totalDuration, seekTime));
        console.log(`Seeking to time (header): ${constrainedTime.toFixed(2)}s`);
        onTimeSeek(constrainedTime);
      }
      drawTimeline();
      return;
    }
    
    // Check if clicking on timeline track
    if (y >= HEADER_HEIGHT && y < HEADER_HEIGHT + TRACK_HEIGHT) {
      const clip = getClipAtPosition(x, y);
      
      if (clip) {
        // Clicked on a clip - select it and seek to clicked position
        if (onClipSelect) {
          onClipSelect(clip);
          selectedClip = clip;
        }
        
        if (onTimeSeek) {
          // Seek to the position within the clip that was clicked
          const relativeX = x - timeToX(clip.startTime);
          const clickTime = clip.startTime + xToTime(relativeX);
          const constrainedTime = Math.max(clip.startTime, Math.min(clip.endTime, clickTime));
          console.log(`Seeking to time (clip): ${constrainedTime.toFixed(2)}s`);
          onTimeSeek(constrainedTime);
        }
        
        console.log(`Clicked on clip "${clip.name}"`);
      } else if (onTimeSeek) {
        // Clicked on empty timeline space - seek to that time
        const seekTime = xToTime(x);
        const constrainedTime = Math.max(0, Math.min(totalDuration, seekTime));
        console.log(`Seeking to time: ${constrainedTime.toFixed(2)}s`);
        onTimeSeek(constrainedTime);
      }
    }
    
    drawTimeline();
  }

  function handleCanvasKeyPress(e: KeyboardEvent) {
    // Handle keyboard interaction for accessibility
    if (e.key === 'Enter' || e.key === ' ') {
      handleCanvasClick(e as any);
    }
  }

  function handleWheel(e: WheelEvent) {
    // Allow scrolling to pass through to the wrapper
    if (!wrapperElement) return;
    
    // Check if we should handle horizontal scrolling
    if (e.deltaX !== 0 || e.shiftKey) {
      e.preventDefault();
      wrapperElement.scrollLeft += e.deltaX;
    }
    
    // Allow vertical scrolling if it's vertical wheel
    if (e.deltaY !== 0 && !e.shiftKey) {
      wrapperElement.scrollTop += e.deltaY;
    }
  }

  function handleCanvasMouseLeave() {
    hoveredClip = null;
    drawTimeline();
  }

  function handleCanvasMouseUp(e: MouseEvent) {
    console.log(`MouseUp: isDraggingScrubber=${isDraggingScrubber}, isTrimming=${isTrimming}, trimmingClip=${trimmingClip?.name}, onClipUpdate=${!!onClipUpdate}`);
    
    // Handle scrubber drag end
    if (isDraggingScrubber) {
      isDraggingScrubber = false;
      justFinishedScrubbing = true;
      if (overlayElement) {
        overlayElement.style.cursor = "default";
      }
      if (onScrubEnd) {
        onScrubEnd();
      }
      drawTimeline();
      return;
    }
    
    if (isTrimming && trimmingClip) {
      // Apply final constraints on release
      if (trimHandle === 'left') {
        const minStartTime = 0;
        const maxStartTime = trimmingClip.endTime - MIN_CLIP_DURATION;
        trimmingClip.startTime = Math.max(minStartTime, Math.min(maxStartTime, trimmingClip.startTime));
      } else if (trimHandle === 'right') {
        const minEndTime = trimmingClip.startTime + MIN_CLIP_DURATION;
        const maxEndTime = totalDuration;
        trimmingClip.endTime = Math.max(minEndTime, Math.min(maxEndTime, trimmingClip.endTime));
      }
      
      if (onClipUpdate) {
        // Finish trimming - update the clip
        console.log(`Committing trim: ${trimHandle} edge of clip "${trimmingClip.name}"`);
        console.log(`  Final values: startTime=${trimmingClip.startTime.toFixed(2)}, endTime=${trimmingClip.endTime.toFixed(2)}`);
        onClipUpdate(trimmingClip.id, trimmingClip.startTime, trimmingClip.endTime);
      } else {
        // No update handler, restore original values
        console.log(`No update handler, restoring original values`);
        trimmingClip.startTime = originalStartTime;
        trimmingClip.endTime = originalEndTime;
      }
    }
    
    const wasTrimming = isTrimming;
    isTrimming = false;
    trimmingClip = null;
    trimHandle = null;
    justFinishedTrimming = wasTrimming; // Set flag if we just finished trimming
    
    // Redraw to ensure clean state
    drawTimeline();
  }

  function handleCanvasGlobalMouseMove(e: MouseEvent) {
    if (!wrapperElement) return;
    
    // Handle scrubber dragging
    if (isDraggingScrubber) {
      const wrapperRect = wrapperElement.getBoundingClientRect();
      const scrollLeft = wrapperElement.scrollLeft;
      const x = e.clientX - wrapperRect.left + scrollLeft;
      const newTime = xToTime(x);
      const constrainedTime = Math.max(0, Math.min(totalDuration, newTime));
      
      if (onTimeSeek) {
        onTimeSeek(constrainedTime);
      }
      
      drawTimeline();
      return;
    }
    
    // Handle trimming
    if (isTrimming && trimmingClip && trimHandle) {
      console.log(`handleCanvasGlobalMouseMove: Trimming in progress`);
      // Get coordinates relative to the wrapper
      const wrapperRect = wrapperElement.getBoundingClientRect();
      const scrollLeft = wrapperElement.scrollLeft;
      const x = e.clientX - wrapperRect.left + scrollLeft;
      const newTime = xToTime(x);
      
      console.log(`Trimming: handle=${trimHandle}, newTime=${newTime.toFixed(2)}`);
      
      if (trimHandle === 'left') {
        // Allow free bidirectional movement - only constrain to absolute timeline boundaries
        trimmingClip.startTime = Math.max(0, Math.min(trimmingClip.endTime - 0.1, newTime));
        console.log(`Left trim: startTime=${trimmingClip.startTime.toFixed(2)}, endTime=${trimmingClip.endTime.toFixed(2)}`);
      } else if (trimHandle === 'right') {
        // Allow free bidirectional movement - only constrain to absolute timeline boundaries
        trimmingClip.endTime = Math.min(totalDuration, Math.max(trimmingClip.startTime + 0.1, newTime));
        console.log(`Right trim: startTime=${trimmingClip.startTime.toFixed(2)}, endTime=${trimmingClip.endTime.toFixed(2)}`);
      }
      
      drawTimeline();
      return;
    }
    
    // Call the regular mouse move handler only if overlayElement exists
    if (overlayElement) {
      handleCanvasMouseMove(e);
    }
  }

  function handleDragOver(e: DragEvent) {
    if (e.type === 'dragover' || e.type === 'dragenter') {
      e.preventDefault();
      isDraggingOver = true;
    }
  }

  function handleDragLeave(e: DragEvent) {
    isDraggingOver = false;
  }
    
  function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDraggingOver = false;

    console.log('Timeline handleDrop called');
    
    if (onDrop && e.dataTransfer) {
      try {
        const textData = e.dataTransfer.getData('text/plain');
        console.log('Raw text data from dataTransfer:', textData);
        const data = JSON.parse(textData);
        console.log('Parsed drop data:', data);
        onDrop(data);
      } catch (err) {
        console.error('Failed to parse drop data:', err);
      }
    } else {
      console.log('No onDrop handler or dataTransfer available');
    }
  }

  function setupCanvas() {
    if (!canvasElement || !overlayElement) return;
    
    canvasContext = canvasElement.getContext('2d', { alpha: false });
    
    // Set canvas dimensions (internal resolution)
    canvasElement.width = contentWidth;
    canvasElement.height = HEADER_HEIGHT + TRACK_HEIGHT;
    
    // Set canvas CSS dimensions (display size)
    canvasElement.style.width = `${contentWidth}px`;
    canvasElement.style.height = `${HEADER_HEIGHT + TRACK_HEIGHT}px`;
    
    // Set overlay width to match canvas width
    overlayElement.style.width = `${contentWidth}px`;
    overlayElement.style.height = `${HEADER_HEIGHT + TRACK_HEIGHT}px`;
    
    drawTimeline();
  }

  $effect(() => {
    clips;
    drawTimeline();
  });

  $effect(() => {
    // Update canvas and overlay when contentWidth or visibleWidth changes
    if (visibleWidth && contentWidth) {
      setupCanvas();
    }
  });

  // Use requestAnimationFrame to throttle scrubber redraws
  let animationFrameId = 0;
  
  $effect(() => {
    if (currentTime !== undefined) {
      // Cancel any pending animation frame
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }
      
      // Schedule redraw for next animation frame
      animationFrameId = requestAnimationFrame(() => {
        drawTimeline();
        animationFrameId = 0;
      });
    }
    
    // Cleanup on unmount
    return () => {
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = 0;
      }
    };
  });

  onMount(() => {
    setupCanvas();
    
    function handleResize() {
      setupCanvas();
      drawTimeline();
    }
    
    // Add window-level listeners for trimming
    function handleWindowMouseMove(e: MouseEvent) {
      if (isTrimming) {
        console.log(`Window mousemove: isTrimming=true, calling handleCanvasGlobalMouseMove`);
        handleCanvasGlobalMouseMove(e);
      }
    }
    
    function handleWindowMouseUp(e: MouseEvent) {
      console.log(`Window mouseup: isTrimming=${isTrimming}`);
      if (isTrimming) {
        handleCanvasMouseUp(e);
      }
    }
    
    window.addEventListener('resize', handleResize);
    window.addEventListener('mousemove', handleWindowMouseMove);
    window.addEventListener('mouseup', handleWindowMouseUp);
    
    return () => {
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleWindowMouseMove);
      window.removeEventListener('mouseup', handleWindowMouseUp);
    };
  });

  onDestroy(() => {
    // Cleanup handled in onMount return
  });
</script>

<div class="timeline-container">
  <div 
    bind:this={wrapperElement}
    class="timeline-wrapper"
    class:drag-over={isDraggingOver}
    role="application"
    aria-label="Video timeline drop zone"
    ondragover={handleDragOver}
    ondragenter={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
  >
    {#if isDraggingOver}
      <div class="drop-overlay"></div>
    {/if}
    
    <canvas
      bind:this={canvasElement}
      class="timeline-canvas"
      role="button"
      tabindex="0"
    ></canvas>
    
    <!-- Invisible overlay for mouse events on the canvas -->
    <div 
      bind:this={overlayElement}
      class="canvas-overlay"
      role="button"
      tabindex="0"
      onmousemove={handleCanvasGlobalMouseMove}
      onmouseleave={handleCanvasMouseLeave}
      onmousedown={handleCanvasMouseDown}
      onclick={handleCanvasClick}
      onmouseup={handleCanvasMouseUp}
      onkeypress={handleCanvasKeyPress}
    ></div>
  </div>
</div>

<style>
  .timeline-container {
    height: 100%;
    min-height: 110px;
    display: flex;
    flex-direction: column;
    background: #1a1a1a;
  }

  .timeline-wrapper {
    flex: 1;
    overflow: auto;
    position: relative;
    border-top: 1px solid #3a3a3a;
  }

  .timeline-wrapper.drag-over {
    background: rgba(74, 222, 128, 0.1);
  }

  .drop-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(74, 222, 128, 0.2);
    pointer-events: none;
    z-index: 20;
    border: 2px dashed rgba(74, 222, 128, 0.5);
  }

  .timeline-canvas {
    display: block;
    cursor: default;
    pointer-events: none;
  }

  .canvas-overlay {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    pointer-events: auto;
    cursor: default;
    /* Width will be set by setupCanvas() */
  }

  .timeline-wrapper::-webkit-scrollbar {
    height: 8px;
  }

  .timeline-wrapper::-webkit-scrollbar-track {
    background: #1a1a1a;
  }

  .timeline-wrapper::-webkit-scrollbar-thumb {
    background: #3a3a3a;
    border-radius: 4px;
  }

  .timeline-wrapper::-webkit-scrollbar-thumb:hover {
    background: #4a4a4a;
  }
</style>
