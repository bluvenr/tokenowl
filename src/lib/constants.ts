export const DATA_SOURCES = [
  { key: "claude_code", name: "Claude Code" },
  { key: "codex_cli", name: "Codex CLI" },
  { key: "gemini_cli", name: "Gemini CLI" },
  { key: "kimi_code", name: "Kimi Code" },
  { key: "qwen_code", name: "Qwen Code" },
] as const;

export const LANGUAGES = [
  { key: "auto", label: "Auto" },
  { key: "zh-CN", label: "中文" },
  { key: "en", label: "English" },
] as const;
