import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";

import { inputClass } from "../components/ui";
import { deleteUpload, listUploads, updateUpload, uploadFile } from "../lib/api";
import { classNames, formatDate } from "../lib/format";
import type { Media } from "../types";

function formatBytes(bytes: number) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function isImage(contentType: string) {
  return contentType.startsWith("image/");
}

function FileIcon() {
  return (
    <svg
      aria-hidden="true"
      className="size-10 text-zinc-300 dark:text-zinc-600"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      viewBox="0 0 24 24"
    >
      <path
        d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function MediaPage() {
  const [items, setItems] = useState<Media[]>([]);
  const [selected, setSelected] = useState<Media | null>(null);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [label, setLabel] = useState("");
  const [savingLabel, setSavingLabel] = useState(false);
  const [copied, setCopied] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    void load();
  }, []);

  useEffect(() => {
    setLabel(selected?.label ?? "");
    setCopied(false);
  }, [selected]);

  async function load() {
    setLoading(true);
    try {
      const data = await listUploads();
      setItems(data);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not load media");
    } finally {
      setLoading(false);
    }
  }

  async function handleUpload(files: FileList | null) {
    if (!files || files.length === 0) return;
    setUploading(true);
    try {
      const uploaded: Media[] = [];
      for (const file of Array.from(files)) {
        const media = (await uploadFile(file)) as unknown as Media;
        uploaded.push(media);
      }
      setItems((prev) => [...uploaded, ...prev]);
      if (uploaded.length === 1) setSelected(uploaded[0]);
      toast.success(`Uploaded ${uploaded.length} file${uploaded.length > 1 ? "s" : ""}`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Upload failed");
    } finally {
      setUploading(false);
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  }

  async function handleSaveLabel() {
    if (!selected) return;
    setSavingLabel(true);
    try {
      const updated = await updateUpload(selected.id, label || null);
      setItems((prev) => prev.map((item) => (item.id === updated.id ? updated : item)));
      setSelected(updated);
      toast.success("Label saved");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not save label");
    } finally {
      setSavingLabel(false);
    }
  }

  async function handleDelete() {
    if (!selected) return;
    if (!confirm(`Delete "${selected.filename}"? This cannot be undone.`)) return;
    try {
      await deleteUpload(selected.id);
      setItems((prev) => prev.filter((item) => item.id !== selected.id));
      setSelected(null);
      toast.success("Deleted");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not delete");
    }
  }

  function copyUrl() {
    if (!selected) return;
    void navigator.clipboard.writeText(selected.url).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  return (
    <main
      className={classNames(
        "grid h-full min-h-0",
        selected ? "grid-cols-[14rem_minmax(0,1fr)_22rem]" : "grid-cols-[14rem_minmax(0,1fr)]",
      )}
    >
      {/* Left sidebar */}
      <aside className="flex min-h-0 flex-col border-r border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950">
        <div className="flex h-14 items-center border-b border-zinc-200 px-4 dark:border-zinc-800">
          <span className="text-sm font-semibold">Media</span>
        </div>

        <div className="flex-1 overflow-auto p-4">
          <p className="mb-3 text-xs text-zinc-500 dark:text-zinc-400">
            {items.length} file{items.length !== 1 ? "s" : ""}
          </p>

          <button
            className="w-full rounded-md border border-zinc-200 px-3 py-2 text-sm hover:bg-zinc-50 disabled:opacity-50 dark:border-zinc-800 dark:hover:bg-zinc-900"
            disabled={uploading}
            onClick={() => fileInputRef.current?.click()}
            type="button"
          >
            {uploading ? "Uploading…" : "Upload files"}
          </button>

          <input
            accept="*/*"
            className="hidden"
            multiple
            onChange={(e) => void handleUpload(e.target.files)}
            ref={fileInputRef}
            type="file"
          />
        </div>
      </aside>

      {/* Grid */}
      <div className="min-h-0 overflow-auto p-6">
        {loading ? (
          <p className="text-sm text-zinc-500">Loading…</p>
        ) : items.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center gap-3 text-zinc-400">
            <FileIcon />
            <p className="text-sm">No files uploaded yet</p>
          </div>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-3">
            {items.map((item) => (
              <button
                className={classNames(
                  "group flex flex-col overflow-hidden rounded-lg border text-left transition-colors",
                  item.id === selected?.id
                    ? "border-zinc-950 ring-2 ring-zinc-950 dark:border-zinc-50 dark:ring-zinc-50"
                    : "border-zinc-200 hover:border-zinc-300 dark:border-zinc-800 dark:hover:border-zinc-700",
                )}
                key={item.id}
                onClick={() => setSelected(item.id === selected?.id ? null : item)}
                type="button"
              >
                <div className="flex aspect-square w-full items-center justify-center bg-zinc-100 dark:bg-zinc-900">
                  {isImage(item.content_type) ? (
                    <img
                      alt={item.label ?? item.filename}
                      className="h-full w-full object-cover"
                      src={item.url}
                    />
                  ) : (
                    <FileIcon />
                  )}
                </div>
                <div className="w-full p-2">
                  <p className="truncate text-xs font-medium">{item.label ?? item.filename}</p>
                  <p className="text-xs text-zinc-400 dark:text-zinc-500">
                    {formatBytes(item.size)}
                  </p>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Inspect panel */}
      {selected && (
        <aside className="flex min-h-0 flex-col border-l border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950">
          <div className="flex h-14 items-center justify-between border-b border-zinc-200 px-4 dark:border-zinc-800">
            <span className="truncate text-sm font-semibold">{selected.filename}</span>
            <button
              className="ml-2 shrink-0 rounded-md p-1 text-zinc-400 hover:bg-zinc-100 hover:text-zinc-700 dark:hover:bg-zinc-900 dark:hover:text-zinc-200"
              onClick={() => setSelected(null)}
              title="Close"
              type="button"
            >
              ✕
            </button>
          </div>

          <div className="min-h-0 flex-1 overflow-auto">
            {/* Preview */}
            <div className="border-b border-zinc-200 dark:border-zinc-800">
              {isImage(selected.content_type) ? (
                <img
                  alt={selected.label ?? selected.filename}
                  className="w-full object-contain"
                  src={selected.url}
                />
              ) : (
                <div className="flex aspect-video items-center justify-center bg-zinc-50 dark:bg-zinc-900">
                  <FileIcon />
                </div>
              )}
            </div>

            <div className="space-y-4 p-4">
              {/* Metadata */}
              <dl className="space-y-2 text-sm">
                <div className="flex justify-between gap-2">
                  <dt className="text-zinc-500 dark:text-zinc-400">Type</dt>
                  <dd className="truncate text-right font-mono text-xs">{selected.content_type}</dd>
                </div>
                <div className="flex justify-between gap-2">
                  <dt className="text-zinc-500 dark:text-zinc-400">Size</dt>
                  <dd>{formatBytes(selected.size)}</dd>
                </div>
                <div className="flex justify-between gap-2">
                  <dt className="text-zinc-500 dark:text-zinc-400">Uploaded</dt>
                  <dd>{formatDate(selected.created_at)}</dd>
                </div>
              </dl>

              {/* URL */}
              <div>
                <p className="mb-1 text-xs font-medium text-zinc-500 dark:text-zinc-400">URL</p>
                <div className="flex gap-2">
                  <input
                    className={classNames(inputClass, "flex-1 truncate font-mono text-xs")}
                    readOnly
                    value={selected.url}
                  />
                  <button
                    className="shrink-0 rounded-md border border-zinc-200 px-2.5 py-1.5 text-xs hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900"
                    onClick={copyUrl}
                    type="button"
                  >
                    {copied ? "Copied" : "Copy"}
                  </button>
                </div>
              </div>

              {/* Label */}
              <div>
                <label className="mb-1 block text-xs font-medium text-zinc-500 dark:text-zinc-400">
                  Label
                </label>
                <input
                  className={classNames(inputClass, "w-full")}
                  onChange={(e) => setLabel(e.target.value)}
                  placeholder="Add a label or alt text…"
                  value={label}
                />
                <button
                  className="mt-2 w-full rounded-md bg-zinc-950 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50 dark:bg-zinc-50 dark:text-zinc-950"
                  disabled={savingLabel || label === (selected.label ?? "")}
                  onClick={() => void handleSaveLabel()}
                  type="button"
                >
                  {savingLabel ? "Saving…" : "Save label"}
                </button>
              </div>

              {/* Delete */}
              <div className="border-t border-zinc-200 pt-4 dark:border-zinc-800">
                <button
                  className="w-full rounded-md px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                  onClick={() => void handleDelete()}
                  type="button"
                >
                  Delete file
                </button>
              </div>
            </div>
          </div>
        </aside>
      )}
    </main>
  );
}
