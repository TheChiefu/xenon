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

export async function register(username: string, password: string, display_name: string, registration_code: string, url: string): Promise<string> {
    const request: RegisterRequest = { username, password, display_name, registration_code };

    const response = await fetch(`${url}/users`, {
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
