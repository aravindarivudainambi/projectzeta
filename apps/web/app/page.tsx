import Link from "next/link";
import { ArrowRight, Bot, Eye, Workflow } from "lucide-react";

const highlights = [
  {
    title: "Prompt-to-config builder",
    description: "Generate a typed agent configuration from a natural-language workflow in seconds.",
    icon: Bot,
  },
  {
    title: "Visual workflow preview",
    description: "Switch between raw JSON and an execution-oriented canvas without losing your draft.",
    icon: Workflow,
  },
  {
    title: "Review before saving",
    description: "Validate the generated config, inspect detected tools, and download the latest draft locally.",
    icon: Eye,
  },
];

export default function HomePage() {
  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(99,102,241,0.08),_transparent_40%),linear-gradient(180deg,#ffffff_0%,#f8fafc_100%)] px-6 py-20 text-slate-900">
      <div className="mx-auto flex max-w-6xl flex-col gap-14">
        <section className="grid gap-12 lg:grid-cols-[1.1fr_0.9fr] lg:items-center">
          <div>
            <p className="text-sm font-semibold uppercase tracking-[0.34em] text-indigo-600">Internal Agent Builder</p>
            <h1 className="mt-6 max-w-3xl text-5xl font-semibold tracking-tight text-slate-900 sm:text-6xl">
              Build, review, and export agent workflows from one workspace.
            </h1>
            <p className="mt-6 max-w-2xl text-lg leading-8 text-slate-600">
              The builder now supports streaming configuration previews, a visual workflow mode, and local draft saving so the core authoring flow is usable end-to-end.
            </p>

            <div className="mt-10 flex flex-wrap gap-4">
              <Link
                href="/builder"
                className="inline-flex items-center gap-2 rounded-2xl bg-indigo-600 px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-indigo-500/20 transition hover:-translate-y-0.5"
              >
                Open builder
                <ArrowRight className="h-4 w-4" />
              </Link>
              <Link
                href="/builder"
                className="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-5 py-3 text-sm font-semibold text-slate-700 shadow-sm transition hover:bg-slate-50"
              >
                Try sample prompts
              </Link>
            </div>
          </div>

          <div className="rounded-[2rem] border border-slate-200 bg-white/50 p-6 shadow-xl shadow-slate-200/50 backdrop-blur">
            <div className="rounded-[1.5rem] border border-slate-200 bg-white/80 p-6">
              <div className="flex items-center justify-between border-b border-slate-200 pb-4">
                <div>
                  <p className="text-xs uppercase tracking-[0.28em] text-indigo-600">Builder status</p>
                  <h2 className="mt-2 text-xl font-semibold text-slate-900">Ready for prompt-driven authoring</h2>
                </div>
                <span className="rounded-full border border-emerald-200 bg-emerald-50 px-3 py-1 text-xs font-semibold text-emerald-700">
                  Updated
                </span>
              </div>

              <dl className="mt-6 grid gap-4 sm:grid-cols-3">
                <div className="rounded-2xl border border-slate-200 bg-slate-50/50 p-4">
                  <dt className="text-xs uppercase tracking-[0.26em] text-slate-500">Streaming</dt>
                  <dd className="mt-2 text-2xl font-semibold text-slate-900">Live</dd>
                </div>
                <div className="rounded-2xl border border-slate-200 bg-slate-50/50 p-4">
                  <dt className="text-xs uppercase tracking-[0.26em] text-slate-500">Canvas mode</dt>
                  <dd className="mt-2 text-2xl font-semibold text-slate-900">Enabled</dd>
                </div>
                <div className="rounded-2xl border border-slate-200 bg-slate-50/50 p-4">
                  <dt className="text-xs uppercase tracking-[0.26em] text-slate-500">Save flow</dt>
                  <dd className="mt-2 text-2xl font-semibold text-slate-900">Local export</dd>
                </div>
              </dl>
            </div>
          </div>
        </section>

        <section className="grid gap-6 md:grid-cols-3">
          {highlights.map(({ title, description, icon: Icon }) => (
            <article key={title} className="rounded-[1.75rem] border border-slate-200 bg-white/60 p-6 backdrop-blur">
              <div className="inline-flex rounded-2xl bg-indigo-100 p-3 text-indigo-600">
                <Icon className="h-5 w-5" />
              </div>
              <h2 className="mt-5 text-xl font-semibold text-slate-900">{title}</h2>
              <p className="mt-3 text-sm leading-7 text-slate-600">{description}</p>
            </article>
          ))}
        </section>
      </div>
    </main>
  );
}
