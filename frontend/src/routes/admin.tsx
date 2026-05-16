import type { FormEvent, MouseEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { toast } from "sonner";

import { FieldEditor } from "../components/field-editor";
import { FieldShell } from "../components/field-shell";
import { CollapsedRail, SidebarHeader } from "../components/sidebar";
import { StatusPill } from "../components/status-pill";
import { inputClass } from "../components/ui";
import { api } from "../lib/api";
import { draftFromDocument, newDraft, titleFor } from "../lib/document";
import { classNames, displayType, formatDate, slugify } from "../lib/format";
import { documentPath } from "../lib/routes";
import type { Document, Draft, Field, Schema, SlugField, Status } from "../types";

export function AdminPage() {
  const navigate = useNavigate();
  const { documentId, documentType } = useParams();
  const [schema, setSchema] = useState<Schema | null>(null);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [selectedType, setSelectedType] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [jsonMode, setJsonMode] = useState(false);
  const [jsonText, setJsonText] = useState("{}");
  const [typesCollapsed, setTypesCollapsed] = useState(false);
  const [docsCollapsed, setDocsCollapsed] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadSchema() {
      setLoading(true);
      try {
        const nextSchema = await api.get<Schema>("/api/schema");
        if (cancelled) return;

        const urlType = nextSchema.types.find((type) => type.name === documentType);
        const nextType = urlType?.name ?? nextSchema.types[0]?.name ?? "";

        setSchema(nextSchema);
        setSelectedType(nextType);
        if (nextType && nextType !== documentType) {
          navigate(documentPath(nextType), { replace: true });
        }
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Could not load schema");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    void loadSchema();
    return () => {
      cancelled = true;
    };
  }, [documentType, navigate]);

  useEffect(() => {
    if (!schema) return;

    const urlType = schema.types.find((type) => type.name === documentType);
    const nextType = urlType?.name ?? schema.types[0]?.name ?? "";
    setSelectedType(nextType);

    if (nextType && nextType !== documentType) {
      navigate(documentPath(nextType), { replace: true });
    }
  }, [documentType, navigate, schema]);

  const selectedDef = useMemo(
    () => schema?.types.find((type) => type.name === selectedType),
    [schema, selectedType],
  );

  useEffect(() => {
    if (!selectedType || !selectedDef) return;
    let cancelled = false;
    const typeDef = selectedDef;

    async function loadDocuments() {
      setLoading(true);
      try {
        const params = new URLSearchParams({ type: selectedType, limit: "100" });
        const nextDocs = await api.get<Document[]>(`/api/documents?${params}`);
        if (cancelled) return;

        const requestedId = documentType === selectedType ? (documentId ?? null) : null;
        let nextDocument = requestedId
          ? (nextDocs.find((doc) => doc.id === requestedId) ?? null)
          : null;
        let visibleDocs = nextDocs;

        if (requestedId && !nextDocument) {
          const fetched = await api.get<Document>(`/api/documents/${requestedId}`);
          if (fetched.type !== selectedType) {
            throw new Error(`Document does not belong to ${displayType(selectedType)}`);
          }
          nextDocument = fetched;
          visibleDocs = [fetched, ...nextDocs];
        }

        setDocuments(visibleDocs);
        setSelectedId(nextDocument?.id ?? null);
        setDraft(nextDocument ? draftFromDocument(nextDocument) : newDraft(typeDef));
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Could not load documents");
        setSelectedId(null);
        setDraft(newDraft(typeDef));
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    void loadDocuments();
    return () => {
      cancelled = true;
    };
  }, [documentId, documentType, selectedType, selectedDef]);

  useEffect(() => {
    setJsonText(JSON.stringify(draft?.data ?? {}, null, 2));
  }, [draft?.data]);

  const selectedDocument = documents.find((doc) => doc.id === selectedId) ?? null;
  const gridClass = typesCollapsed
    ? docsCollapsed
      ? "grid-cols-[3.5rem_3.5rem_minmax(0,1fr)]"
      : "grid-cols-[3.5rem_20rem_minmax(0,1fr)]"
    : docsCollapsed
      ? "grid-cols-[14rem_3.5rem_minmax(0,1fr)]"
      : "grid-cols-[14rem_20rem_minmax(0,1fr)]";

  function selectDocument(doc: Document) {
    setSelectedId(doc.id);
    setDraft(draftFromDocument(doc));
  }

  function navigateTo(event: MouseEvent, type: string, id?: string | null) {
    event.preventDefault();
    setSelectedType(type);
    setSelectedId(id ?? null);
    navigate(documentPath(type, id ?? null));
    setJsonMode(false);
  }

  function createDocument() {
    if (!selectedDef) return;
    setSelectedId(null);
    setDraft(newDraft(selectedDef));
    navigate(documentPath(selectedDef.name));
  }

  function updateField(field: Field, value: unknown) {
    setDraft((current) => {
      if (!current) return current;

      const nextData = { ...current.data, [field.name]: value };
      const slugFieldDef = selectedDef?.fields.find((item) => item.type === "slug");
      if (slugFieldDef && field.name === slugFieldDef.source && !current.data[slugFieldDef.name]) {
        nextData[slugFieldDef.name] = slugify(String(value));
      }

      return { ...current, data: nextData };
    });
  }

  function generateSlug(field?: SlugField) {
    if (!draft || !field) return;

    const sourceName = field.source;
    const sourceValue =
      sourceName && typeof draft.data[sourceName] === "string"
        ? draft.data[sourceName]
        : typeof draft.data.title === "string"
          ? draft.data.title
          : typeof draft.data.name === "string"
            ? draft.data.name
            : null;
    const nextSlug = slugify(String(sourceValue ?? ""));

    if (!nextSlug) {
      toast.error("Add a title or name before generating a slug");
      return;
    }

    setDraft((current) => {
      if (!current) return current;
      return { ...current, data: { ...current.data, [field.name]: nextSlug } };
    });
  }

  async function save(event: FormEvent) {
    event.preventDefault();
    await persist();
  }

  async function persist(statusOverride?: Status) {
    if (!draft || !selectedDef) return;

    setSaving(true);

    try {
      const data = jsonMode ? JSON.parse(jsonText) : { ...draft.data };

      const body = {
        type: selectedDef.name,
        status: statusOverride ?? draft.status,
        data,
      };

      const saved = selectedId
        ? await api.send<Document>(`/api/documents/${selectedId}`, "PUT", body)
        : await api.send<Document>("/api/documents", "POST", body);

      setSelectedId(saved.id);
      setDraft(draftFromDocument(saved));
      navigate(documentPath(saved.type, saved.id), { replace: true });
      setDocuments((current) => {
        const exists = current.some((doc) => doc.id === saved.id);
        return exists
          ? current.map((doc) => (doc.id === saved.id ? saved : doc))
          : [saved, ...current];
      });
      toast.success(statusOverride === "published" ? "Published" : "Saved");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not save document");
    } finally {
      setSaving(false);
    }
  }

  async function removeDocument() {
    if (!selectedId) return;
    setSaving(true);

    try {
      await api.send<{ deleted: boolean }>(`/api/documents/${selectedId}`, "DELETE");
      const remaining = documents.filter((doc) => doc.id !== selectedId);
      const next = remaining[0];
      setDocuments(remaining);
      setSelectedId(next?.id ?? null);
      setDraft(next ? draftFromDocument(next) : selectedDef ? newDraft(selectedDef) : null);
      navigate(documentPath(selectedType, next?.id ?? null), { replace: true });
      toast.success("Deleted");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not delete document");
    } finally {
      setSaving(false);
    }
  }

  if (loading && !schema) {
    return (
      <main className="grid min-h-screen place-items-center text-sm text-zinc-500">Loading</main>
    );
  }

  return (
    <main className={classNames("grid h-full min-h-0", gridClass)}>
      <aside className="flex min-h-0 flex-col border-r border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950">
        <SidebarHeader
          collapsed={typesCollapsed}
          label="Types"
          onToggle={() => setTypesCollapsed((value) => !value)}
        />

        <div className="min-h-0 flex-1 overflow-auto">
          {typesCollapsed ? (
            <CollapsedRail label="Types" />
          ) : (
            <nav className="space-y-1 p-3" aria-label="Content types">
              {schema?.types.map((type) => (
                <a
                  className={classNames(
                    "block w-full rounded-md px-3 py-2 text-left text-sm",
                    type.name === selectedType
                      ? "bg-zinc-950 text-white dark:bg-zinc-50 dark:text-zinc-950"
                      : "text-zinc-600 hover:bg-zinc-100 hover:text-zinc-950 dark:text-zinc-400 dark:hover:bg-zinc-900 dark:hover:text-zinc-50",
                  )}
                  href={documentPath(type.name)}
                  key={type.name}
                  onClick={(event) => navigateTo(event, type.name)}
                >
                  <span className="truncate">{displayType(type.name)}</span>
                </a>
              ))}
            </nav>
          )}
        </div>
      </aside>

      <aside className="min-h-0 border-r border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950">
        <SidebarHeader
          collapsed={docsCollapsed}
          label="Entries"
          onToggle={() => setDocsCollapsed((value) => !value)}
        />

        {docsCollapsed ? (
          <CollapsedRail label="Entries" />
        ) : (
          <div className="flex h-[calc(100%-3.5rem)] min-h-0 flex-col">
            <div className="flex items-center justify-between border-b border-zinc-200 px-4 py-3 dark:border-zinc-800">
              <div className="min-w-0">
                <p className="truncate text-sm font-medium">
                  {selectedDef ? displayType(selectedDef.name) : "No type"}
                </p>
                <p className="text-xs text-zinc-500 dark:text-zinc-400">
                  {documents.length} entries
                </p>
              </div>
              <button
                className="rounded-md border border-zinc-200 px-2.5 py-1.5 text-sm hover:bg-zinc-100 dark:border-zinc-800 dark:hover:bg-zinc-900"
                onClick={createDocument}
                type="button"
              >
                New
              </button>
            </div>

            <div className="min-h-0 flex-1 overflow-auto p-2">
              {documents.length === 0 ? (
                <p className="px-2 py-6 text-sm text-zinc-500 dark:text-zinc-400">
                  No entries yet.
                </p>
              ) : (
                documents.map((doc) => (
                  <a
                    className={classNames(
                      "mb-1 block w-full rounded-md px-3 py-2 text-left",
                      doc.id === selectedId
                        ? "bg-zinc-100 dark:bg-zinc-900"
                        : "hover:bg-zinc-50 dark:hover:bg-zinc-900/70",
                    )}
                    href={documentPath(doc.type, doc.id)}
                    key={doc.id}
                    onClick={(event) => {
                      navigateTo(event, doc.type, doc.id);
                      selectDocument(doc);
                    }}
                  >
                    <span className="block truncate text-sm font-medium">
                      {titleFor(doc, selectedDef)}
                    </span>
                    <span className="mt-1 flex items-center justify-between gap-2 text-xs text-zinc-500 dark:text-zinc-400">
                      <span className="truncate">{formatDate(doc.created_at)}</span>
                      <StatusPill status={doc.status} />
                    </span>
                  </a>
                ))
              )}
            </div>
          </div>
        )}
      </aside>

      <form className="flex min-h-0 flex-col" onSubmit={(event) => void save(event)}>
        <header className="flex h-14 shrink-0 items-center justify-between gap-4 border-b border-zinc-200 bg-white/95 px-6 dark:border-zinc-800 dark:bg-zinc-950/95">
          <div className="min-w-0">
            <h1 className="truncate text-base font-semibold">
              {selectedDocument ? titleFor(selectedDocument, selectedDef) : "New entry"}
            </h1>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">
              {selectedDef ? displayType(selectedDef.name) : "No schema"} · Updated{" "}
              {formatDate(selectedDocument?.updated_at ?? null)}
            </p>
          </div>

          <div className="flex items-center gap-2">
            <button
              className="rounded-md border border-zinc-200 px-3 py-1.5 text-sm hover:bg-zinc-100 dark:border-zinc-800 dark:hover:bg-zinc-900"
              onClick={() => setJsonMode((value) => !value)}
              type="button"
            >
              {jsonMode ? "Fields" : "JSON"}
            </button>
            {selectedId ? (
              <button
                className="rounded-md px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                disabled={saving}
                onClick={() => void removeDocument()}
                type="button"
              >
                Delete
              </button>
            ) : null}
          </div>
        </header>

        <div className="min-h-0 flex-1 overflow-auto px-6 py-6">
          <div className="mx-auto max-w-3xl">
            {draft && selectedDef ? (
              <div className="space-y-5">
                {jsonMode ? (
                  <FieldShell label="Raw data">
                    <textarea
                      className={`${inputClass} min-h-96 font-mono text-sm`}
                      onChange={(event) => setJsonText(event.target.value)}
                      spellCheck={false}
                      value={jsonText}
                    />
                  </FieldShell>
                ) : (
                  selectedDef.fields.map((field) => {
                    return (
                      <FieldEditor
                        field={field}
                        key={field.name}
                        onGenerateSlug={() =>
                          generateSlug(field.type === "slug" ? field : undefined)
                        }
                        onChange={(nextValue) => updateField(field, nextValue)}
                        schema={schema}
                        value={draft.data[field.name]}
                      />
                    );
                  })
                )}
              </div>
            ) : (
              <p className="text-sm text-zinc-500 dark:text-zinc-400">No content schema loaded.</p>
            )}
          </div>
        </div>

        <footer className="h-14 shrink-0 border-t border-zinc-200 bg-white/95 px-6 py-2 dark:border-zinc-800 dark:bg-zinc-950/95">
          <div className="mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-3">
            {draft ? (
              <label className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300">
                <span>Status</span>
                <select
                  className="rounded-md border border-zinc-200 bg-white px-2.5 py-1.5 text-sm text-zinc-950 outline-none focus:border-zinc-400 focus:ring-4 focus:ring-zinc-100 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-50 dark:focus:ring-zinc-800"
                  onChange={(event) => setDraft({ ...draft, status: event.target.value as Status })}
                  value={draft.status}
                >
                  <option value="draft">Draft</option>
                  <option value="published">Published</option>
                  <option value="archived">Archived</option>
                </select>
              </label>
            ) : (
              <span />
            )}

            <div className="flex items-center gap-2">
              {draft?.status !== "published" ? (
                <button
                  className="rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-950 hover:bg-zinc-50 disabled:opacity-50 dark:border-zinc-700 dark:bg-zinc-950 dark:text-zinc-50 dark:hover:bg-zinc-900"
                  disabled={saving || !draft}
                  onClick={() => void persist("published")}
                  type="button"
                >
                  {saving ? "Saving" : "Save and publish"}
                </button>
              ) : null}
              <button
                className="rounded-md bg-zinc-950 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50 dark:bg-zinc-50 dark:text-zinc-950"
                disabled={saving || !draft}
                type="submit"
              >
                {saving ? "Saving" : "Save"}
              </button>
            </div>
          </div>
        </footer>
      </form>
    </main>
  );
}
