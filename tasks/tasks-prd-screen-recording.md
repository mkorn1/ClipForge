# Task List: Screen Recording Feature

**Based on:** `prd-screen-recording.md`  
**Status:** In Progress  
**Created:** 2024

---

## Relevant Files

- `src/lib/components/RecordButton.svelte` - New component for recording controls with start/stop functionality and timer display
- `src/lib/components/RecordingPreview.svelte` - New component for displaying live MediaStream during recording (or extend VideoPlayer.svelte)
- `src/routes/+page.svelte` - Main page component requiring updates to handle recording state and live preview display
- `src-tauri/src/lib.rs` - Rust backend file requiring new Tauri commands for FFmpeg recording process management
- `src-tauri/Cargo.toml` - Dependency manifest file requiring potential tokio dependency addition
- `src/lib/components/VideoPlayer.svelte` - May need extension to handle MediaStream objects in addition to video URLs
- `docs/README.md` or `docs/INSTALLATION.md` - Documentation requiring FFmpeg installation instructions

### Notes

- Unit tests should be created for new components and Tauri commands following existing test patterns (if tests exist in the codebase)
- FFmpeg must be installed on the user's system - this should be documented clearly
- Consider adding integration tests for the recording workflow end-to-end

---

## Tasks

- [x] 1.0 Backend: Implement Screen Recording Tauri Commands
  - [x] 1.1 Check if `tokio` dependency is present in `Cargo.toml`, add if missing for async process handling
  - [x] 1.2 Add new data structures in `lib.rs` for recording: `RecordingProcessId` (to track FFmpeg process) and `RecordingResult` (to return file path and metadata)
  - [x] 1.3 Implement `start_screen_recording` Tauri command that accepts output file path parameter
  - [x] 1.4 In `start_screen_recording`, generate unique temporary file path using timestamp format: `clipforge-recording-{timestamp}.mp4`
  - [x] 1.5 In `start_screen_recording`, construct FFmpeg command for macOS using `avfoundation` input format: `ffmpeg -f avfoundation -i "1:0" -video_size 1920x1080 -framerate 30 -pix_fmt yuv420p {output_path}` (Updated to use device "4:" for screen capture)
  - [x] 1.6 In `start_screen_recording`, spawn FFmpeg process asynchronously and store process handle (use `tokio::process::Command` or `std::process::Command`)
  - [x] 1.7 In `start_screen_recording`, return unique process identifier to frontend for tracking
  - [x] 1.8 Implement `stop_screen_recording` Tauri command that accepts process ID parameter
  - [x] 1.9 In `stop_screen_recording`, locate and terminate the FFmpeg process using the process ID
  - [x] 1.10 In `stop_screen_recording`, wait for FFmpeg process to complete and ensure file is fully written
  - [x] 1.11 In `stop_screen_recording`, verify the output file exists and return the file path to frontend
  - [x] 1.12 Implement `check_screen_recording_permission` Tauri command for macOS (use `CGPreflightScreenCaptureAccess` or similar API if available, or return a placeholder)
  - [x] 1.13 Add error handling for FFmpeg not found (check if `ffmpeg` command exists before starting)
  - [x] 1.14 Add error handling for process spawn failures and file write failures
  - [x] 1.15 Register all new Tauri commands in the `invoke_handler` within the `run()` function
  - [x] 1.16 Add proper error types and serialization for all command results (use `Result<T, String>` pattern)

- [x] 2.0 Frontend: Create RecordButton Component
  - [x] 2.1 Create new file `src/lib/components/RecordButton.svelte`
  - [x] 2.2 Define component props interface: `onRecordStart?: () => void` and `onRecordStop?: () => void`
  - [x] 2.3 Add reactive state for `isRecording` using Svelte 5 `$state` reactivity
  - [x] 2.4 Add reactive state for `recordingTime` (seconds) using Svelte 5 `$state`
  - [x] 2.5 Create `formatTime` helper function to format seconds as `MM:SS` display
  - [x] 2.6 Implement `startTimer` function that increments `recordingTime` every second using `setInterval`
  - [x] 2.7 Implement `stopTimer` function that clears the interval
  - [x] 2.8 Create main button element with conditional rendering: show "🎥 Record Screen" when idle, "⏹ Stop Recording" when recording
  - [x] 2.9 Add visual indicator (red dot/badge) that appears when `isRecording` is true
  - [x] 2.10 Display recording timer next to button text when recording: `MM:SS` format
  - [x] 2.11 Style button to match `ImportButton.svelte` styling (referencing existing styles)
  - [x] 2.12 Add distinct styling for recording state (e.g., red background/border, pulsing effect)
  - [x] 2.13 Implement `handleClick` function that calls `onRecordStart` when idle or `onRecordStop` when recording
  - [x] 2.14 Add proper cleanup in component destructor to clear timer interval if component unmounts during recording
  - [x] 2.15 Ensure button is disabled or shows loading state during recording start/stop transitions

- [x] 3.0 Frontend: Implement Live Preview Display
  - [x] 3.1 Decide between extending `VideoPlayer.svelte` or creating `RecordingPreview.svelte` (recommendation: create new component for clarity)
  - [x] 3.2 Create new file `src/lib/components/RecordingPreview.svelte`
  - [x] 3.3 Define component props: `stream: MediaStream | null` and optional `onStreamEnd?: () => void`
  - [x] 3.4 Create video element reference using Svelte `bind:this`
  - [x] 3.5 Implement `$effect` that sets `videoElement.srcObject = stream` when stream prop changes
  - [x] 3.6 Implement `$effect` cleanup to stop all tracks in MediaStream when component unmounts or stream changes
  - [x] 3.7 Ensure video element has `autoplay` and `playsinline` attributes
  - [x] 3.8 Style video element to maintain aspect ratio and fill preview container (match `VideoPlayer.svelte` video container styles)
  - [x] 3.9 Add optional "RECORDING" overlay text/banner to clearly indicate live recording state
  - [x] 3.10 Handle case when stream is null (show placeholder or hide component)
  - [x] 3.11 Ensure video element does not show controls (since this is live preview, not playback)
  - [x] 3.12 Match the visual container styling from `VideoPlayer.svelte` for consistency

- [x] 4.0 Frontend: Integrate Recording State Management in Main Page
  - [x] 4.1 Add `isRecording` state variable to `+page.svelte` using `$state` (default: `false`)
  - [x] 4.2 Add `recordingStream` state variable to `+page.svelte` using `$state<MediaStream | null>` (default: `null`)
  - [x] 4.3 Add `recordingProcessId` state variable to `+page.svelte` using `$state<number | null>` (default: `null`)
  - [x] 4.4 Import `RecordButton` component and add it to the panel header next to `ImportButton`
  - [x] 4.5 Import `RecordingPreview` component
  - [x] 4.6 Modify preview panel conditional rendering: add `{#if isRecording}` block before existing `{#if selectedClip}` block
  - [x] 4.7 In recording block, render `RecordingPreview` component with `recordingStream` prop (with fallback placeholder when no stream)
  - [x] 4.8 Implement `handleRecordStart` async function
  - [x] 4.9 In `handleRecordStart`, request screen capture using `navigator.mediaDevices.getDisplayMedia({ video: true, audio: false })` (with fallback if API unavailable)
  - [x] 4.10 In `handleRecordStart`, set `recordingStream` to the returned MediaStream
  - [x] 4.11 In `handleRecordStart`, call Tauri `invoke('start_screen_recording', { outputPath })` with generated temp path
  - [x] 4.12 In `handleRecordStart`, store returned process ID in `recordingProcessId` state
  - [x] 4.13 In `handleRecordStart`, set `isRecording` to `true`
  - [x] 4.14 Implement `handleRecordStop` async function
  - [x] 4.15 In `handleRecordStop`, stop all tracks in `recordingStream` using `recordingStream.getTracks().forEach(track => track.stop())`
  - [x] 4.16 In `handleRecordStop`, set `recordingStream` to `null`
  - [x] 4.17 In `handleRecordStop`, call Tauri `invoke('stop_screen_recording', { processId: recordingProcessId })`
  - [x] 4.18 In `handleRecordStop`, store returned file path from backend
  - [x] 4.19 In `handleRecordStop`, set `isRecording` to `false` and reset `recordingProcessId` to `null`
  - [x] 4.20 Pass `handleRecordStart` and `handleRecordStop` to `RecordButton` as props
  - [x] 4.21 Ensure `selectedClip` is cleared when recording starts (so preview shows recording, not old clip)
  - [x] 4.22 Handle case where user denies `getDisplayMedia()` permission (catch error, show message, don't start backend recording)

- [x] 5.0 Permissions: Add macOS Screen Recording Permission Handling
  - [x] 5.1 Create helper function `checkScreenRecordingPermission` that calls Tauri `invoke('check_screen_recording_permission')`
  - [ ] 5.2 Call `checkScreenRecordingPermission` on app mount or when RecordButton is first rendered (deferred - permission checked via getDisplayMedia)
  - [x] 5.3 If permission is denied, show user-friendly message with instructions: "Screen recording permission is required. Please enable it in System Preferences > Security & Privacy > Privacy > Screen Recording."
  - [ ] 5.4 Create permission error message component or UI state to display permission guidance (basic alert implemented, could be enhanced)
  - [ ] 5.5 Handle permission denial gracefully - disable RecordButton or show warning icon when permission denied (errors shown, but no proactive disabling)
  - [x] 5.6 Handle `getDisplayMedia()` user cancellation (user closes browser permission dialog) - don't show error, just reset state
  - [ ] 5.7 Add link or button to open System Preferences (if Tauri provides this capability) for easier permission access
  - [x] 5.8 Ensure permission check happens before attempting to start recording (checked via getDisplayMedia)
  - [ ] 5.9 Show loading/checking state while verifying permissions

- [x] 6.0 Integration: Auto-Add Recordings to Timeline
  - [x] 6.1 In `handleRecordStop`, after receiving file path from backend, read the file using `@tauri-apps/plugin-fs` `readFile` function
  - [x] 6.2 Create Blob from file data with MIME type `video/mp4`
  - [x] 6.3 Generate blob URL using `URL.createObjectURL(blob)`
  - [x] 6.4 Extract filename from file path for clip name (format: `screen-recording-{timestamp}.mp4`)
  - [x] 6.5 Call existing `getVideoDuration` helper function (from `+page.svelte`) to get video duration from blob URL
  - [x] 6.6 Calculate start time for new clip: if timeline has clips, use last clip's `endTime`, otherwise use `0`
  - [x] 6.7 Create new `TimelineClip` object with: `id` (generated), `videoUrl` (blob URL), `videoPath` (original file path), `name` (filename), `duration`, `startTime`, `endTime`
  - [x] 6.8 Add new clip to `timelineClips` array using spread operator: `timelineClips = [...timelineClips, newClip]`
  - [x] 6.9 Set `selectedClip` to the newly created clip: `selectedClip = newClip`
  - [x] 6.10 Ensure preview switches from recording view to standard video player automatically
  - [ ] 6.11 Show success notification or visual feedback that recording was saved and added (optional, but recommended)
  - [x] 6.12 Handle errors during file read or blob creation - show error message, don't crash

- [x] 7.0 Error Handling and Edge Cases
  - [x] 7.1 Handle `getDisplayMedia()` errors: user denial, API not available, browser compatibility
  - [x] 7.2 Handle FFmpeg process errors: process not found, spawn failure, execution errors
  - [x] 7.3 Handle file save errors: disk full, permission denied, path invalid
  - [x] 7.4 Add try-catch blocks around all async operations in `handleRecordStart` and `handleRecordStop`
  - [x] 7.5 Display user-friendly error messages in UI (toast, alert, or inline error state)
  - [x] 7.6 Ensure cleanup happens even on errors: stop MediaStream tracks, clean up temp files if possible
  - [ ] 7.7 Handle case where FFmpeg process is killed externally or crashes - detect and reset state (basic handling exists, could be enhanced)
  - [x] 7.8 Prevent multiple simultaneous recordings - disable RecordButton while `isRecording` is true
  - [ ] 7.9 Handle app closure during recording - ensure MediaStream stops (Tauri may handle this, but verify)
  - [x] 7.10 Add timeout mechanism for FFmpeg process completion (if it doesn't stop in reasonable time) (wait with delay implemented)
  - [x] 7.11 Validate that blob URL creation succeeds before adding to timeline
  - [x] 7.12 Handle case where video duration cannot be determined - use fallback duration or show error
  - [x] 7.13 Add console error logging for debugging (but not expose to user in production)
  - [x] 7.14 Ensure error states don't leave UI in broken state (always reset `isRecording`, clear streams)

- [ ] 8.0 Documentation and User Guidance
  - [ ] 8.1 Add FFmpeg installation instructions to `README.md` or create `docs/INSTALLATION.md`
  - [ ] 8.2 Document macOS screen recording permission requirements
  - [ ] 8.3 Add step-by-step guide for enabling screen recording permission
  - [ ] 8.4 Document supported recording settings (resolution, frame rate, format)
  - [ ] 8.5 Add troubleshooting section for common recording issues (FFmpeg not found, permission denied, etc.)
  - [ ] 8.6 Update main `README.md` to mention screen recording feature in features list
  - [ ] 8.7 Add inline help text or tooltips for RecordButton explaining the feature (optional but recommended)
  - [ ] 8.8 Document any known limitations (no audio, entire screen only, etc.)
  - [ ] 8.9 Add code comments in key functions explaining the recording workflow
  - [ ] 8.10 Consider adding a "First Time Recording" onboarding tooltip or modal (optional enhancement)

