import type { ChangeEvent } from "react";

import { uploadFile } from "../../lib/api";
import type { ImageField } from "../../types";
import { inputClass } from "../ui";

export function ImageFieldEditor({
  field,
  onChange,
  value,
}: {
  field: ImageField;
  onChange: (value: unknown) => void;
  value: string;
}) {
  function handleUpload(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;
    void uploadFile(file).then((body) => onChange(body.url));
  }

  return (
    <div className="space-y-2">
      {value ? (
        <img
          alt=""
          className="aspect-video w-full max-w-md rounded-md border border-zinc-200 object-cover dark:border-zinc-800"
          src={value}
        />
      ) : null}
      <input
        className={inputClass}
        disabled={field.readOnly}
        onChange={(event) => onChange(event.target.value)}
        placeholder={field.placeholder ?? "https://..."}
        required={field.required}
        type="url"
        value={value}
      />
      {!field.readOnly ? (
        <input
          accept={field.accept ?? "image/*"}
          className="block w-full text-sm text-zinc-500 file:mr-3 file:rounded-md file:border-0 file:bg-zinc-100 file:px-3 file:py-2 file:text-sm file:text-zinc-700 dark:text-zinc-400 dark:file:bg-zinc-800 dark:file:text-zinc-200"
          onChange={handleUpload}
          type="file"
        />
      ) : null}
    </div>
  );
}
