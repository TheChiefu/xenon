import type { RequestLogin, RequestRegistrationCreate, RequestRegistrationRevoke, RespondLogin, RespondRegistration, RespondRegistrationCode } from "@/bindings/routes/auth";
import type { Registration, RegistrationCode } from "@/bindings/types/auth";
import { request } from "@/lib/utils";

export async function listRegistrationCodes(): Promise<RegistrationCode[]> {
    const response = await request("/codes");
    return await response.json() as RegistrationCode[];
}

export async function createRegistrationCode(body: RequestRegistrationCreate): Promise<string> {
    const response = await request("/codes", "POST", body);
    return (await response.json() as RespondRegistrationCode).code;
}

export async function revokeRegistrationCode(code: string): Promise<void> {
    const body: RequestRegistrationRevoke = { code };
    await request("/codes", "DELETE", body);
}

export async function login(username: string, password: string, url: string): Promise<string> {
    const request: RequestLogin = { username, password };

    const response = await fetch(`${url}/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(request),
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return (data as RespondLogin).token;
}

// Revokes an account's session on its server
export async function signOut(url: string, token: string): Promise<void> {
    await fetch(`${url}/me/session`, {
        method: "DELETE",
        headers: { "Authorization": `Bearer ${token}` },
    });
}

export async function register(
    username: string,
    password: string,
    display_name: string,
    registration_code: string | null,
    url: string
): Promise<string> {
    const request: Registration = { username, password, display_name, registration_code };

    const response = await fetch(`${url}/users`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(request),
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return (data as RespondRegistration).session_token;
}
