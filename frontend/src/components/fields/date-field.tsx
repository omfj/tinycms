import type { DateField } from "../../types";
import { inputClass } from "../ui";

export function DateFieldEditor({
  field,
  onChange,
  value,
}: {
  field: DateField;
  onChange: (value: unknown) => void;
  value: string;
}) {
  return (
    <input
      className={inputClass}
      disabled={field.readOnly}
      max={field.max != null ? new Date(field.max).toISOString().slice(0, 16) : undefined}
      min={field.min != null ? new Date(field.min).toISOString().slice(0, 16) : undefined}
      onChange={(event) => onChange(event.target.value)}
      required={field.required}
      type="datetime-local"
      value={value}
    />
  );
}
