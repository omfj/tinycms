export function documentPath(type: string | null, id?: string | null) {
  if (!type) return "/";
  return `/${encodeURIComponent(type)}${id ? `/${encodeURIComponent(id)}` : ""}`;
}
