<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { readFile } from "@tauri-apps/plugin-fs";

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
        const videoFiles: VideoFile[] = await Promise.all(
          files.map(async (path) => {
            const name = path.split(/[/\\]/).pop() || "Unknown";
            
            try {
              // Read the file and create a blob URL
              const fileData = await readFile(path);
              const mimeType = getMimeType(name);
              const blob = new Blob([fileData], { type: mimeType });
              const url = URL.createObjectURL(blob);
              
              console.log("Importing file:", name);
              console.log("Path:", path);
              console.log("File size:", fileData.length);
              console.log("MIME type:", mimeType);
              console.log("Created blob URL:", url);
              console.log("Blob size:", blob.size);
              
              return {
                path,
                name,
                url,
                size: fileData.length,
              };
            } catch (error) {
              console.error(`Error reading file ${name}:`, error);
              throw error;
            }
          })
        );
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
      const filePromises = Array.from(files)
        .filter((file) => {
          const ext = file.name.split(".").pop()?.toLowerCase();
          return ["mp4", "mov", "avi", "mkv", "webm"].includes(ext || "");
        })
        .map(async (file) => {
          // For Tauri, we need to read the file content
          const arrayBuffer = await file.arrayBuffer();
          const blob = new Blob([arrayBuffer], { type: file.type || 'video/*' });
          const url = URL.createObjectURL(blob);
          
          return {
            path: (file as any).path || file.name,
            name: file.name,
            url,
            size: file.size,
          };
        });
      
      Promise.all(filePromises).then((videoFiles) => {
        if (videoFiles.length > 0) {
          onImport(videoFiles);
        }
      });
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
    padding: 1rem;
    border: 2px dashed var(--border, #ccc);
    border-radius: 6px;
    background: var(--bg-tertiary, #f9f9f9);
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
    border-radius: 6px;
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
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
