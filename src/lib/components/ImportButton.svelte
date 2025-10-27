<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";

  interface VideoFile {
    path: string;
    name: string;
    url: string;
    size?: number;
  }

  interface Props {
    onImport?: (files: VideoFile[]) => void;
  }

  let { onImport }: Props = $props();

  let isDragging = $state(false);

  async function handleImport() {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Video",
            extensions: ["mp4", "mov", "avi", "mkv", "webm"],
          },
        ],
      });

      if (selected && onImport) {
        const files = Array.isArray(selected) ? selected : [selected];
        const videoFiles: VideoFile[] = files.map((path) => {
          const name = path.split(/[/\\]/).pop() || "Unknown";
          return {
            path,
            name,
            url: convertFileSrc(path),
          };
        });
        onImport(videoFiles);
      }
    } catch (error) {
      console.error("Error importing files:", error);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave() {
    isDragging = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    const files = e.dataTransfer?.files;
    if (files && onImport) {
      const videoFiles: VideoFile[] = Array.from(files)
        .filter((file) => {
          const ext = file.name.split(".").pop()?.toLowerCase();
          return ["mp4", "mov", "avi", "mkv", "webm"].includes(ext || "");
        })
        .map((file) => {
          return {
            path: (file as any).path || file.name,
            name: file.name,
            url: URL.createObjectURL(file),
            size: file.size,
          };
        });
      
      if (videoFiles.length > 0) {
        onImport(videoFiles);
      }
    }
  }
</script>

<div
  class="import-area"
  class:dragging={isDragging}
  role="region"
  aria-label="Import video files"
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
>
  <button class="import-button" onclick={handleImport}>
    📁 Import Video
  </button>
  <p class="import-hint">or drag and drop video files here</p>
</div>

<style>
  .import-area {
    padding: 2rem;
    border: 2px dashed var(--border, #ccc);
    border-radius: 8px;
    background: var(--bg-secondary, #f9f9f9);
    text-align: center;
    transition: all 0.2s;
  }

  .import-area.dragging {
    border-color: var(--accent, #396cd8);
    background: var(--bg-drag, #e8f0fe);
  }

  .import-button {
    background: var(--accent, #396cd8);
    color: white;
    border: none;
    border-radius: 8px;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .import-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  }

  .import-button:active {
    transform: translateY(0);
  }

  .import-hint {
    margin: 0.5rem 0 0 0;
    color: var(--text-secondary, #666);
    font-size: 0.9rem;
  }

  @media (prefers-color-scheme: dark) {
    .import-area {
      background: var(--bg-secondary, #1f1f1f);
      border-color: var(--border, #444);
    }

    .import-area.dragging {
      background: var(--bg-drag, #2a3a4a);
      border-color: var(--accent, #396cd8);
    }

    .import-hint {
      color: var(--text-secondary, #ccc);
    }
  }
</style>
