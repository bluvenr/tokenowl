/// Format USD amount: $12.34 or $0.50
export function formatCost(usd: number): string {
  if (usd >= 1000) {
    return `$${(usd / 1000).toFixed(2)}k`;
  }
  if (usd >= 1) {
    return `$${usd.toFixed(2)}`;
  }
  if (usd >= 0.01) {
    return `$${usd.toFixed(2)}`;
  }
  if (usd > 0) {
    return `$${usd.toFixed(4)}`;
  }
  return "$0.00";
}

/// Format token count: 1.2M, 450K, etc.
export function formatTokens(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}K`;
  }
  return count.toString();
}

/// Source color for charts
export function getSourceColor(source: string): string {
  const colors: Record<string, string> = {
    cc_switch: "#D97706",
    claude_code: "#D97706",
    codex_cli: "#10B981",
    gemini_cli: "#3B82F6",
  };
  return colors[source] || "#6B7280";
}

/// Model color palette for charts (cycling through preset colors)
const MODEL_COLORS = [
  "#D97706", "#3B82F6", "#10B981", "#EC4899", "#8B5CF6",
  "#EF4444", "#14B8A6", "#F59E0B", "#6366F1", "#06B6D4",
];

export function getModelColor(_model: string, index: number): string {
  return MODEL_COLORS[index % MODEL_COLORS.length];
}
