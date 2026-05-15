import type { FormEvent } from "react";
import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";

import { inputClass } from "../components/ui";
import { useAuth } from "../lib/auth";

type Providers = {
  credentials: boolean;
  github: boolean;
  google: boolean;
};

export function LoginPage({ darkTheme }: { darkTheme: boolean }) {
  const { refresh } = useAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const [providers, setProviders] = useState<Providers | null>(null);
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(searchParams.get("error"));

  useEffect(() => {
    fetch("/api/auth/providers")
      .then((r) => r.json() as Promise<Providers>)
      .then(setProviders)
      .catch(() => setProviders({ credentials: false, github: false, google: false }));
  }, []);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setBusy(true);

    try {
      if (mode === "login") {
        const res = await fetch("/api/auth/login", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ email, password }),
        });
        if (!res.ok) {
          const body = (await res.json()) as { error?: string };
          setError(body.error ?? "Invalid credentials");
          return;
        }
        await refresh();
        navigate("/", { replace: true });
      } else {
        const res = await fetch("/api/auth/register", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ email, password, name: name || undefined }),
        });
        if (res.status === 202) {
          navigate("/pending", { replace: true });
          return;
        }
        if (!res.ok) {
          const body = (await res.json()) as { error?: string };
          setError(body.error ?? "Registration failed");
          return;
        }
        await refresh();
        navigate("/", { replace: true });
      }
    } catch {
      setError("Something went wrong, please try again");
    } finally {
      setBusy(false);
    }
  }

  const errorMessages: Record<string, string> = {
    access_denied: "Access was denied",
    auth_failed: "Authentication failed",
    invalid_state: "Login session expired, please try again",
    no_email: "Could not retrieve your email from the provider",
    not_configured: "This sign-in method is not configured",
    suspended: "Your account has been suspended",
  };

  const displayError = error ? (errorMessages[error] ?? error) : null;
  const hasOAuth = providers?.github || providers?.google;

  return (
    <main
      className={`flex min-h-screen items-center justify-center bg-zinc-50 px-4 text-zinc-950 dark:bg-zinc-950 dark:text-zinc-50 ${darkTheme ? "dark" : ""}`}
    >
      <div className="w-full max-w-sm">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-semibold">TinyCMS</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
            {mode === "login" ? "Sign in to your workspace" : "Create an account"}
          </p>
        </div>

        {displayError ? (
          <div className="mb-4 rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-400">
            {displayError}
          </div>
        ) : null}

        {providers === null ? (
          <p className="text-center text-sm text-zinc-400">Loading…</p>
        ) : (
          <>
            {providers.credentials ? (
              <>
                <form className="space-y-3" onSubmit={(e) => void handleSubmit(e)}>
                  {mode === "register" ? (
                    <div>
                      <label className="mb-1 block text-sm font-medium" htmlFor="name">
                        Name
                      </label>
                      <input
                        autoComplete="name"
                        className={inputClass}
                        id="name"
                        onChange={(e) => setName(e.target.value)}
                        placeholder="Optional"
                        type="text"
                        value={name}
                      />
                    </div>
                  ) : null}

                  <div>
                    <label className="mb-1 block text-sm font-medium" htmlFor="email">
                      Email
                    </label>
                    <input
                      autoComplete="email"
                      className={inputClass}
                      id="email"
                      onChange={(e) => setEmail(e.target.value)}
                      placeholder="you@example.com"
                      required
                      type="email"
                      value={email}
                    />
                  </div>

                  <div>
                    <label className="mb-1 block text-sm font-medium" htmlFor="password">
                      Password
                    </label>
                    <input
                      autoComplete={mode === "login" ? "current-password" : "new-password"}
                      className={inputClass}
                      id="password"
                      minLength={mode === "register" ? 8 : undefined}
                      onChange={(e) => setPassword(e.target.value)}
                      required
                      type="password"
                      value={password}
                    />
                  </div>

                  <button
                    className="w-full rounded-md bg-zinc-950 px-4 py-2.5 text-sm font-medium text-white disabled:opacity-50 dark:bg-zinc-50 dark:text-zinc-950"
                    disabled={busy}
                    type="submit"
                  >
                    {busy ? "Please wait…" : mode === "login" ? "Sign in" : "Create account"}
                  </button>
                </form>

                <p className="mt-4 text-center text-sm text-zinc-500 dark:text-zinc-400">
                  {mode === "login" ? "No account?" : "Already have an account?"}{" "}
                  <button
                    className="font-medium text-zinc-950 underline-offset-2 hover:underline dark:text-zinc-50"
                    onClick={() => {
                      setMode(mode === "login" ? "register" : "login");
                      setError(null);
                    }}
                    type="button"
                  >
                    {mode === "login" ? "Register" : "Sign in"}
                  </button>
                </p>
              </>
            ) : null}

            {providers.credentials && hasOAuth ? (
              <div className="my-6 flex items-center gap-3">
                <div className="h-px flex-1 bg-zinc-200 dark:bg-zinc-800" />
                <span className="text-xs text-zinc-400">or continue with</span>
                <div className="h-px flex-1 bg-zinc-200 dark:bg-zinc-800" />
              </div>
            ) : null}

            {hasOAuth ? (
              <div className="space-y-2">
                {providers.github ? (
                  <a
                    className="flex w-full items-center justify-center gap-2 rounded-md border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-950 hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-50 dark:hover:bg-zinc-800"
                    href="/api/auth/github"
                  >
                    <GitHubIcon />
                    GitHub
                  </a>
                ) : null}
                {providers.google ? (
                  <a
                    className="flex w-full items-center justify-center gap-2 rounded-md border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-950 hover:bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-50 dark:hover:bg-zinc-800"
                    href="/api/auth/google"
                  >
                    <GoogleIcon />
                    Google
                  </a>
                ) : null}
              </div>
            ) : null}
          </>
        )}
      </div>
    </main>
  );
}

function GitHubIcon() {
  return (
    <svg aria-hidden="true" fill="currentColor" height="16" viewBox="0 0 24 24" width="16">
      <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0 0 22 12.017C22 6.484 17.522 2 12 2z" />
    </svg>
  );
}

function GoogleIcon() {
  return (
    <svg aria-hidden="true" height="16" viewBox="0 0 24 24" width="16">
      <path
        d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
        fill="#4285F4"
      />
      <path
        d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
        fill="#34A853"
      />
      <path
        d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
        fill="#FBBC05"
      />
      <path
        d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
        fill="#EA4335"
      />
    </svg>
  );
}
