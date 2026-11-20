export function isNonEmpty(value: string | null | undefined): value is string {
  return value !== null && value !== undefined && value !== "";
}
