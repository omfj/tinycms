import { LogOut, Moon, Sun } from "lucide-react";
import { Link, useLocation } from "react-router-dom";

import { useAuth } from "../lib/auth";
import { classNames } from "../lib/format";

export function TopNav({
  storageConfigured,
  darkTheme,
  onTheme,
}: {
  storageConfigured: boolean;
  darkTheme: boolean;
  onTheme: () => void;
}) {
  const { pathname } = useLocation();
  const { user, logout } = useAuth();

  function isActive(path: string) {
    if (path === "/") return pathname !== "/media" && pathname !== "/settings";
    return pathname.startsWith(path);
  }

  function tabClass(path: string) {
    return classNames(
      "rounded-md px-3 py-1.5 text-sm font-medium transition-colors",
      isActive(path)
        ? "bg-zinc-950 text-white dark:bg-zinc-50 dark:text-zinc-950"
        : "text-zinc-500 hover:bg-zinc-100 hover:text-zinc-950 dark:text-zinc-400 dark:hover:bg-zinc-900 dark:hover:text-zinc-50",
    );
  }

  const iconClass =
    "grid size-8 place-items-center rounded-md text-zinc-500 hover:bg-zinc-100 hover:text-zinc-950 dark:text-zinc-400 dark:hover:bg-zinc-900 dark:hover:text-zinc-50";

  return (
    <header className="flex h-12 shrink-0 items-center justify-between border-b border-zinc-200 bg-white px-3 dark:border-zinc-800 dark:bg-zinc-950">
      <div className="flex items-center gap-1">
        <Link className={tabClass("/")} to="/">
          Documents
        </Link>
        {storageConfigured && (
          <Link className={tabClass("/media")} to="/media">
            Media
          </Link>
        )}
        <Link className={tabClass("/settings")} to="/settings">
          Settings
        </Link>
      </div>

      <div className="flex items-center gap-2">
        {user && (
          <span className="text-sm text-zinc-600 dark:text-zinc-300">
            {user.name ?? user.email}
          </span>
        )}
        <button
          className={iconClass}
          onClick={onTheme}
          title={darkTheme ? "Use light theme" : "Use dark theme"}
          type="button"
        >
          {darkTheme ? <Sun aria-hidden="true" size={16} /> : <Moon aria-hidden="true" size={16} />}
          <span className="sr-only">{darkTheme ? "Use light theme" : "Use dark theme"}</span>
        </button>
        <button className={iconClass} onClick={() => void logout()} title="Sign out" type="button">
          <LogOut aria-hidden="true" size={16} />
          <span className="sr-only">Sign out</span>
        </button>
      </div>
    </header>
  );
}
