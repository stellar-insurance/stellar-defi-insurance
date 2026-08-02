// Fix: Handle edge cases
export function safeFormat(value: number | null | undefined): string {
  if (value === null || value === undefined || isNaN(value)) return '0';
  return value.toFixed(2);
}
