import type { NumberField } from "../../types";
import { inputClass } from "../ui";

export function NumberFieldEditor({
  field,
  onChange,
  value,
}: {
  field: NumberField;
  onChange: (value: unknown) => void;
  value: unknown;
}) {
  return (
    <input
      className={inputClass}
      disabled={field.readOnly}
      max={field.max}
      min={field.min}
      onChange={(event) => onChange(event.target.value === "" ? "" : Number(event.target.value))}
      placeholder={field.placeholder}
      required={field.required}
      type="number"
      value={typeof value === "number" ? value : ""}
    />
  );
}
