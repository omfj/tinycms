import { useEffect, useState } from "react";
import { toast } from "sonner";

import { SettingStat } from "../components/setting-stat";
import { inputClass } from "../components/ui";
import { useAuth } from "../lib/auth";
import { api } from "../lib/api";
import { displayType } from "../lib/format";
import type { ApiToken, Schema, User, WorkspaceSettings } from "../types";

export function SettingsPage() {
  const { user } = useAuth();
  const isAdmin = user?.role === "admin";

  const [schema, setSchema] = useState<Schema | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceSettings | null>(null);
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  // Workspace form state
  const [workspaceName, setWorkspaceName] = useState("");
  const [requireApproval, setRequireApproval] = useState(true);
  const [defaultRole, setDefaultRole] = useState("editor");

  // API tokens state
  const [tokens, setTokens] = useState<ApiToken[]>([]);
  const [newTokenName, setNewTokenName] = useState("");
  const [newTokenHasExpiry, setNewTokenHasExpiry] = useState(false);
  const [newTokenExpiry, setNewTokenExpiry] = useState("");
  const [creatingToken, setCreatingToken] = useState(false);
  const [revealedToken, setRevealedToken] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      try {
        const [nextSchema, nextWorkspace] = await Promise.all([
          api.get<Schema>("/api/schema"),
          api.get<WorkspaceSettings>("/api/workspace"),
        ]);
        if (cancelled) return;
        setSchema(nextSchema);
        setWorkspace(nextWorkspace);
        setWorkspaceName(nextWorkspace.name);
        setRequireApproval(nextWorkspace.require_approval);
        setDefaultRole(nextWorkspace.default_role);

        if (isAdmin) {
          const nextUsers = await api.get<User[]>("/api/users");
          if (!cancelled) setUsers(nextUsers);
        }

        const nextTokens = await api.get<ApiToken[]>("/api/tokens");
        if (!cancelled) setTokens(nextTokens);
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Could not load settings");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [isAdmin]);

  async function saveWorkspace() {
    setSaving(true);
    try {
      const updated = await api.send<WorkspaceSettings>("/api/workspace", "PATCH", {
        name: workspaceName,
        require_approval: requireApproval,
        default_role: defaultRole,
      });
      setWorkspace(updated);
      toast.success("Workspace settings saved");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not save settings");
    } finally {
      setSaving(false);
    }
  }

  async function updateUser(id: string, patch: { status?: string; role?: string }) {
    try {
      const updated = await api.send<User>(`/api/users/${id}`, "PATCH", patch);
      setUsers((prev) => prev.map((u) => (u.id === id ? updated : u)));
      toast.success("User updated");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not update user");
    }
  }

  async function createToken() {
    if (!newTokenName.trim()) {
      toast.error("Token name is required");
      return;
    }
    setCreatingToken(true);
    try {
      const res = await api.send<ApiToken & { raw_token: string }>("/api/tokens", "POST", {
        name: newTokenName.trim(),
        expires_at:
          newTokenHasExpiry && newTokenExpiry ? new Date(newTokenExpiry).toISOString() : null,
      });
      const { raw_token, ...token } = res;
      setTokens((prev) => [token, ...prev]);
      setRevealedToken(raw_token);
      setNewTokenName("");
      setNewTokenHasExpiry(false);
      setNewTokenExpiry("");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not create token");
    } finally {
      setCreatingToken(false);
    }
  }

  async function deleteToken(id: string) {
    try {
      const res = await fetch(`/api/tokens/${id}`, { method: "DELETE" });
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
      setTokens((prev) => prev.filter((t) => t.id !== id));
      toast.success("Token deleted");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Could not delete token");
    }
  }

  const pendingUsers = users.filter((u) => u.status === "pending");
  const activeUsers = users.filter((u) => u.status === "active");

  return (
    <main className="h-full overflow-auto">
      <section className="mx-auto max-w-3xl px-6 py-8">
        <div className="mb-8">
          <h1 className="text-xl font-semibold">{workspace?.name ?? "Workspace settings"}</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
            Manage your workspace configuration and users
          </p>
        </div>

        {loading ? (
          <p className="text-sm text-zinc-500 dark:text-zinc-400">Loading…</p>
        ) : (
          <div className="space-y-10">
            {/* Schema overview */}
            <section>
              <h2 className="mb-3 text-sm font-semibold">Schema</h2>
              <div className="mb-3 grid grid-cols-2 gap-3">
                <SettingStat label="Types" value={schema?.types.length ?? 0} />
                <SettingStat
                  label="Configured fields"
                  value={schema?.types.reduce((sum, type) => sum + type.fields.length, 0) ?? 0}
                />
              </div>
              <div className="divide-y divide-zinc-200 rounded-md border border-zinc-200 dark:divide-zinc-800 dark:border-zinc-800">
                {schema?.types.map((type) => (
                  <div
                    className="flex items-center justify-between px-3 py-2 text-sm"
                    key={type.name}
                  >
                    <span>{displayType(type.name)}</span>
                    <span className="text-zinc-500 dark:text-zinc-400">
                      {type.fields.length} fields
                    </span>
                  </div>
                ))}
              </div>
            </section>

            {/* Workspace settings — admin only */}
            {isAdmin ? (
              <section>
                <h2 className="mb-3 text-sm font-semibold">Workspace</h2>
                <div className="space-y-4 rounded-md border border-zinc-200 p-4 dark:border-zinc-800">
                  <div>
                    <label className="mb-1 block text-sm font-medium" htmlFor="ws-name">
                      Name
                    </label>
                    <input
                      className={inputClass}
                      id="ws-name"
                      onChange={(e) => setWorkspaceName(e.target.value)}
                      type="text"
                      value={workspaceName}
                    />
                  </div>

                  <div>
                    <label className="mb-1 block text-sm font-medium" htmlFor="ws-role">
                      Default role for new users
                    </label>
                    <select
                      className="rounded-md border border-zinc-200 bg-white px-2.5 py-2 text-sm outline-none focus:border-zinc-400 focus:ring-4 focus:ring-zinc-100 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-50 dark:focus:ring-zinc-800"
                      id="ws-role"
                      onChange={(e) => setDefaultRole(e.target.value)}
                      value={defaultRole}
                    >
                      <option value="viewer">Viewer</option>
                      <option value="editor">Editor</option>
                      <option value="admin">Admin</option>
                    </select>
                  </div>

                  <label className="flex cursor-pointer items-center gap-3">
                    <input
                      checked={requireApproval}
                      className="size-4 rounded accent-zinc-950 dark:accent-zinc-50"
                      onChange={(e) => setRequireApproval(e.target.checked)}
                      type="checkbox"
                    />
                    <span className="text-sm">Require admin approval for new sign-ups</span>
                  </label>

                  <div className="flex justify-end pt-1">
                    <button
                      className="rounded-md bg-zinc-950 px-4 py-1.5 text-sm font-medium text-white disabled:opacity-50 dark:bg-zinc-50 dark:text-zinc-950"
                      disabled={saving}
                      onClick={() => void saveWorkspace()}
                      type="button"
                    >
                      {saving ? "Saving…" : "Save"}
                    </button>
                  </div>
                </div>
              </section>
            ) : null}

            {/* Pending approvals */}
            {isAdmin && pendingUsers.length > 0 ? (
              <section>
                <h2 className="mb-3 text-sm font-semibold">
                  Pending approval{" "}
                  <span className="ml-1 rounded-full bg-amber-100 px-2 py-0.5 text-xs font-medium text-amber-800 dark:bg-amber-900/40 dark:text-amber-400">
                    {pendingUsers.length}
                  </span>
                </h2>
                <div className="divide-y divide-zinc-200 rounded-md border border-zinc-200 dark:divide-zinc-800 dark:border-zinc-800">
                  {pendingUsers.map((u) => (
                    <div className="flex items-center justify-between px-3 py-2.5" key={u.id}>
                      <div>
                        <p className="text-sm font-medium">{u.name ?? u.email}</p>
                        {u.name ? (
                          <p className="text-xs text-zinc-500 dark:text-zinc-400">{u.email}</p>
                        ) : null}
                      </div>
                      <div className="flex items-center gap-2">
                        <button
                          className="rounded-md border border-zinc-200 px-3 py-1.5 text-xs font-medium hover:bg-zinc-100 dark:border-zinc-800 dark:hover:bg-zinc-900"
                          onClick={() => void updateUser(u.id, { status: "active" })}
                          type="button"
                        >
                          Approve
                        </button>
                        <button
                          className="rounded-md px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                          onClick={() => void updateUser(u.id, { status: "suspended" })}
                          type="button"
                        >
                          Deny
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </section>
            ) : null}

            {/* API tokens */}
            <section>
              <h2 className="mb-3 text-sm font-semibold">API Tokens</h2>

              {revealedToken ? (
                <div className="mb-4 rounded-md border border-green-200 bg-green-50 p-4 dark:border-green-800 dark:bg-green-950/30">
                  <p className="mb-2 text-sm font-medium text-green-800 dark:text-green-300">
                    Token created — copy it now, it won&apos;t be shown again.
                  </p>
                  <div className="flex items-center gap-2">
                    <code className="flex-1 break-all rounded bg-white px-2 py-1.5 font-mono text-xs dark:bg-zinc-900">
                      {revealedToken}
                    </code>
                    <button
                      className="shrink-0 rounded-md border border-zinc-200 px-3 py-1.5 text-xs font-medium hover:bg-zinc-100 dark:border-zinc-800 dark:hover:bg-zinc-900"
                      onClick={() => {
                        void navigator.clipboard.writeText(revealedToken);
                        toast.success("Copied to clipboard");
                      }}
                      type="button"
                    >
                      Copy
                    </button>
                    <button
                      className="shrink-0 rounded-md px-2 py-1.5 text-xs text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100"
                      onClick={() => setRevealedToken(null)}
                      type="button"
                    >
                      Dismiss
                    </button>
                  </div>
                </div>
              ) : null}

              <div className="mb-3 space-y-2">
                <div className="flex gap-2">
                  <input
                    className={inputClass}
                    onChange={(e) => setNewTokenName(e.target.value)}
                    placeholder="Token name"
                    type="text"
                    value={newTokenName}
                  />
                  <button
                    className="shrink-0 rounded-md bg-zinc-950 px-4 py-1.5 text-sm font-medium text-white disabled:opacity-50 dark:bg-zinc-50 dark:text-zinc-950"
                    disabled={creatingToken}
                    onClick={() => void createToken()}
                    type="button"
                  >
                    {creatingToken ? "Creating…" : "Create"}
                  </button>
                </div>
                <label className="flex cursor-pointer items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
                  <input
                    checked={newTokenHasExpiry}
                    className="size-3.5 rounded accent-zinc-950 dark:accent-zinc-50"
                    onChange={(e) => {
                      setNewTokenHasExpiry(e.target.checked);
                      if (!e.target.checked) setNewTokenExpiry("");
                    }}
                    type="checkbox"
                  />
                  Set expiry date
                  {newTokenHasExpiry ? (
                    <input
                      className={inputClass + " ml-1"}
                      min={new Date().toISOString().split("T")[0]}
                      onChange={(e) => setNewTokenExpiry(e.target.value)}
                      type="date"
                      value={newTokenExpiry}
                    />
                  ) : null}
                </label>
              </div>

              {tokens.length === 0 ? (
                <p className="text-sm text-zinc-500 dark:text-zinc-400">No API tokens yet.</p>
              ) : (
                <div className="divide-y divide-zinc-200 rounded-md border border-zinc-200 dark:divide-zinc-800 dark:border-zinc-800">
                  {tokens.map((t) => (
                    <div className="flex items-center justify-between px-3 py-2.5" key={t.id}>
                      <div>
                        <p className="text-sm font-medium">{t.name}</p>
                        <p className="text-xs text-zinc-500 dark:text-zinc-400">
                          Created {new Date(t.created_at).toLocaleDateString()}
                          {t.expires_at
                            ? ` · Expires ${new Date(t.expires_at).toLocaleDateString()}`
                            : " · No expiry"}
                          {t.last_used_at
                            ? ` · Last used ${new Date(t.last_used_at).toLocaleDateString()}`
                            : " · Never used"}
                        </p>
                      </div>
                      <button
                        className="rounded-md px-2.5 py-1 text-xs text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                        onClick={() => void deleteToken(t.id)}
                        type="button"
                      >
                        Revoke
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </section>

            {/* Active users */}
            {isAdmin && activeUsers.length > 0 ? (
              <section>
                <h2 className="mb-3 text-sm font-semibold">Users</h2>
                <div className="divide-y divide-zinc-200 rounded-md border border-zinc-200 dark:divide-zinc-800 dark:border-zinc-800">
                  {activeUsers.map((u) => (
                    <div className="flex items-center justify-between px-3 py-2.5" key={u.id}>
                      <div>
                        <p className="text-sm font-medium">{u.name ?? u.email}</p>
                        {u.name ? (
                          <p className="text-xs text-zinc-500 dark:text-zinc-400">{u.email}</p>
                        ) : null}
                      </div>
                      <div className="flex items-center gap-2">
                        <select
                          className="rounded-md border border-zinc-200 bg-white px-2 py-1 text-xs outline-none focus:border-zinc-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-50"
                          onChange={(e) => void updateUser(u.id, { role: e.target.value })}
                          value={u.role}
                        >
                          <option value="viewer">Viewer</option>
                          <option value="editor">Editor</option>
                          <option value="admin">Admin</option>
                        </select>
                        {u.id !== user?.id ? (
                          <button
                            className="rounded-md px-2.5 py-1 text-xs text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                            onClick={() => void updateUser(u.id, { status: "suspended" })}
                            type="button"
                          >
                            Remove
                          </button>
                        ) : null}
                      </div>
                    </div>
                  ))}
                </div>
              </section>
            ) : null}
          </div>
        )}
      </section>
    </main>
  );
}
