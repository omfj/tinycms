import type { Document, Draft, Field, TypeDef } from "../types";

export function newDraft(type: TypeDef): Draft {
  return {
    status: "draft",
    data: Object.fromEntries(type.fields.map((field) => [field.name, initialValue(field)])),
  };
}

export function draftFromDocument(doc: Document): Draft {
  return {
    status: doc.status as Draft["status"],
    data: { ...doc.data },
  };
}

export function titleFor(doc: Document, type?: TypeDef) {
  const titleField =
    type?.fields.find((field) => ["title", "name"].includes(field.name)) ??
    type?.fields.find((field) => field.type === "string");
  const value = titleField ? doc.data[titleField.name] : undefined;
  if (typeof value === "string" && value.trim()) return value;
  return "Untitled";
}

function initialValue(field: Field): unknown {
  if (field.type === "boolean") return false;
  return null;
}
