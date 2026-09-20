import { getBaseUrl } from "@/lib/session";
import type { LoginRequest, LoginResponse } from "@/bindings/routes/auth";

export async function login(username: string, password: string): Promise<string> {
    const request: LoginRequest = { username, password };

    const response = await fetch(`${getBaseUrl()}/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(request),
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return (data as LoginResponse).token;
}
