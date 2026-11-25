<script lang="ts">
  import { Notify, Permission, Visibility } from "@/bindings/types/rooms";
  import { DESCRIPTIONS_VISIBILITY, LABELS_NOTIFY, LABELS_PERMISSION, LABELS_VISIBILITY, MSG_DELETE_ROOM, MSG_ERROR, MSG_LEAVE_ROOM, MSG_SAVED } from "@/lib/labels";
  import * as events from "@/lib/events";
  import * as settings from "@/lib/rooms/settings.svelte";

  const permissions = $derived(settings.getPermissions());
  const can_rename = $derived(permissions.includes(Permission.rename));
  const can_grant = $derived(permissions.includes(Permission.grant));
  const can_delete = $derived(permissions.includes(Permission.delete_room));

  const current = settings.getRoom();
  let name = $state(current?.name ?? "");
  let visibility = $state<Visibility>(current?.visibility ?? Visibility.public);
  let granted = $state<Permission[]>(current?.default_permissions ?? []);
  let saved_text = $state("");

  async function changeNotify(event: Event) {
    if (!(event.currentTarget instanceof HTMLSelectElement)) return;
    await settings.saveNotify(event.currentTarget.value as Notify);
  }

  async function leave() {
    if (!await events.confirm(MSG_LEAVE_ROOM)) return;
    await settings.leave();
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saved_text = "";

    const saved = await settings.saveRoom({
      name: can_rename ? name : null,
      visibility: can_rename ? visibility : null,
      default_permissions: can_grant ? granted : null,
    });

    if (saved) saved_text = MSG_SAVED;
  }

  async function deleteRoom() {
    if (!await events.confirm(MSG_DELETE_ROOM)) return;
    await settings.deleteRoom();
  }
</script>

<style>
  fieldset {
    display: grid;
    gap: var(--row-gap);
    border: 1px solid var(--border);
  }

</style>

<div class="section fields">
  <div class="grid-title full-row">
    <h3>Membership</h3>
  </div>

  <label for="room-notify">Notify me about</label>
  <select id="room-notify" value={settings.getNotify()} onchange={changeNotify} disabled={settings.isPending()}>
    {#each Object.values(Notify) as value}
      <option {value}>{LABELS_NOTIFY[value]}</option>
    {/each}
  </select>

  <button class="full-row is-danger" onclick={leave} disabled={settings.isPending()}>Leave Room</button>
</div>

{#if can_rename || can_grant || can_delete}
  <div class="section fields">
    <div class="grid-title full-row">
      <h3>Room</h3>
      <span class="notify">{saved_text}</span>
    </div>

    {#if can_rename || can_grant}
      <form class="fields full-row" onsubmit={save}>
        {#if can_rename}
          <label for="room-name">Name</label>
          <input id="room-name" bind:value={name} required/>

          <label for="room-visibility">Visibility</label>
          <select id="room-visibility" bind:value={visibility}>
            {#each Object.values(Visibility) as value}
              <option {value} title={DESCRIPTIONS_VISIBILITY[value]}>{LABELS_VISIBILITY[value]}</option>
            {/each}
          </select>
        {/if}

        {#if can_grant}
          <fieldset class="full-row">
            <legend>Members join with</legend>
            {#each Object.values(Permission) as value}
              <label class="check">
                <input type="checkbox" {value} bind:group={granted}/>
                {LABELS_PERMISSION[value]}
              </label>
            {/each}
          </fieldset>
        {/if}

        <button class="full-row" type="submit" disabled={settings.isPending()}>Save</button>
      </form>
    {/if}

    {#if can_delete}
      <span class="divider-horizontal full-row"></span>
      <button class="full-row is-danger" onclick={deleteRoom} disabled={settings.isPending()}>Delete Room</button>
    {/if}
  </div>
{/if}

{#if settings.getError()}
  <span class="notify">{MSG_ERROR}: {settings.getError()}</span>
{/if}
