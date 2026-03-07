"use client";

import { useState } from "react";
import { Search, SlidersHorizontal, ArrowRight, Play, Copy, Check, X, XCircle } from "lucide-react";
import Image from "next/image";
import { useRouter } from "next/navigation";

export interface AgentTemplate {
  id: string;
  name: string;
  description: string;
  fullDescription: string;
  toolBadges: string[];
  runCount: number;
  creatorName: string;
  creatorAvatar: string;
  department: string;
  complexity: 'Low' | 'Medium' | 'High';
  exampleOutput: string;
  steps: string[];
  requiredConnectors: string[];
}

interface MarketplaceClientProps {
  initialTemplates: AgentTemplate[];
  userConnectedTools: string[];
}

export default function MarketplaceClient({ initialTemplates, userConnectedTools }: MarketplaceClientProps) {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTools, setSelectedTools] = useState<string[]>([]);
  const [selectedDepartment, setSelectedDepartment] = useState<string>("All");
  const [selectedTemplate, setSelectedTemplate] = useState<AgentTemplate | null>(null);
  const [isForking, setIsForking] = useState(false);

  const allTools = Array.from(new Set(initialTemplates.flatMap(t => t.toolBadges)));
  const allDepartments = ["All", ...Array.from(new Set(initialTemplates.map(t => t.department)))];

  const filteredTemplates = initialTemplates.filter(template => {
    const matchesSearch = template.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
                          template.description.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesDepartment = selectedDepartment === "All" || template.department === selectedDepartment;
    
    // If selectedTools has items, the template must have ALL selected tools
    const matchesTools = selectedTools.length === 0 || 
                         selectedTools.every(tool => template.toolBadges.includes(tool));

    return matchesSearch && matchesDepartment && matchesTools;
  });

  const toggleToolFilter = (tool: string) => {
    setSelectedTools(prev => 
      prev.includes(tool) ? prev.filter(t => t !== tool) : [...prev, tool]
    );
  };

  const handleFork = async (templateId: string) => {
    setIsForking(true);
    await new Promise(r => setTimeout(r, 800)); // Simulate fork latency
    setIsForking(false);
    router.push(`/builder?templateId=${templateId}`);
  };

  return (
    <div className="max-w-7xl mx-auto flex flex-col h-full space-y-8">
      {/* Header and Search */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Agent Marketplace</h1>
          <p className="text-gray-500 dark:text-gray-400 mt-1 pb-4">
            Discover, fork, and automate workflows with agent templates shared by your team.
          </p>
        </div>
      </div>

      <div className="flex flex-col md:flex-row gap-4 items-start md:items-center bg-gray-50 dark:bg-zinc-800/50 p-4 rounded-xl">
        <div className="relative flex-1 w-full">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-400" />
          <input
            type="text"
            placeholder="Search templates..."
            className="w-full pl-10 pr-4 py-2 border border-gray-200 dark:border-zinc-700 rounded-lg bg-white dark:bg-zinc-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-white"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        
        <div className="flex gap-2 items-center w-full md:w-auto">
          <SlidersHorizontal className="h-5 w-5 text-gray-500 shrink-0" />
          <select 
            className="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 text-gray-700 dark:text-gray-200 rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
            value={selectedDepartment}
            onChange={(e) => setSelectedDepartment(e.target.value)}
          >
            {allDepartments.map(dept => (
              <option key={dept} value={dept}>{dept} Department</option>
            ))}
          </select>
        </div>
      </div>

      {/* Filter Chips */}
      <div className="flex flex-wrap gap-2">
        {allTools.map(tool => (
          <button
            key={tool}
            onClick={() => toggleToolFilter(tool)}
            className={`px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
              selectedTools.includes(tool) 
                ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300 border border-blue-200 dark:border-blue-800' 
                : 'bg-white dark:bg-zinc-800 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-zinc-700 hover:bg-gray-50 dark:hover:bg-zinc-700'
            }`}
          >
            Uses {tool}
          </button>
        ))}
      </div>

      {/* Grid */}
      {filteredTemplates.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredTemplates.map(template => (
            <div 
              key={template.id} 
              className="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 rounded-xl p-5 hover:border-blue-300 dark:hover:border-blue-700 shadow-sm hover:shadow-md transition-all cursor-pointer flex flex-col h-full"
              onClick={() => setSelectedTemplate(template)}
            >
              <div className="flex justify-between items-start mb-4">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white line-clamp-1">{template.name}</h3>
                <span className={`text-xs px-2 py-0.5 rounded-full ${
                  template.complexity === 'Low' ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 
                  template.complexity === 'Medium' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400' :
                  'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
                }`}>
                  {template.complexity}
                </span>
              </div>
              
              <p className="text-sm text-gray-600 dark:text-gray-400 flex-1 mb-4 line-clamp-2">{template.description}</p>
              
              <div className="flex flex-wrap gap-2 mb-4">
                {template.toolBadges.map(tool => (
                  <span key={tool} className="text-xs bg-gray-100 dark:bg-zinc-800 text-gray-600 dark:text-gray-300 px-2 py-1 rounded-md">
                    {tool}
                  </span>
                ))}
              </div>
              
              <div className="mt-auto pt-4 border-t border-gray-100 dark:border-zinc-800 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="h-6 w-6 rounded-full bg-gradient-to-r from-indigo-400 to-purple-500 flex items-center justify-center text-white text-xs font-bold shrink-0">
                    {template.creatorName.charAt(0)}
                  </div>
                  <span className="text-xs text-gray-500 dark:text-gray-400">{template.creatorName}</span>
                </div>
                <div className="flex items-center text-xs text-gray-500 dark:text-gray-400 gap-1.5">
                  <Play className="h-3 w-3" />
                  {template.runCount.toLocaleString()} runs
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-20 text-center bg-gray-50 dark:bg-zinc-800/30 rounded-xl border border-dashed border-gray-200 dark:border-zinc-700">
          <p className="text-gray-500 dark:text-gray-400 mb-4">No templates match — you can create one and share it.</p>
          <button 
            onClick={() => router.push('/builder')}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium flex items-center gap-2"
          >
            Create New Agent <ArrowRight className="h-4 w-4" />
          </button>
        </div>
      )}

      {/* Detail Modal */}
      {selectedTemplate && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm transition-opacity">
          <div className="bg-white dark:bg-zinc-900 rounded-2xl max-w-3xl w-full max-h-[90vh] overflow-y-auto shadow-2xl relative border border-gray-200 dark:border-zinc-800 flex flex-col">
            <button 
              onClick={() => setSelectedTemplate(null)}
              className="absolute top-4 right-4 p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-full hover:bg-gray-100 dark:hover:bg-zinc-800 transition"
            >
              <X className="h-5 w-5" />
            </button>
            
            <div className="p-8 pb-6 border-b border-gray-100 dark:border-zinc-800">
              <div className="flex items-center gap-3 mb-3">
                <span className="text-sm font-medium text-blue-600 dark:text-blue-400">{selectedTemplate.department}</span>
                <span className="h-1 w-1 rounded-full bg-gray-300 dark:bg-zinc-600"></span>
                <div className="flex items-center gap-2">
                  <div className="h-5 w-5 rounded-full bg-gradient-to-r from-indigo-400 to-purple-500 flex items-center justify-center text-white text-[10px] font-bold shrink-0">
                    {selectedTemplate.creatorName.charAt(0)}
                  </div>
                  <span className="text-sm text-gray-500 dark:text-gray-400">Created by {selectedTemplate.creatorName}</span>
                </div>
              </div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">{selectedTemplate.name}</h2>
              <p className="text-gray-600 dark:text-gray-300 leading-relaxed text-sm md:text-base">
                {selectedTemplate.fullDescription}
              </p>
            </div>

            <div className="p-8 grid grid-cols-1 md:grid-cols-3 gap-8">
              <div className="md:col-span-2 space-y-8">
                <div>
                  <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-4">Step-by-step Execution</h3>
                  <ol className="relative border-l border-gray-200 dark:border-zinc-700 ml-3 space-y-6">
                    {selectedTemplate.steps.map((step, idx) => (
                      <li key={idx} className="pl-6 relative">
                        <span className="absolute flex items-center justify-center w-6 h-6 bg-blue-100 dark:bg-blue-900/30 rounded-full -left-3 top-0 ring-4 ring-white dark:ring-zinc-900 text-blue-600 dark:text-blue-400 text-xs font-bold border border-blue-200 dark:border-blue-800">
                          {idx + 1}
                        </span>
                        <p className="text-sm text-gray-700 dark:text-gray-200 font-medium">{step}</p>
                      </li>
                    ))}
                  </ol>
                </div>
                
                <div>
                  <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-3">Example Output</h3>
                  <div className="bg-gray-50 dark:bg-zinc-950 rounded-lg p-4 font-mono text-sm text-gray-800 dark:text-gray-300 whitespace-pre-wrap border border-gray-200 dark:border-zinc-800">
                    {selectedTemplate.exampleOutput}
                  </div>
                </div>
              </div>

              <div className="space-y-6 flex flex-col">
                <div className="bg-gray-50 dark:bg-zinc-800/50 p-5 rounded-xl border border-gray-200 dark:border-zinc-700">
                  <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-900 dark:text-white mb-4">Required Connectors</h3>
                  <div className="space-y-3">
                    {selectedTemplate.requiredConnectors.map(conn => {
                      const isConnected = userConnectedTools.includes(conn);
                      return (
                        <div key={conn} className="flex items-center justify-between">
                          <span className="text-sm font-medium text-gray-700 dark:text-gray-200 flex items-center gap-2">
                            {conn}
                          </span>
                          {isConnected ? (
                            <span className="flex items-center text-xs text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 px-2 py-1 rounded-md border border-green-200 dark:border-green-800/50">
                              <Check className="h-3 w-3 mr-1" /> Connected
                            </span>
                          ) : (
                            <span className="flex items-center text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 px-2 py-1 rounded-md border border-red-200 dark:border-red-800/50">
                              <XCircle className="h-3 w-3 mr-1" /> Missing
                            </span>
                          )}
                        </div>
                      )
                    })}
                  </div>
                  
                  {!selectedTemplate.requiredConnectors.every(conn => userConnectedTools.includes(conn)) && (
                    <div className="mt-4 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800/50 rounded-lg text-xs text-amber-800 dark:text-amber-200 leading-relaxed">
                      You need to connect missing integrations in settings before this agent can run successfully.
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
                  <p className="text-center text-xs text-gray-500 dark:text-gray-400">
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
