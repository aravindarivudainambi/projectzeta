"use client";

import { useState, useEffect } from "react";
import {
  Search,
  SlidersHorizontal,
  ArrowRight,
  Play,
  Copy,
  Check,
  X,
  XCircle,
  Bot,
} from "lucide-react";
import { useRouter } from "next/navigation";
import { forkMarketplaceTemplate, listAgents } from "@api-client";
import type { AgentConfig, MarketplaceTemplate } from "@schema-types";

interface AgentsClientProps {
  initialTemplates: MarketplaceTemplate[];
  userConnectedTools: string[];
}

export default function AgentsClient({
  initialTemplates,
  userConnectedTools,
}: AgentsClientProps) {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTools, setSelectedTools] = useState<string[]>([]);
  const [selectedDepartment, setSelectedDepartment] = useState<string>("All");
  const [selectedTemplate, setSelectedTemplate] =
    useState<MarketplaceTemplate | null>(null);
  const [isForking, setIsForking] = useState(false);
  const [forkError, setForkError] = useState<string | null>(null);

  // Saved agents from the backend
  const [savedAgents, setSavedAgents] = useState<AgentConfig[]>([]);
  const [isLoadingAgents, setIsLoadingAgents] = useState(true);

  useEffect(() => {
    listAgents()
      .then(setSavedAgents)
      .catch(() => setSavedAgents([]))
      .finally(() => setIsLoadingAgents(false));
  }, []);

  const allTools = Array.from(
    new Set(initialTemplates.flatMap((t) => t.toolBadges)),
  );
  const allDepartments = [
    "All",
    ...Array.from(new Set(initialTemplates.map((t) => t.department))),
  ];

  const filteredTemplates = initialTemplates.filter((template) => {
    const matchesSearch =
      template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.description.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesDepartment =
      selectedDepartment === "All" ||
      template.department === selectedDepartment;

    // If selectedTools has items, the template must have ALL selected tools
    const matchesTools =
      selectedTools.length === 0 ||
      selectedTools.every((tool) => template.toolBadges.includes(tool));

    return matchesSearch && matchesDepartment && matchesTools;
  });

  const toggleToolFilter = (tool: string) => {
    setSelectedTools((prev) =>
      prev.includes(tool) ? prev.filter((t) => t !== tool) : [...prev, tool],
    );
  };

  const handleFork = async (templateId: string) => {
    try {
      setForkError(null);
      setIsForking(true);

      const savedAgent = await forkMarketplaceTemplate({
        templateId,
      });

      router.push(`/agents/${savedAgent.id}`);
    } catch (error) {
      setForkError(
        error instanceof Error
          ? error.message
          : "Fork failed. Please try again.",
      );
    } finally {
      setIsForking(false);
    }
  };

  return (
    <div className="max-w-7xl mx-auto flex flex-col h-full space-y-8 p-6">
      {/* Header and Search */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 ">
            Community Agents
          </h1>
          <p className="text-gray-500  mt-1">
            Discover, fork, and automate workflows with agent templates shared
            by your team.
          </p>
        </div>
      </div>

      <div className="flex flex-col md:flex-row gap-4 items-start md:items-center bg-gray-50  p-4 rounded-xl">
        <div className="relative flex-1 w-full">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-400" />
          <input
            type="text"
            placeholder="Search agents..."
            className="w-full pl-10 pr-4 py-2 border border-gray-200  rounded-lg bg-white  focus:outline-none focus:ring-2 focus:ring-blue-500 "
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        <div className="flex gap-2 items-center w-full md:w-auto">
          <SlidersHorizontal className="h-5 w-5 text-gray-500 shrink-0" />
          <select
            className="bg-white  border border-gray-200  text-gray-700  rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
            value={selectedDepartment}
            onChange={(e) => setSelectedDepartment(e.target.value)}
          >
            {allDepartments.map((dept) => (
              <option key={dept} value={dept}>
                {dept} Department
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Filter Chips */}
      <div className="flex flex-wrap gap-2">
        {allTools.map((tool) => (
          <button
            key={tool}
            onClick={() => toggleToolFilter(tool)}
            className={`px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
              selectedTools.includes(tool)
                ? "bg-blue-100 text-blue-700   border border-blue-200 "
                : "bg-white  text-gray-600  border border-gray-200  hover:bg-gray-50 :bg-zinc-700"
            }`}
          >
            Uses {tool}
          </button>
        ))}
      </div>

      {/* My Agents — saved agents from backend */}
      {savedAgents.length > 0 && (
        <section>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-gray-900">My Agents</h2>
            <button
              onClick={() => router.push("/dashboard")}
              className="px-3 py-1.5 text-sm font-medium text-blue-600 hover:text-blue-700 transition-colors flex items-center gap-1.5"
            >
              Create New <ArrowRight className="h-3.5 w-3.5" />
            </button>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-10">
            {savedAgents.map((agent) => {
              const triggerLabel =
                typeof agent.trigger === "string"
                  ? agent.trigger
                  : "Schedule" in agent.trigger
                    ? `Schedule (${agent.trigger.Schedule.cron})`
                    : "Event" in agent.trigger
                      ? `${agent.trigger.Event.source}.${agent.trigger.Event.event}`
                      : "Unknown";

              const toolNames = agent.steps
                .map((s) => s.tool_name)
                .filter(Boolean) as string[];

              return (
                <div
                  key={agent.id}
                  className="bg-white border border-gray-200 rounded-xl p-5 hover:border-indigo-300 shadow-sm hover:shadow-md transition-all cursor-pointer flex flex-col"
                  onClick={() => router.push(`/agents/${agent.id}`)}
                >
                  <div className="flex items-center gap-3 mb-3">
                    <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-indigo-100 text-indigo-600">
                      <Bot className="h-4 w-4" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <h3 className="text-base font-semibold text-gray-900 truncate">
                        {agent.name}
                      </h3>
                      <p className="text-xs text-gray-500">
                        {agent.steps.length} step
                        {agent.steps.length !== 1 ? "s" : ""} &middot;{" "}
                        {triggerLabel}
                      </p>
                    </div>
                  </div>

                  {toolNames.length > 0 && (
                    <div className="flex flex-wrap gap-1.5 mt-auto pt-3 border-t border-gray-100">
                      {toolNames.map((tool) => (
                        <span
                          key={tool}
                          className="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded-md"
                        >
                          {tool}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Community Templates
          </h2>
        </section>
      )}

      {/* Grid */}
      {filteredTemplates.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredTemplates.map((template) => (
            <div
              key={template.id}
              className="bg-white  border border-gray-200  rounded-xl p-5 hover:border-blue-300 :border-blue-700 shadow-sm hover:shadow-md transition-all cursor-pointer flex flex-col h-full"
              onClick={() => setSelectedTemplate(template)}
            >
              <div className="flex justify-between items-start mb-4">
                <h3 className="text-lg font-semibold text-gray-900  line-clamp-1">
                  {template.name}
                </h3>
                <span
                  className={`text-xs px-2 py-0.5 rounded-full ${
                    template.complexity === "Low"
                      ? "bg-green-100 text-green-700  "
                      : template.complexity === "Medium"
                        ? "bg-yellow-100 text-yellow-700  "
                        : "bg-red-100 text-red-700  "
                  }`}
                >
                  {template.complexity}
                </span>
              </div>

              <p className="text-sm text-gray-600  flex-1 mb-4 line-clamp-2">
                {template.description}
              </p>

              <div className="flex flex-wrap gap-2 mb-4">
                {template.toolBadges.map((tool) => (
                  <span
                    key={tool}
                    className="text-xs bg-gray-100  text-gray-600  px-2 py-1 rounded-md"
                  >
                    {tool}
                  </span>
                ))}
              </div>

              <div className="mt-auto pt-4 border-t border-gray-100  flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="h-6 w-6 rounded-full bg-gradient-to-r from-indigo-400 to-purple-500 flex items-center justify-center text-white text-xs font-bold shrink-0">
                    {template.creatorName.charAt(0)}
                  </div>
                  <span className="text-xs text-gray-500 ">
                    {template.creatorName}
                  </span>
                </div>
                <div className="flex items-center text-xs text-gray-500  gap-1.5">
                  <Play className="h-3 w-3" />
                  {template.runCount.toLocaleString()} runs
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-20 text-center bg-gray-50  rounded-xl border border-dashed border-gray-200 ">
          <p className="text-gray-500  mb-4">
            No agents match — you can create one and share it.
          </p>
          <button
            onClick={() => router.push("/dashboard")}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium flex items-center gap-2"
          >
            Create New Agent <ArrowRight className="h-4 w-4" />
          </button>
        </div>
      )}

      {/* Detail Modal */}
      {selectedTemplate && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm transition-opacity">
          <div className="bg-white  rounded-2xl max-w-3xl w-full max-h-[90vh] overflow-y-auto shadow-2xl relative border border-gray-200  flex flex-col">
            <button
              onClick={() => setSelectedTemplate(null)}
              className="absolute top-4 right-4 p-2 text-gray-400 hover:text-gray-600 :text-gray-200 rounded-full hover:bg-gray-100 :bg-zinc-800 transition"
            >
              <X className="h-5 w-5" />
            </button>

            <div className="p-8 pb-6 border-b border-gray-100 ">
              <div className="flex items-center gap-3 mb-3">
                <span className="text-sm font-medium text-blue-600 ">
                  {selectedTemplate.department}
                </span>
                <span className="h-1 w-1 rounded-full bg-gray-300 "></span>
                <div className="flex items-center gap-2">
                  <div className="h-5 w-5 rounded-full bg-gradient-to-r from-indigo-400 to-purple-500 flex items-center justify-center text-white text-[10px] font-bold shrink-0">
                    {selectedTemplate.creatorName.charAt(0)}
                  </div>
                  <span className="text-sm text-gray-500 ">
                    Created by {selectedTemplate.creatorName}
                  </span>
                </div>
              </div>
              <h2 className="text-2xl font-bold text-gray-900  mb-2">
                {selectedTemplate.name}
              </h2>
              <p className="text-gray-600  leading-relaxed text-sm md:text-base">
                {selectedTemplate.fullDescription}
              </p>
            </div>

            <div className="p-8 grid grid-cols-1 md:grid-cols-3 gap-8">
              <div className="md:col-span-2 space-y-8">
                <div>
                  <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-500  mb-4">
                    Step-by-step Execution
                  </h3>
                  <ol className="relative border-l border-gray-200  ml-3 space-y-6">
                    {selectedTemplate.steps.map((step, idx) => (
                      <li key={idx} className="pl-6 relative">
                        <span className="absolute flex items-center justify-center w-6 h-6 bg-blue-100  rounded-full -left-3 top-0 ring-4 ring-white  text-blue-600  text-xs font-bold border border-blue-200 ">
                          {idx + 1}
                        </span>
                        <p className="text-sm text-gray-700  font-medium">
                          {step}
                        </p>
                      </li>
                    ))}
                  </ol>
                </div>

                <div>
                  <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-500  mb-3">
                    Example Output
                  </h3>
                  <div className="bg-gray-50  rounded-lg p-4 font-mono text-sm text-gray-800  whitespace-pre-wrap border border-gray-200 ">
                    {selectedTemplate.exampleOutput}
                  </div>
                </div>
              </div>

              <div className="space-y-6 flex flex-col">
                <div className="bg-gray-50  p-5 rounded-xl border border-gray-200 ">
                  <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-900  mb-4">
                    Required Connectors
                  </h3>
                  <div className="space-y-3">
                    {selectedTemplate.requiredConnectors.map((conn) => {
                      const isConnected = userConnectedTools.includes(conn);
                      return (
                        <div
                          key={conn}
                          className="flex items-center justify-between"
                        >
                          <span className="text-sm font-medium text-gray-700  flex items-center gap-2">
                            {conn}
                          </span>
                          {isConnected ? (
                            <span className="flex items-center text-xs text-green-600  bg-green-50  px-2 py-1 rounded-md border border-green-200 ">
                              <Check className="h-3 w-3 mr-1" /> Connected
                            </span>
                          ) : (
                            <span className="flex items-center text-xs text-red-600  bg-red-50  px-2 py-1 rounded-md border border-red-200 ">
                              <XCircle className="h-3 w-3 mr-1" /> Missing
                            </span>
                          )}
                        </div>
                      );
                    })}
                  </div>

                  {!selectedTemplate.requiredConnectors.every((conn) =>
                    userConnectedTools.includes(conn),
                  ) && (
                    <div className="mt-4 p-3 bg-amber-50  border border-amber-200  rounded-lg text-xs text-amber-800  leading-relaxed">
                      You need to connect missing integrations in settings
                      before this agent can run successfully.
                    </div>
                  )}

                  {forkError && (
                    <div className="mt-4 rounded-lg border border-red-200 bg-red-50 p-3 text-xs leading-relaxed text-red-700">
                      {forkError}
                    </div>
                  )}
                </div>

                <div className="mt-auto pt-4 flex flex-col gap-3">
                  <button
                    onClick={() => handleFork(selectedTemplate.id)}
                    disabled={isForking}
                    className="w-full py-3 bg-blue-600 hover:bg-blue-700 disabled:opacity-75 disabled:cursor-not-allowed text-white rounded-xl font-medium flex justify-center items-center gap-2 transition shadow-sm"
                  >
                    {isForking ? (
                      <div className="flex items-center gap-2">
                        <div className="h-4 w-4 rounded-full border-2 border-white/50 border-t-white animate-spin"></div>
                        <span>Forking...</span>
                      </div>
                    ) : (
                      <>
                        <Copy className="h-4 w-4" /> Fork to My Agents
                      </>
                    )}
                  </button>
                  <p className="text-center text-xs text-gray-500 ">
                    Creates a private copy you can edit.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
