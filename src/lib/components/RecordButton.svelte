<script lang="ts">
  interface Props {
    isRecording?: boolean;
    recordingTime?: number;
    onRecordStart?: () => void;
    onRecordStop?: () => void;
  }

  let { 
    isRecording = $bindable(false), 
    recordingTime = $bindable(0),
    onRecordStart,
    onRecordStop 
  }: Props = $props();

  let timerInterval: ReturnType<typeof setInterval> | null = $state(null);

  function formatTime(seconds: number): string {
    if (!seconds || isNaN(seconds)) return "0:00";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  function startTimer() {
    if (timerInterval) return;
    recordingTime = 0;
    timerInterval = setInterval(() => {
      recordingTime += 1;
    }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  // Watch for isRecording changes to start/stop timer
  $effect(() => {
    if (isRecording) {
      startTimer();
    } else {
      stopTimer();
      recordingTime = 0;
    }
  });

  function handleClick() {
    if (isRecording) {
      if (onRecordStop) {
        onRecordStop();
      }
    } else {
      if (onRecordStart) {
        onRecordStart();
      }
    }
  }

  // Cleanup on unmount
  $effect(() => {
    return () => {
      stopTimer();
    };
  });
</script>

<div class="record-button-container">
  <button 
    class="record-button" 
    class:recording={isRecording}
    onclick={handleClick}
    disabled={isRecording === undefined}
    aria-label={isRecording ? "Stop recording" : "Start recording"}
  >
    {#if isRecording}
      <span class="record-indicator"></span>
      <span class="button-text">⏹ Stop Recording</span>
      <span class="timer">{formatTime(recordingTime)}</span>
    {:else}
      <span class="button-text">🎥 Record Screen</span>
    {/if}
  </button>
</div>

<style>
  .record-button-container {
    display: inline-block;
  }

  .record-button {
    background: var(--accent, #396cd8);
    color: white;
    border: none;
    border-radius: 4px;
    padding: 0.375rem 0.75rem;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 120px;
    justify-content: center;
  }

  .record-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  }

  .record-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .record-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .record-button.recording {
    background: #dc2626;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.8;
    }
  }

  .record-indicator {
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

  .button-text {
    white-space: nowrap;
  }

  .timer {
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.75rem;
    background: rgba(0, 0, 0, 0.2);
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    min-width: 40px;
    text-align: center;
  }

  @media (prefers-color-scheme: dark) {
    .record-button {
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    }

    .record-button:hover:not(:disabled) {
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.4);
    }

    .timer {
      background: rgba(0, 0, 0, 0.4);
    }
  }
</style>

