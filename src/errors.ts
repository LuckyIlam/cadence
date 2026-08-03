export function erreurMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  if (e && typeof e === "object") {
    const obj = e as Record<string, unknown>;
    if (typeof obj.message === "string") return obj.message;
    const premiere = Object.values(obj).find((v) => typeof v === "string");
    if (typeof premiere === "string") return premiere;
  }
  return String(e);
}
