/**
 * Format USD currency (default 4 decimal places for precision)
 */
export function formatUsd(amount: number, decimals = 4): string {
  if (amount >= 1000) {
    return `$${(amount / 1000).toFixed(2)}k`;
  }
  return `$${amount.toFixed(decimals)}`;
}

/**
 * Format token count with abbreviations
 */
export function formatTokens(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}k`;
  }
  return count.toString();
}

/**
 * Format percentage
 */
export function formatPercent(value: number, decimals = 1): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 7) {
    return d.toLocaleDateString();
  }
  if (diffDays > 0) {
    return `${diffDays}天前`;
  }
  if (diffHours > 0) {
    return `${diffHours}小时前`;
  }
  if (diffMins > 0) {
    return `${diffMins}分钟前`;
  }
  return '刚刚';
}

/**
 * Format date for display
 */
export function formatDate(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Format datetime for display (full: year-month-day hour:minute:second)
 */
export function formatDateTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
}

/**
 * Format file size
 */
export function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824) {
    return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
  }
  if (bytes >= 1_048_576) {
    return `${(bytes / 1_048_576).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${bytes} B`;
}

/**
 * Format latency in seconds (2 decimal places)
 */
export function formatLatency(ms: number): string {
  return `${(ms / 1000).toFixed(2)}s`;
}

/**
 * Format model name for display
 */
export function formatModelName(model: string): string {
  // Common model name transformations
  const mappings: Record<string, string> = {
    'claude-3-5-sonnet': 'Claude 3.5 Sonnet',
    'claude-3-opus': 'Claude 3 Opus',
    'gpt-4-turbo': 'GPT-4 Turbo',
    'gpt-4o': 'GPT-4o',
    'gpt-4o-mini': 'GPT-4o Mini',
  };
  return mappings[model.toLowerCase()] || model;
}
