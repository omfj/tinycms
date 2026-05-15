import { classNames } from "../lib/format";

export function StatusPill({ status }: { status: string }) {
  return (
    <span
      className={classNames(
        "shrink-0 rounded-full px-2 py-0.5 text-[0.65rem] capitalize",
        status === "published"
          ? "bg-emerald-50 text-emerald-700"
          : status === "draft"
            ? "bg-zinc-100 text-zinc-600"
            : "bg-amber-50 text-amber-700",
      )}
    >
      {status}
    </span>
  );
}
