import { useEffect, useState } from "react";
import { BrowserRouter, Navigate, Route, Routes, useNavigate, useSearchParams } from "react-router-dom";
import { Toaster } from "sonner";

import { TopNav } from "./components/top-nav";
import { AuthProvider, useAuth } from "./lib/auth";
import { api } from "./lib/api";
import { classNames } from "./lib/format";
import { initialDarkTheme, themeStorageKey } from "./lib/theme";
import { LoginPage } from "./routes/login";
import { PendingPage } from "./routes/pending";
import { AdminPage } from "./routes/admin";
import { MediaPage } from "./routes/media";
import { SettingsPage } from "./routes/settings";
import type { Schema } from "./types";

export default function App() {
  const [darkTheme, setDarkTheme] = useState(initialDarkTheme);

  useEffect(() => {
    window.localStorage.setItem(themeStorageKey, darkTheme ? "dark" : "light");
  }, [darkTheme]);

  const toggleTheme = () => setDarkTheme((v) => !v);

  return (
    <BrowserRouter>
      <AuthProvider>
        <AppRoutes darkTheme={darkTheme} onTheme={toggleTheme} />
      </AuthProvider>
      <Toaster richColors theme={darkTheme ? "dark" : "light"} />
    </BrowserRouter>
  );
}

function AppRoutes({ darkTheme, onTheme }: { darkTheme: boolean; onTheme: () => void }) {
  const { user, loading } = useAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [storageConfigured, setStorageConfigured] = useState(false);

  useEffect(() => {
    if (!user) return;
    api
      .get<Schema>("/api/schema")
      .then((s) => setStorageConfigured(s.storage_configured))
      .catch(() => {});
  }, [user]);

  useEffect(() => {
    if (loading) return;
    const status = searchParams.get("status");
    const error = searchParams.get("error");
    if (!user) {
      if (status === "pending") {
        navigate("/pending", { replace: true });
      } else if (error) {
        navigate(`/login?error=${error}`, { replace: true });
      }
    }
  }, [loading, user, searchParams, navigate]);

  if (loading) {
    return (
      <main className="grid min-h-screen place-items-center text-sm text-zinc-500">Loading</main>
    );
  }

  if (!user) {
    return (
      <Routes>
        <Route element={<LoginPage darkTheme={darkTheme} />} path="/login" />
        <Route element={<PendingPage darkTheme={darkTheme} />} path="/pending" />
        <Route element={<Navigate replace to="/login" />} path="*" />
      </Routes>
    );
  }

  return (
    <div
      className={classNames(
        "flex h-screen flex-col overflow-hidden bg-zinc-50 text-zinc-950 dark:bg-zinc-950 dark:text-zinc-50",
        darkTheme && "dark",
      )}
    >
      <TopNav darkTheme={darkTheme} onTheme={onTheme} storageConfigured={storageConfigured} />
      <div className="min-h-0 flex-1">
        <Routes>
          <Route element={<SettingsPage />} path="/settings" />
          <Route element={<MediaPage />} path="/media" />
          <Route element={<AdminPage />} path="/" />
          <Route element={<AdminPage />} path="/:documentType" />
          <Route element={<AdminPage />} path="/:documentType/:documentId" />
          <Route element={<Navigate replace to="/" />} path="*" />
        </Routes>
      </div>
    </div>
  );
}
