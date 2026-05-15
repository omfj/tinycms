import type { ArrayField, Schema } from "../../types";
import { ImageFieldEditor } from "./image-field";
import { ReferenceField } from "../reference-field";
import { inputClass } from "../ui";

function ItemInput({
  field,
  index,
  onChange,
  schema,
  value,
}: {
  field: ArrayField;
  index: number;
  onChange: (value: unknown) => void;
  schema: Schema | null;
  value: unknown;
}) {
  const stringValue = typeof value === "string" ? value : value == null ? "" : String(value);

  if (field.of === "image") {
    return (
      <ImageFieldEditor
        field={{ type: "image", name: field.name }}
        onChange={onChange}
        value={stringValue}
      />
    );
  }

  if (field.of === "reference") {
    return (
      <ReferenceField
        field={{ type: "reference", name: field.name, to: [] }}
        id={`${field.name}-${index}`}
        onChange={onChange}
        schema={schema}
        value={stringValue}
      />
    );
  }

  return (
    <input
      className={inputClass}
      onChange={(e) => onChange(field.of === "number" ? Number(e.target.value) : e.target.value)}
      type={field.of === "number" ? "number" : "text"}
      value={field.of === "number" ? (typeof value === "number" ? value : 0) : stringValue}
    />
  );
}

export function ArrayFieldEditor({
  field,
  onChange,
  schema,
  value,
}: {
  field: ArrayField;
  onChange: (value: unknown) => void;
  schema: Schema | null;
  value: unknown;
}) {
  const items = Array.isArray(value) ? value : [];

  function update(index: number, next: unknown) {
    const arr = [...items];
    arr[index] = next;
    onChange(arr);
  }

  function remove(index: number) {
    onChange(items.filter((_, i) => i !== index));
  }

  function add() {
    onChange([...items, field.of === "number" ? 0 : ""]);
  }

  return (
    <div className="space-y-2">
      {items.map((item, index) => (
        <div className="flex items-start gap-2" key={index}>
          <div className="flex-1">
            <ItemInput
              field={field}
              index={index}
              onChange={(next) => update(index, next)}
              schema={schema}
              value={item}
            />
          </div>
          <button
            className="mt-px rounded-md px-2.5 py-2 text-sm text-zinc-400 hover:bg-zinc-100 hover:text-red-600 dark:hover:bg-zinc-900 dark:hover:text-red-400"
            onClick={() => remove(index)}
            title="Remove item"
            type="button"
          >
            ✕
          </button>
        </div>
      ))}
      <button
        className="rounded-md border border-dashed border-zinc-300 px-3 py-1.5 text-sm text-zinc-500 hover:border-zinc-400 hover:text-zinc-700 dark:border-zinc-700 dark:text-zinc-400 dark:hover:border-zinc-600 dark:hover:text-zinc-200"
        onClick={add}
        type="button"
      >
        + Add item
      </button>
    </div>
  );
}
