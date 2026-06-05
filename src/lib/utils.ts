import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Merge Tailwind CSS classes
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Sleep for a given number of milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Debounce a function
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Get color for cost level
 */
export function getCostColor(cost: number): string {
  if (cost > 10) return 'text-red-500';
  if (cost > 5) return 'text-orange-500';
  if (cost > 1) return 'text-yellow-500';
  return 'text-green-500';
}

/**
 * Get color for percentage
 */
export function getPercentColor(pct: number): string {
  if (pct >= 80) return 'text-red-500';
  if (pct >= 60) return 'text-orange-500';
  if (pct >= 40) return 'text-yellow-500';
  return 'text-green-500';
}
