export function SidebarHeader({
  collapsed,
  label,
  onToggle,
}: {
  collapsed: boolean;
  label: string;
  onToggle: () => void;
}) {
  return (
    <div className="flex h-14 items-center justify-between border-b border-zinc-200 px-3 dark:border-zinc-800">
      {collapsed ? null : <span className="text-sm font-semibold">{label}</span>}
      <button
        className="grid size-8 place-items-center rounded-md text-zinc-500 hover:bg-zinc-100 hover:text-zinc-950 dark:text-zinc-400 dark:hover:bg-zinc-900 dark:hover:text-zinc-50"
        onClick={onToggle}
        title={collapsed ? `Expand ${label}` : `Collapse ${label}`}
        type="button"
      >
        {collapsed ? ">" : "<"}
      </button>
    </div>
  );
}

export function CollapsedRail({ label }: { label: string }) {
  return (
    <div className="grid h-full place-items-center">
      <span className="-rotate-90 text-xs font-medium uppercase tracking-wide text-zinc-400">
        {label}
      </span>
    </div>
  );
}
