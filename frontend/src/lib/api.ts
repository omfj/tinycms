async function readError(res: Response) {
  try {
    const body = (await res.json()) as { error?: string };
    return body.error ?? `${res.status} ${res.statusText}`;
  } catch {
    return `${res.status} ${res.statusText}`;
  }
}

export const api = {
  async get<T>(path: string): Promise<T> {
    const res = await fetch(path);
    if (!res.ok) throw new Error(await readError(res));
    return res.json() as Promise<T>;
  },

  async send<T>(path: string, method: string, body?: unknown): Promise<T> {
    const res = await fetch(path, {
      method,
      headers: body === undefined ? undefined : { "Content-Type": "application/json" },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!res.ok) throw new Error(await readError(res));
    return res.json() as Promise<T>;
  },
};

export async function uploadFile(file: File) {
  const form = new FormData();
  form.append("file", file);

  const res = await fetch("/api/uploads", { method: "POST", body: form });
  if (!res.ok) throw new Error(await readError(res));
  return res.json() as Promise<{ url: string; id: string; key: string }>;
}

export async function listUploads() {
  const res = await fetch("/api/uploads");
  if (!res.ok) throw new Error(await readError(res));
  return res.json() as Promise<import("../types").Media[]>;
}

export async function updateUpload(id: string, label: string | null) {
  const res = await fetch(`/api/uploads/${id}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ label }),
  });
  if (!res.ok) throw new Error(await readError(res));
  return res.json() as Promise<import("../types").Media>;
}

export async function deleteUpload(id: string) {
  const res = await fetch(`/api/uploads/${id}`, { method: "DELETE" });
  if (!res.ok) throw new Error(await readError(res));
}
