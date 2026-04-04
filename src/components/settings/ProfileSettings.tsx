import { useState, useEffect, useCallback, useRef } from "react";
import type { MeetingTemplate, TriggerConfig } from "../../lib/types";
import { getTemplates, saveTemplate as saveTemplateApi, deleteTemplate as deleteTemplateApi } from "../../lib/tauri";

export function ProfileSettings() {
  const [templates, setTemplates] = useState<MeetingTemplate[]>([]);
  const [selected, setSelected] = useState<MeetingTemplate | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    getTemplates().then((list) => {
      setTemplates(list);
      if (list.length > 0 && !selected) setSelected(list[0]);
    }).catch(console.error);
  }, []);

  const autoSave = useCallback((template: MeetingTemplate) => {
    if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    saveTimeoutRef.current = setTimeout(() => {
      saveTemplateApi(template).catch(console.error);
    }, 500);
  }, []);

  const updateField = <K extends keyof MeetingTemplate>(field: K, value: MeetingTemplate[K]) => {
    if (!selected) return;
    const updated = { ...selected, [field]: value };
    setSelected(updated);
    setTemplates((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
    autoSave(updated);
  };

  const updateTrigger = <K extends keyof TriggerConfig>(field: K, value: TriggerConfig[K]) => {
    if (!selected) return;
    const updatedConfig = { ...selected.trigger_config, [field]: value };
    updateField("trigger_config", updatedConfig);
  };

  const handleKeywordsChange = (field: "custom_keywords" | "domain_keywords", text: string) => {
    const keywords = text.split(/[,，\n]/).map((s) => s.trim()).filter(Boolean);
    updateTrigger(field, keywords);
  };

  const handleNewTemplate = async () => {
    const id = `custom-${Date.now()}`;
    const newTemplate: MeetingTemplate = {
      id,
      name: "新模板",
      description: "",
      system_prompt: "",
      trigger_hints: [],
      advice_style: "general",
      enabled: true,
      role_persona: "",
      mimic_style: "",
      expertise_context: "",
      trigger_config: {
        on_ask_opinion: true,
        on_domain_topic: true,
        on_decision_point: true,
        on_discussion_stuck: true,
        custom_keywords: [],
        domain_keywords: [],
      },
    };
    await saveTemplateApi(newTemplate);
    setTemplates((prev) => [...prev, newTemplate]);
    setSelected(newTemplate);
  };

  const handleDelete = async () => {
    if (!selected) return;
    await deleteTemplateApi(selected.id);
    setTemplates((prev) => prev.filter((t) => t.id !== selected.id));
    setSelected(templates.find((t) => t.id !== selected.id) || null);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold mb-1">发言建议配置</h2>
          <p className="text-sm text-[var(--text-muted)]">
            配置不同会议场景下的发言建议策略
          </p>
        </div>
        <button
          onClick={handleNewTemplate}
          className="px-3 py-1.5 text-xs bg-[var(--accent-purple)] text-white rounded hover:opacity-90"
        >
          + 新建模板
        </button>
      </div>

      <div className="flex gap-4">
        {/* Template list */}
        <div className="w-48 space-y-1 shrink-0">
          {templates.map((t) => (
            <button
              key={t.id}
              onClick={() => setSelected(t)}
              className={`w-full text-left px-3 py-2 rounded text-sm ${
                selected?.id === t.id
                  ? "bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-card)]"
              }`}
            >
              <div className="font-medium truncate">{t.name}</div>
              <div className="text-xs text-[var(--text-muted)] truncate">{t.description}</div>
            </button>
          ))}
        </div>

        {/* Edit form */}
        {selected && (
          <div className="flex-1 space-y-5">
            {/* Basic info */}
            <section className="space-y-3">
              <h3 className="text-sm font-medium text-[var(--text-muted)] uppercase">基本信息</h3>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs text-[var(--text-muted)]">模板名称</label>
                  <input
                    value={selected.name}
                    onChange={(e) => updateField("name", e.target.value)}
                    className="w-full mt-1 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-sm"
                  />
                </div>
                <div>
                  <label className="text-xs text-[var(--text-muted)]">简介</label>
                  <input
                    value={selected.description}
                    onChange={(e) => updateField("description", e.target.value)}
                    className="w-full mt-1 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-sm"
                  />
                </div>
              </div>
            </section>

            {/* Role persona */}
            <section className="space-y-3">
              <h3 className="text-sm font-medium text-[var(--text-muted)] uppercase">角色设定</h3>
              <div>
                <label className="text-xs text-[var(--text-muted)]">你的角色定位</label>
                <textarea
                  value={selected.role_persona}
                  onChange={(e) => updateField("role_persona", e.target.value)}
                  placeholder="例如：前端技术专家，目标成为小组长，要展示技术深度和leadership"
                  rows={2}
                  className="w-full mt-1 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-sm resize-none"
                />
              </div>
              <div>
                <label className="text-xs text-[var(--text-muted)]">模仿风格（可选）</label>
                <textarea
                  value={selected.mimic_style}
                  onChange={(e) => updateField("mimic_style", e.target.value)}
                  placeholder="例如：像张一鸣一样简洁直接、数据驱动"
                  rows={2}
                  className="w-full mt-1 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-sm resize-none"
                />
              </div>
            </section>

            {/* Expertise */}
            <section className="space-y-3">
              <h3 className="text-sm font-medium text-[var(--text-muted)] uppercase">专业背景</h3>
              <div>
                <label className="text-xs text-[var(--text-muted)]">
                  专业知识和背景（会注入到 AI 的上下文中）
                </label>
                <textarea
                  value={selected.expertise_context}
                  onChange={(e) => updateField("expertise_context", e.target.value)}
                  placeholder="例如：熟悉 React 性能优化、Webpack 构建优化、微前端架构。负责过千万 DAU 的电商前端项目。"
                  rows={4}
                  className="w-full mt-1 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-sm resize-none"
                />
              </div>
            </section>

            {/* Trigger config */}
            <section className="space-y-3">
              <h3 className="text-sm font-medium text-[var(--text-muted)] uppercase">发言时机</h3>
              <div className="space-y-2">
                {[
                  { key: "on_ask_opinion" as const, label: "有人征求意见时", desc: "检测到\"大家觉得怎么样\"\"前端怎么看\"等表达" },
                  { key: "on_domain_topic" as const, label: "讨论涉及你的领域时", desc: "检测到下方配置的领域关键词" },
                  { key: "on_decision_point" as const, label: "出现技术决策点时", desc: "检测到方案争论、需要拍板等场景" },
                  { key: "on_discussion_stuck" as const, label: "讨论卡住或跑偏时", desc: "语速骤降或出现\"跑题了\"等信号" },
                ].map(({ key, label, desc }) => (
                  <label key={key} className="flex items-start gap-3 p-2 rounded hover:bg-[var(--bg-card)]">
                    <input
                      type="checkbox"
                      checked={selected.trigger_config[key]}
                      onChange={(e) => updateTrigger(key, e.target.checked)}
                      className="mt-0.5 accent-[var(--accent-purple)]"
                    />
                    <div>
                      <div className="text-sm">{label}</div>
                      <div className="text-xs text-[var(--text-muted)]">{desc}</div>
                    </div>
                  </label>
                ))}
              </div>

              <div>
                <label className="text-xs text-[var(--text-muted)]">
                  领域关键词（逗号分隔，讨论中出现这些词时触发建议）
                </label>
                <textarea
                  value={selected.trigger_config.domain_keywords.join("，")}
                  onChange={(e) => handleKeywordsChange("domain_keywords", e.target.value)}
                  placeholder="前端，组件，React，性能优化"
                  rows={2}
                  className="w-full mt-1 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-sm resize-none"
                />
              </div>

              <div>
                <label className="text-xs text-[var(--text-muted)]">
                  自定义触发关键词（逗号分隔）
                </label>
                <textarea
                  value={selected.trigger_config.custom_keywords.join("，")}
                  onChange={(e) => handleKeywordsChange("custom_keywords", e.target.value)}
                  placeholder="进度怎么样，有什么阻塞，deadline"
                  rows={2}
                  className="w-full mt-1 px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-sm resize-none"
                />
              </div>
            </section>

            {/* Advanced: raw system prompt */}
            <section>
              <button
                onClick={() => setShowAdvanced(!showAdvanced)}
                className="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)]"
              >
                {showAdvanced ? "▼" : "▶"} 高级设置（原始 System Prompt）
              </button>
              {showAdvanced && (
                <div className="mt-2">
                  <p className="text-xs text-[var(--text-muted)] mb-1">
                    如果上方角色/背景配置为空，将直接使用此 Prompt。否则系统会自动从上方配置生成。
                  </p>
                  <textarea
                    value={selected.system_prompt}
                    onChange={(e) => updateField("system_prompt", e.target.value)}
                    rows={8}
                    className="w-full px-3 py-1.5 bg-[var(--bg-card)] border border-[var(--border)] rounded text-xs font-mono resize-none"
                  />
                </div>
              )}
            </section>

            {/* Delete */}
            <div className="pt-2 border-t border-[var(--border)]">
              <button
                onClick={handleDelete}
                className="text-xs text-red-400 hover:text-red-300"
              >
                删除此模板
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
