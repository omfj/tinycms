import type { RichtextField, StringField, TextField, UrlField } from "../../types";
import { inputClass } from "../ui";

type TextComponentField = StringField | TextField | RichtextField | UrlField;

export function TextFieldEditor({
  field,
  onChange,
  value,
}: {
  field: TextComponentField;
  onChange: (value: unknown) => void;
  value: string;
}) {
  if (field.type === "text" || field.type === "richtext") {
    const minHeight = field.type === "richtext" ? "min-h-56" : "min-h-28";
    const style = field.rows ? { height: `${field.rows * 1.5}rem` } : undefined;
    return (
      <textarea
        className={`${inputClass} ${minHeight}`}
        disabled={field.readOnly}
        onChange={(event) => onChange(event.target.value)}
        placeholder={field.placeholder}
        required={field.required}
        style={style}
        value={value}
      />
    );
  }

  return (
    <input
      className={inputClass}
      disabled={field.readOnly}
      onChange={(event) => onChange(event.target.value)}
      pattern={"pattern" in field ? field.pattern : undefined}
      placeholder={field.placeholder}
      required={field.required}
      type={field.type === "url" ? "url" : "text"}
      value={value}
    />
  );
}
