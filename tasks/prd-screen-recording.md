# Product Requirements Document: ClipForge
## Screen Recording Feature

**Version:** 1.0  
**Date:** 2025  
**Status:** Ready for Development  
**Product:** ClipForge - Fast Video Editor

---

## 1. Introduction/Overview

### 1.1 Overview
This feature enables users to record their screen directly within ClipForge, with live preview displayed in the preview panel. Upon completing or stopping the recording, the recorded video is automatically added to the timeline, providing a seamless workflow from recording to editing.

### 1.2 Problem Statement
Currently, users must record their screen using external tools, then import the recording into ClipForge for editing. This creates friction in the workflow. By integrating screen recording directly into ClipForge, users can quickly capture screen content and immediately begin editing without switching applications.

### 1.3 Goal
Enable users to record their screen with live preview feedback, then automatically add the recording to the timeline for immediate editing.

---

## 2. Goals

1. **Primary Goal:** Users can record screen content directly within ClipForge with real-time visual feedback
2. **Secondary Goal:** Recorded videos automatically appear in the timeline without manual import steps
3. **Quality Goal:** Recordings maintain high quality suitable for editing workflows
4. **UX Goal:** The recording experience feels intuitive and provides clear feedback about recording status

---

## 3. User Stories

### Story 1: Start Screen Recording
**As a** content creator  
**I want** to start recording my screen from within ClipForge  
**So that** I can capture content without leaving the application  

**Acceptance Criteria:**
- User can click a "Record Screen" button in the preview panel header
- Clicking the button initiates screen recording after requesting necessary permissions
- A live preview of the screen capture appears in the preview panel
- The button changes to show recording state (e.g., "Stop Recording")
- A recording timer displays showing elapsed recording time
- Visual indicator (e.g., red dot) shows recording is active

### Story 2: View Live Recording Preview
**As a** user  
**I want** to see a live preview of what I'm recording  
**So that** I can verify the recording is capturing the correct content  

**Acceptance Criteria:**
- While recording, the preview panel displays a live stream of the screen capture
- The preview replaces any currently selected clip view
- Preview updates in real-time with minimal latency
- Preview maintains proper aspect ratio
- Preview is clearly distinguishable from regular video playback (optional: show "RECORDING" overlay)

### Story 3: Stop Recording and Add to Timeline
**As a** user  
**I want** to stop recording and have it automatically added to my timeline  
**So that** I can immediately begin editing without additional steps  

**Acceptance Criteria:**
- User can stop recording by clicking the "Stop Recording" button
- Recording stops and the file is saved with a descriptive name (e.g., "screen-recording-2024-01-15-14-30.mp4")
- The recorded video automatically appears on the timeline at the end position
- The newly recorded clip is automatically selected and displayed in the preview panel
- The preview switches from live recording view to standard video player for the recorded file
- User receives confirmation that recording was saved successfully

### Story 4: Handle Screen Recording Permissions
**As a** user on macOS  
**I want** clear guidance when screen recording permissions are needed  
**So that** I can successfully enable recording without confusion  

**Acceptance Criteria:**
- Application checks for screen recording permission before starting
- If permission is not granted, user sees clear instructions on how to enable it
- Permission request is made at the appropriate time (system dialog)
- If permission is denied, user sees a helpful error message with steps to enable it manually
- Application gracefully handles permission denial without crashing

---

## 4. Functional Requirements

### 4.1 Recording Controls
1. The system must provide a "Record Screen" button in the preview panel header, positioned next to the Import button
2. The button must change its label and appearance when recording is active (e.g., "Stop Recording")
3. The system must display elapsed recording time while recording is active
4. The system must provide visual feedback indicating recording status (e.g., red dot indicator)

### 4.2 Live Preview
5. While recording is active, the preview panel must display a live stream of the screen capture
6. The live preview must replace the default "no preview" state and any selected clip preview
7. The live preview must update in real-time with minimal noticeable latency
8. The preview must maintain the aspect ratio of the screen capture

### 4.3 Screen Capture
9. The system must use browser `getDisplayMedia()` API to capture screen content for preview
10. The system must use FFmpeg (via Rust backend) to record high-quality screen capture to a video file
11. The system must request screen recording permission from macOS before initiating capture
12. The system must handle permission denial gracefully with user guidance

### 4.4 File Management
13. Recorded files must be saved with descriptive filenames including timestamp
14. Recorded files must be saved in a temporary directory that persists for the session
15. Recorded files must be in MP4 format (H.264 codec) for compatibility
16. The system must generate blob URLs for recorded files to enable playback in the video player

### 4.5 Timeline Integration
17. Upon stopping recording, the system must automatically add the recorded video as a clip to the timeline
18. The new clip must be positioned at the end of existing clips on the timeline
19. The new clip must be automatically selected upon creation
20. The new clip must appear in the preview panel using the standard video player component

### 4.6 Error Handling
21. The system must handle errors if screen capture fails to start
22. The system must handle errors if recording file cannot be saved
23. The system must display clear, actionable error messages to the user
24. The system must clean up temporary files if recording fails or is cancelled

### 4.7 State Management
25. The system must prevent starting a new recording while one is already in progress
26. The system must properly stop all recording processes when the user stops recording
27. The system must synchronize frontend preview stream and backend recording process
28. The system must handle application closure during recording gracefully (optional: save partial recording)

---

## 5. Non-Goals (Out of Scope)

1. **Audio Recording:** This feature does not include capturing system audio or microphone input during screen recording (future enhancement)
2. **Recording Region Selection:** Users cannot select specific screen regions to record - entire screen capture only (future enhancement)
3. **Recording Quality Settings:** No user-configurable quality, resolution, or frame rate settings (uses defaults)
4. **Multiple Recording Sessions:** Cannot record multiple screens simultaneously
5. **Recording Scheduled Start/Stop:** No scheduled recording or automatic stop after time limit
6. **Recording Editing During Capture:** No ability to trim or edit while recording is in progress
7. **Recording Format Selection:** No ability to choose output format - always MP4

---

## 6. Design Considerations

### 6.1 UI/UX
- **Button Placement:** Record button appears in preview panel header, adjacent to Import button, right-aligned
- **Button Styling:** Should match ImportButton styling for consistency, with distinct state for recording (e.g., red background or border when recording)
- **Preview State:** Live recording preview should be visually distinct from regular video playback to avoid confusion
- **Timer Display:** Recording timer should be prominently displayed, possibly integrated into button or shown nearby
- **Visual Feedback:** Use common recording indicators (red dot, pulsing effect) to clearly indicate active recording state

### 6.2 Preview Panel States
The preview panel must handle three states:
1. **Default:** "Click a clip to preview" message (current behavior)
2. **Clip Selected:** Standard VideoPlayer component showing selected clip (current behavior)
3. **Recording:** Live screen capture stream display (new state)

### 6.3 User Flow
```
1. User clicks "Record Screen" button
2. System requests screen recording permission (if needed)
3. User grants permission
4. Screen capture stream starts → Preview panel shows live stream
5. Backend FFmpeg recording process starts
6. Button changes to "Stop Recording" with timer
7. User performs screen actions while recording
8. User clicks "Stop Recording"
9. Frontend stream stops
10. Backend FFmpeg saves file
11. File is automatically added to timeline
12. New clip is auto-selected and preview switches to standard player
```

---

## 7. Technical Considerations

### 7.1 Frontend (Svelte)
- Use `navigator.mediaDevices.getDisplayMedia()` API for live preview stream
- Create new `RecordButton.svelte` component similar to `ImportButton.svelte`
- Extend `+page.svelte` to handle recording state and display live preview
- May need to create `RecordingPreview.svelte` component for displaying MediaStream, or extend `VideoPlayer.svelte` to handle both video URLs and MediaStream objects
- Use Svelte 5 reactivity (`$state`) to manage recording state across components

### 7.2 Backend (Rust/Tauri)
- Implement Tauri commands in `src-tauri/src/lib.rs`:
  - `start_screen_recording(output_path: String)` → Returns process ID or error
  - `stop_screen_recording(process_id: u32)` → Returns saved file path or error
  - `check_screen_recording_permission()` → Returns permission status (macOS specific)
- Execute FFmpeg commands using Rust `std::process::Command` or `tokio::process::Command`
- FFmpeg command structure for macOS:
  ```bash
  ffmpeg -f avfoundation -i "1:0" -video_size 1920x1080 -framerate 30 -pix_fmt yuv420p output.mp4
  ```
- Handle process lifecycle (start, monitor, stop)
- Generate unique temporary file paths for recordings
- Manage FFmpeg process cleanup on errors

### 7.3 Dependencies
- **Frontend:** No additional npm packages required (uses native browser APIs)
- **Backend:** May need `tokio` crate for async process handling (if not already present)
- **System:** FFmpeg must be installed on user's system (documentation requirement)

### 7.4 Platform-Specific Considerations
- **macOS:** Requires Screen Recording permission in System Preferences
- **macOS:** Use `avfoundation` input format for FFmpeg
- **macOS:** May need to handle different display configurations (multiple monitors)
- **Future:** Consider Windows and Linux support with platform-specific FFmpeg input formats

### 7.5 File Handling
- Recordings saved to system temp directory: `std::env::temp_dir()`
- Filename format: `clipforge-recording-{timestamp}.mp4`
- Generate blob URLs from saved files for preview playback
- Consider cleanup of old recordings (optional: keep for session, clean up on close)

### 7.6 State Synchronization
- Frontend must manage MediaStream lifecycle separately from backend FFmpeg process
- Ensure both streams start and stop together
- Handle cases where one fails but the other succeeds
- Consider timeout mechanisms for hanging processes

---

## 8. Success Metrics

### 8.1 Functional Metrics
- 95% of recording attempts complete successfully without errors
- Recordings are saved correctly 99% of the time
- Recording-to-timeline workflow completes in < 3 seconds after stopping
- Live preview updates with < 100ms latency

### 8.2 User Experience Metrics
- Users can successfully start recording on first attempt (after permissions granted)
- Average time from "Record" click to live preview showing: < 2 seconds
- Users report satisfaction with seamless recording-to-editing workflow
- Zero confusion about recording state (clear visual indicators)

### 8.3 Technical Metrics
- No memory leaks from MediaStream objects
- FFmpeg processes properly cleaned up on completion or error
- Application remains responsive during recording (no UI freezing)
- Recording file sizes are reasonable for typical 1-5 minute recordings

---

## 9. Open Questions

1. **FFmpeg Installation:** Should we bundle FFmpeg with the application or require users to install it separately? (Recommendation: Document requirement initially, consider bundling for future release)
2. **Recording Quality Defaults:** What should be the default resolution, frame rate, and quality settings? (Recommendation: Detect screen resolution, 30fps, high quality)
3. **Multiple Monitors:** How should we handle users with multiple displays? (Recommendation: Record primary display, future enhancement for selection)
4. **Partial Recordings:** Should we save recordings if the app is closed during recording? (Recommendation: No for initial version - clean shutdown only)
5. **Recording Limits:** Should there be maximum recording duration limits? (Recommendation: No hard limit, but monitor for disk space)
6. **Error Recovery:** What happens if FFmpeg fails but the frontend stream is working? (Recommendation: Show error, allow user to retry with frontend-only recording as fallback in future)
7. **Audio Delay:** Should we capture audio in future iterations? (Out of scope for this feature, but consider architecture for future)
8. **Recording Format:** Should we allow users to choose recording quality/format? (Out of scope - always MP4, future enhancement)

---

## 10. Related Features

This feature integrates with:
- **Import Feature:** Both Record and Import buttons share the header space and add clips to timeline
- **Timeline Feature:** Recorded videos automatically added to timeline using existing clip structure
- **Preview/VideoPlayer:** Uses existing VideoPlayer component (or extended version) for playback
- **Export Feature:** Recorded videos can be exported using existing export functionality

---

## 11. Implementation Notes

- This feature should build upon existing patterns in the codebase
- The `handleImport` function in `+page.svelte` provides a good pattern for adding clips to timeline automatically
- The `VideoPlayer.svelte` component may need extension to handle MediaStream objects in addition to video URLs
- Consider creating a shared utility for generating blob URLs from file paths
- Error handling should follow existing patterns in the application
- Recording state should be managed at the page level (`+page.svelte`) to coordinate between RecordButton and Preview display

