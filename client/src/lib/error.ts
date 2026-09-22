import type { ErrorResponse } from "@/bindings/error";

// Verifies unknown JSON actually matches the server's ErrorResponse shape
// before it's trusted. Returns the parsed value, or null if it doesn't fit.
export function parseErrorResponse(value: unknown): ErrorResponse | null {

  // confirms value is an actual object, not null
  if (typeof value !== "object" || value === null) {
    return null;
  }

  // Check if ErrorResponse's current field count match server binding
  if (Object.keys(value).length !== 1) {
    return null;
  }

  // Check if the "error" field exists and is a string
  const errorValue = (value as Record<string, unknown>).error;
  if (typeof errorValue !== "string") {
    return null;
  }

  return {
    error: errorValue
  };
}
