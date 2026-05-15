import type { BooleanField } from "../../types";

export function BooleanFieldEditor({
  field,
  label,
  checked,
  onChange,
}: {
  field: BooleanField;
  label: string;
  checked: boolean;
  onChange: (value: unknown) => void;
}) {
  return (
    <label className="flex items-center justify-between rounded-md border border-zinc-200 bg-white px-3 py-2 dark:border-zinc-800 dark:bg-zinc-900">
      <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">{label}</span>
      <input
        checked={checked}
        className="size-4 disabled:cursor-not-allowed disabled:opacity-50"
        disabled={field.readOnly}
        onChange={(event) => onChange(event.target.checked)}
        type="checkbox"
      />
    </label>
  );
}
