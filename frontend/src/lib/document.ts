import type { Document, Draft, Field, TypeDef } from "../types";

export function newDraft(type: TypeDef): Draft {
  return {
    slug: "",
    status: "draft",
    data: Object.fromEntries(type.fields.map((field) => [field.name, initialValue(field)])),
  };
}

export function draftFromDocument(doc: Document, type?: TypeDef): Draft {
  const data = { ...doc.data };
  const slugField = type?.fields.find((field) => field.type === "slug");
  if (slugField && doc.slug) data[slugField.name] = doc.slug;

  return {
    slug: doc.slug ?? "",
    status: doc.status as Draft["status"],
    data,
  };
}

export function titleFor(doc: Document, type?: TypeDef) {
  const titleField =
    type?.fields.find((field) => ["title", "name"].includes(field.name)) ??
    type?.fields.find((field) => field.type === "string");
  const value = titleField ? doc.data[titleField.name] : undefined;
  if (typeof value === "string" && value.trim()) return value;
  return doc.slug || "Untitled";
}

function initialValue(field: Field): unknown {
  if (field.type === "boolean") return false;
  if (field.type === "number") return "";
  return "";
}
