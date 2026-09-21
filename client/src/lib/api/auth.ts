import type { LoginRequest, LoginResponse, RegisterRequest, RegisterResponse } from "@/bindings/routes/auth";

export async function login(username: string, password: string, url: string): Promise<string> {
    const request: LoginRequest = { username, password };

    const response = await fetch(`${url}/login`, {
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

export async function register(username: string, password: string, display_name: string, invite_code: string, url: string): Promise<string> {
    const request: RegisterRequest = { username, password, display_name, invite_code };

    const response = await fetch(`${url}/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(request),
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return (data as RegisterResponse).session_token;
}
