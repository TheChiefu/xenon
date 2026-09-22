<script lang="ts">
  import { login, register } from "@/lib/api/auth";
  import { addLogin, getLogins, resumeLogin, removeLogin } from "@/lib/session.svelte";
  import iconX from "@/assets/icons/x.svg?raw";

  // Matches Bind::default() in: server/src/config.rs
  export const DEFAULT_BASE_URL = "http://localhost:3000";
  // User Credentials
  let username = $state("");
  let password = $state("");
  let url = $state(DEFAULT_BASE_URL);
  let display_name = $state("");
  let invite_code = $state("");

  // View State
  let error = $state<string | null>(null);
  let pending = $state(false);
  let current_tab = $state(0);
  let showForm = $state(false);

  // Whether any login has ever been saved
  function hasSavedLogins(): boolean {
    return Object.keys(getLogins()).length > 0;
  }

  // Attempt to login
  async function tryLogin(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    pending = true;

    try {
      // Remove trailing '/' to avoid fetch error confusion on users
      const clean_url = (url || DEFAULT_BASE_URL).replace(/\/+$/, "");

      // Attempt to fetch via cleaned URL
      const token = await login(username, password, clean_url);
      addLogin(username, clean_url, token);
      showForm = false;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      pending = false;
    }
  }

  // Attempt to create an account
  async function tryRegister(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    pending = true;

    try {
      // Remove trailing '/' to avoid fetch error confusion on users
      const clean_url = (url || DEFAULT_BASE_URL).replace(/\/+$/, "");

      // Attempt to fetch via cleaned URL
      const token = await register(username, password, display_name, invite_code, clean_url);
      addLogin(username, clean_url, token);
      showForm = false;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      pending = false;
    }
  }

  // Cycle between login/registration items
  function changeTab(tab: number) {
    current_tab = tab;
  }

  // Returns "is-active" if the given tab index is the current one
  function tabClass(tab: number): string {
    return current_tab == tab ? 'is-active' : '';
  }

  // Prompt before removing a saved login
  function confirmRemoveLogin(key: string, username: string) {
    if (confirm(`Remove ${username}?`)) {
      removeLogin(key);
    }
  }

  // Reveal the sign-in form to add another account
  function startAddAccount() {
    username = "";
    password = "";
    url = DEFAULT_BASE_URL;
    error = null;
    current_tab = 0;
    showForm = true;
  }

  function toggleForm() {
    showForm = !showForm;
  }

</script>

<style>
  .login {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
  }

  .login-card {
    width: 25rem;
    padding: 1.5rem;
    border: 1px solid var(--border);
    background-color: var(--component);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
  }

  .login-entry {
    display: flex;
    align-items: stretch;
  }

  .login-entries {
      width: 100%;
      border: 1px solid var(--border-dim);
      background-color: var(--background);
  }

  .login-entry-main {
    flex: 1;
    text-align: left;
    border: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .login-remove {
    padding: 0.5rem;
    flex: none;
    display: flex;
    align-items: center;
  }

  form, .login-entry, button[onclick] {
    width: 100%;
  }

  .login-form-panel {
    width: 100%;
    border: 1px solid var(--border);
    margin-top: 1rem;
  }

  .login-tabs {
    display: flex;
    width: 100%;
  }

  .login-tabs button {
    flex: 1;
  }

  form {
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
  }

  .field {
    display: grid;
    gap: 0.3rem;
  }

  .saved {
      display:grid;
      width: 100%;
      gap: 0.75rem;
      text-align: center;
  }

</style>

<div class="login">
    <div class="login-card">
        <img src="../../../public/icon-192.png" width="64" height="64" alt="Xenon Logo"/>

        <h2>Xenon</h2>

        {#if !hasSavedLogins() || showForm}

            <div class="login-form-panel">

                <div class="login-tabs">
                    <button class="is-tab {tabClass(0)}" onclick={() => changeTab(0)}>Sign In</button>
                    <button class="is-tab {tabClass(1)}" onclick={() => changeTab(1)}>Register</button>
                </div>

                <!-- Primary Tabs -->
                {#if current_tab == 0}

                    <!-- Sign In tab -->
                    <form data-panel="login" onsubmit={tryLogin}>
                        <label for="username">Username</label>
                        <input name="username" autocapitalize="none" spellcheck="false" bind:value={username} required/>
                        <label for="password">Password</label>
                        <input name="password" type="password" bind:value={password} required/>
                        <details>
                            <summary>Server</summary>
                            <input name="url" type="url" bind:value={url}/>
                        </details>
                        <button class="is-action" type="submit" disabled={pending}>Sign In</button>
                    </form>
                {:else if current_tab == 1}

                    <!-- Registration Tab -->
                    <form data-panel="registration" onsubmit={tryRegister}>
                        <div class="field">
                            <label for="username">Username</label>
                            <input name="username" autocapitalize="none" spellcheck="false" bind:value={username} required/>
                            <small>Allowed: lower, underscore, digit, and dashes</small>
                        </div>
                        <div class="field">
                            <label for="password">Password</label>
                            <input name="password" type="password" bind:value={password} required/>
                            <small>Minimum/Maximum: Set by server owner</small>
                        </div>
                        <div class="field">
                            <label for="display-name">Display Name</label>
                            <input name="display-name" bind:value={display_name} required/>
                            <small>Name shown on client (any characters)</small>
                        </div>
                        <div class="field">
                            <label for="invite-code">Invite Code</label>
                            <input name="invite-code" bind:value={invite_code} required/>
                            <small>Leave blank if not required</small>
                        </div>
                        <div class="field">
                            <label for="url">Server Domain</label>
                            <input name="url" type="url" bind:value={url} required/>
                            <small>Leave default if server is same as current domain</small>
                        </div>
                        <button class="is-action" type="submit" disabled={pending}>Create Account</button>
                    </form>
                {:else}
                    <p>Invalid tab, please reload page.</p>
                {/if}

                <!-- Error Messages-->
                {#if error}
                    <p>Error: {error}</p>
                {/if}
            </div>

            {#if hasSavedLogins()}
                <button class="is-action" type="submit" onclick={toggleForm}>
                    Back to Accounts
                </button>
            {/if}

        {:else}
            <!-- Saved logins -->
            <div class="saved">
                <div class="login-entries">
                    {#each Object.entries(getLogins()) as [key, entry]}
                        <div class="login-entry">
                            <button class="is-entry login-entry-main" onclick={() => resumeLogin(key)}>
                                @{entry.username}<br/>{entry.url}
                            </button>
                            <button class="inherit is-danger is-entry" aria-label="Remove {entry.username}" onclick={() => confirmRemoveLogin(key, entry.username)}>
                                <span class="icon" aria-hidden="true">{@html iconX}</span>
                            </button>
                        </div>
                    {/each}
                </div>
                <button class="is-action" onclick={startAddAccount}>Add Account</button>
            </div>

        {/if}
    </div>
</div>
