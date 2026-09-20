<script lang="ts">
    import { login } from "@/lib/api/auth";
    import { setToken } from "@/lib/session";

    interface Props {
      onLogin: () => void;
    }
    let { onLogin }: Props = $props();

    let username = $state("");
    let password = $state("");
    let display_name = $state("");
    let invite_code = $state("");

    let error = $state<string | null>(null);
    let pending = $state(false);
    let current_tab = $state(0);

    async function submit(event: SubmitEvent) {
      event.preventDefault();
      error = null;
      pending = true;

      try {
        const token = await login(username, password);
        setToken(token);
        onLogin();
      } catch (err) {
        error = err instanceof Error ? err.message : String(err);
      } finally {
        pending = false;
      }
    }

    function changeTab(tab: number) {
      current_tab = tab;
    }

</script>

<div class="login">
    <img src="../../../public/icon.svg" width="64" height="64" alt="Xenon Logo"/>
    <div>
        <h1>Xenon</h1>
        <div> <!-- Tabs -->
            <button onclick={() => changeTab(0)}>Sign In</button>
            <button onclick={() => changeTab(1)}>Register</button>
        </div>

        <!-- Primary Tabs -->
        {#if current_tab == 0}

            <!-- Sign In tab -->
            <form data-panel="login" onsubmit={submit}>
                <label for="username">Username</label>
                <input name="username" autocapitalize="none" spellcheck="false" bind:value={username} required/>
                <label for="password">Password</label>
                <input name="password" type="password" bind:value={password} required/>
                <button type="submit" disabled={pending}>Sign In</button>
            </form>
        {:else if current_tab == 1}

            <!-- Registration Tab -->
            <form data-panel="registration">
                <label for="username">Username</label>
                <input name="username" autocapitalize="none" spellcheck="false" bind:value={username} required/>
                <small>Lowercase letters, digits, underscores, and dashes only.</small>
                <label for="password">Password</label>
                <input name="password" type="password" bind:value={password} required/>
                <label for="display-name">Display Name</label>
                <input name="display-name" bind:value={display_name} required/>
                <small>Visible name to yourself and others on clients.</small>
                <label for="invite-code">Invite Code</label>
                <input name="invite-code" bind:value={invite_code} required/>
                <button type="submit" disabled={pending}>Create Account</button>
            </form>
        {:else}
            <p>Invalid tab, please reload page.</p>
        {/if}

        <!-- Error Messages-->
        {#if error}
            <p>Error: {error}</p>
        {/if}

        <!-- Server -->
        <form data-panel="server">
            <label for="base">Base URL</label>
            <input name="base" type="url" />
            <small>Leave empty when client is served from same origin as the server.</small>
            <button type="submit">Save</button>
        </form>


    </div>
</div>
