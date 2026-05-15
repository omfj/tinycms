import type { ReactNode } from "react";

export function FieldShell({
  children,
  label,
  description,
  required,
}: {
  children: ReactNode;
  label: string;
  description?: string;
  required?: boolean;
}) {
  return (
    <label className="block">
      <span className="mb-1.5 flex items-center gap-2 text-sm font-medium text-zinc-700 dark:text-zinc-300">
        {label}
        {required ? (
          <span className="text-xs font-normal text-zinc-400 dark:text-zinc-500">Required</span>
        ) : null}
      </span>
      {children}
      {description ? (
        <p className="mt-1.5 text-xs text-zinc-500 dark:text-zinc-400">{description}</p>
      ) : null}
    </label>
  );
}
