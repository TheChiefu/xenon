// Matches Bind::default() in: server/src/config.rs
// Update if the defaults ever change
const DEFAULT_BASE_URL = "http://localhost:3000";

export function getBaseUrl(): string {
    return localStorage.getItem('serverBaseUrl') ?? DEFAULT_BASE_URL;
}