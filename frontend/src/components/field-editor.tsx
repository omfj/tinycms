import { displayType } from "../lib/format";
import type { Field, NumberField, Schema, StringField } from "../types";
import { selectClass } from "./ui";
import { FieldShell } from "./field-shell";
import { ArrayFieldEditor } from "./fields/array-field";
import { BooleanFieldEditor } from "./fields/boolean-field";
import { DateFieldEditor } from "./fields/date-field";
import { ImageFieldEditor } from "./fields/image-field";
import { NumberFieldEditor } from "./fields/number-field";
import { TextFieldEditor } from "./fields/text-field";
import { ReferenceField as ReferenceFieldEditor } from "./reference-field";
import { SlugInput } from "./slug-input";

export function FieldEditor({
  field,
  onGenerateSlug,
  schema,
  value,
  onChange,
}: {
  field: Field;
  onGenerateSlug?: () => void;
  schema: Schema | null;
  value: unknown;
  onChange: (value: unknown) => void;
}) {
  if (field.hidden) return null;

  const label = field.title ?? displayType(field.name);
  const stringValue = typeof value === "string" ? value : value == null ? "" : String(value);

  if (field.type === "boolean") {
    return (
      <div>
        <BooleanFieldEditor
          field={field}
          label={label}
          checked={Boolean(value)}
          onChange={onChange}
        />
        {field.description ? (
          <p className="mt-1.5 text-xs text-zinc-500 dark:text-zinc-400">{field.description}</p>
        ) : null}
      </div>
    );
  }

  const hasOptions = (field.type === "string" || field.type === "number") && field.options != null;

  if (hasOptions) {
    const opts = (field as StringField | NumberField).options!;
    const selectValue = value == null ? "" : String(value);
    return (
      <FieldShell description={field.description} label={label} required={field.required}>
        <select
          className={selectClass}
          disabled={field.readOnly}
          onChange={(e) => {
            const opt = opts.find((o) => String(o.value) === e.target.value);
            onChange(opt?.value ?? e.target.value);
          }}
          required={field.required}
          value={selectValue}
        >
          <option value="">Select…</option>
          {opts.map((opt) => (
            <option key={String(opt.value)} value={String(opt.value)}>
              {opt.label}
            </option>
          ))}
        </select>
      </FieldShell>
    );
  }

  return (
    <FieldShell description={field.description} label={label} required={field.required}>
      {field.type === "text" || field.type === "richtext" ? (
        <TextFieldEditor field={field} onChange={onChange} value={stringValue} />
      ) : field.type === "number" ? (
        <NumberFieldEditor field={field} onChange={onChange} value={value} />
      ) : field.type === "date" ? (
        <DateFieldEditor field={field} onChange={onChange} value={stringValue} />
      ) : field.type === "slug" ? (
        <SlugInput
          onChange={(nextValue) => onChange(nextValue)}
          onGenerate={onGenerateSlug}
          value={stringValue}
        />
      ) : field.type === "reference" ? (
        <ReferenceFieldEditor
          field={field}
          id={`field-${field.name}`}
          onChange={onChange}
          schema={schema}
          value={stringValue}
        />
      ) : field.type === "image" ? (
        <ImageFieldEditor field={field} onChange={onChange} value={stringValue} />
      ) : field.type === "array" ? (
        <ArrayFieldEditor field={field} onChange={onChange} schema={schema} value={value} />
      ) : (
        <TextFieldEditor field={field} onChange={onChange} value={stringValue} />
      )}
    </FieldShell>
  );
}
