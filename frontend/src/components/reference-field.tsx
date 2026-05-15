import { useEffect, useState } from "react";

import { api } from "../lib/api";
import { titleFor } from "../lib/document";
import { classNames, displayType } from "../lib/format";
import type { Document, ReferenceField, Schema, TypeDef } from "../types";
import { inputClass } from "./ui";

type ReferenceOption = {
  doc: Document;
  typeDef?: TypeDef;
};

export function ReferenceField({
  field,
  id,
  onChange,
  schema,
  value,
}: {
  field: ReferenceField;
  id: string;
  onChange: (value: unknown) => void;
  schema: Schema | null;
  value: string;
}) {
  const [options, setOptions] = useState<ReferenceOption[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const targetTypes = field.to ?? [];
  const selectedOption = options.find((option) => option.doc.id === value) ?? null;

  useEffect(() => {
    let cancelled = false;

    async function loadReferences() {
      if (targetTypes.length === 0) {
        setOptions([]);
        return;
      }

      setLoading(true);
      try {
        const groups = await Promise.all(
          targetTypes.map(async (type) => {
            const params = new URLSearchParams({ type, limit: "200" });
            const docs = await api.get<Document[]>(`/api/documents?${params}`);
            const typeDef = schema?.types.find((item) => item.name === type);
            return docs.map((doc) => ({ doc, typeDef }));
          }),
        );

        if (!cancelled) setOptions(groups.flat());
      } catch {
        if (!cancelled) setOptions([]);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    void loadReferences();
    return () => {
      cancelled = true;
    };
  }, [schema, targetTypes.join("|")]);

  useEffect(() => {
    if (!value) {
      setQuery("");
      return;
    }

    if (selectedOption) {
      setQuery(referenceLabel(selectedOption));
    }
  }, [selectedOption, value]);

  if (targetTypes.length === 0) {
    return (
      <input
        className={inputClass}
        id={id}
        onChange={(event) => onChange(event.target.value)}
        placeholder="Document id"
        required={field.required}
        value={value}
      />
    );
  }

  const normalizedQuery = query.trim().toLowerCase();
  const filteredOptions = normalizedQuery
    ? options.filter((option) => referenceLabel(option).toLowerCase().includes(normalizedQuery))
    : options;
  const visibleOptions = filteredOptions.slice(0, 20);

  return (
    <div className="relative">
      <input
        aria-autocomplete="list"
        aria-expanded={open}
        className={inputClass}
        id={id}
        onBlur={() => window.setTimeout(() => setOpen(false), 120)}
        onChange={(event) => {
          setQuery(event.target.value);
          setOpen(true);
          if (value) onChange("");
        }}
        onFocus={() => setOpen(true)}
        placeholder={loading ? "Loading references..." : "Search documents"}
        required={field.required}
        role="combobox"
        value={query}
      />

      {value ? (
        <button
          className="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-1 text-xs text-zinc-500 hover:bg-zinc-100 hover:text-zinc-950 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-50"
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => {
            onChange("");
            setQuery("");
            setOpen(true);
          }}
          type="button"
        >
          Clear
        </button>
      ) : null}

      {open ? (
        <div className="absolute z-20 mt-1 max-h-64 w-full overflow-auto rounded-md border border-zinc-200 bg-white p-1 shadow-lg dark:border-zinc-800 dark:bg-zinc-950">
          {value && !selectedOption ? (
            <ReferenceOptionButton
              label={value}
              onSelect={() => {
                setQuery(value);
                setOpen(false);
              }}
            />
          ) : null}

          {visibleOptions.length === 0 ? (
            <div className="px-3 py-2 text-sm text-zinc-500 dark:text-zinc-400">
              {loading ? "Loading..." : "No matches"}
            </div>
          ) : (
            visibleOptions.map((option) => (
              <ReferenceOptionButton
                key={option.doc.id}
                label={referenceLabel(option)}
                onSelect={() => {
                  onChange(option.doc.id);
                  setQuery(referenceLabel(option));
                  setOpen(false);
                }}
                selected={option.doc.id === value}
              />
            ))
          )}
        </div>
      ) : null}
    </div>
  );
}

function referenceLabel({ doc, typeDef }: ReferenceOption) {
  return `${displayType(doc.type)} / ${titleFor(doc, typeDef)}`;
}

function ReferenceOptionButton({
  label,
  onSelect,
  selected,
}: {
  label: string;
  onSelect: () => void;
  selected?: boolean;
}) {
  return (
    <button
      className={classNames(
        "block w-full rounded px-3 py-2 text-left text-sm",
        selected
          ? "bg-zinc-100 text-zinc-950 dark:bg-zinc-800 dark:text-zinc-50"
          : "text-zinc-700 hover:bg-zinc-100 dark:text-zinc-300 dark:hover:bg-zinc-900",
      )}
      onMouseDown={(event) => event.preventDefault()}
      onClick={onSelect}
      type="button"
    >
      {label}
    </button>
  );
}
