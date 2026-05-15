import { inputClass } from "./ui";

export function SlugInput({
  onChange,
  onGenerate,
  value,
}: {
  onChange: (value: string) => void;
  onGenerate?: () => void;
  value: string;
}) {
  return (
    <div className="flex gap-2">
      <input
        className={inputClass}
        onChange={(event) => onChange(event.target.value)}
        placeholder="optional-url-slug"
        value={value}
      />
      <button
        className="shrink-0 rounded-md border border-zinc-200 px-3 py-2 text-sm text-zinc-700 hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
        onClick={onGenerate}
        type="button"
      >
        Generate
      </button>
    </div>
  );
}
