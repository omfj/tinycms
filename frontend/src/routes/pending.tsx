import { Clock } from "lucide-react";

export function PendingPage({ darkTheme }: { darkTheme: boolean }) {
  return (
    <main
      className={`flex min-h-screen items-center justify-center bg-zinc-50 px-4 text-zinc-950 dark:bg-zinc-950 dark:text-zinc-50 ${darkTheme ? "dark" : ""}`}
    >
      <div className="max-w-sm text-center">
        <div className="mx-auto mb-4 flex size-12 items-center justify-center rounded-full bg-zinc-100 dark:bg-zinc-900">
          <Clock aria-hidden="true" className="text-zinc-500" size={24} />
        </div>
        <h1 className="text-xl font-semibold">Awaiting approval</h1>
        <p className="mt-2 text-sm text-zinc-500 dark:text-zinc-400">
          Your account is pending admin approval. You'll be able to sign in once an admin reviews
          your request.
        </p>
        <a
          className="mt-6 inline-block text-sm text-zinc-500 underline-offset-2 hover:underline dark:text-zinc-400"
          href="/login"
        >
          Back to sign in
        </a>
      </div>
    </main>
  );
}
