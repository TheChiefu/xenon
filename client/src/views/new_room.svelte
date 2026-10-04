<script lang="ts">
  import { Permission, Visibility } from "@/bindings/types";
  import { DESCRIPTIONS_VISIBILITY, LABELS_PERMISSION, LABELS_VISIBILITY } from "@/lib/labels";
  import { dialogFly } from "@/lib/transitions";
  import { getError, isPending, submit } from "@/lib/rooms/creation.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);

  // Room Properties
  let name = $state("");
  let visibility = $state<Visibility>(Visibility.Public);
  let claim_all = $state(true);
  let granted = $state<Permission[]>([Permission.Post, Permission.Attach, Permission.Invite]);

  $effect(() => {
    dialog?.showModal();
  });

  function cancel(event: Event) {
    event.preventDefault();
    onClose();
  }

  function create(event: SubmitEvent) {
    event.preventDefault();
    submit({ name, visibility, default_permissions: granted, claim_all });
  }
</script>

<style>
  dialog {
    width: 24rem;
    padding: 1.5rem;
    border: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
  }

  dialog::backdrop {
    background: var(--modal-background)
  }

  .header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1.0rem;
    border-bottom: 1px solid var(--border);
  }

  form {
    display: grid;
    gap: 0.75rem;
  }

  .field {
    display: grid;
    gap: 0.3rem;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  fieldset {
    display: grid;
    gap: 0.3rem;
    border: 1px solid var(--border);
  }

  .row {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
  }
</style>

<dialog bind:this={dialog} onclose={onClose} oncancel={cancel} transition:dialogFly={{ y: "1rem" }}>
  <div class="header">
    <h2>New Room</h2>
    <button class="inherit" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <form onsubmit={create}>
    <div class="field">
      <label for="room-name">Name</label>
      <input id="room-name" bind:value={name} required/>
    </div>

    <div class="field">
      <label for="room-visibility">Visibility</label>
      <select id="room-visibility" bind:value={visibility}>
        {#each Object.values(Visibility) as value}
          <option {value} title={DESCRIPTIONS_VISIBILITY[value]}>{LABELS_VISIBILITY[value]}</option>
        {/each}
      </select>
    </div>

    <label class="check" title="Unchecked, you join on the permissions below">
      <input type="checkbox" bind:checked={claim_all}/>
      Take every permission for myself
    </label>

    <fieldset>
      <legend>Members join with</legend>
      {#each Object.values(Permission) as value}
        <label class="check">
          <input type="checkbox" {value} bind:group={granted}/>
          {LABELS_PERMISSION[value]}
        </label>
      {/each}
    </fieldset>

    {#if getError()}
      <p>Error: {getError()}</p>
    {/if}

    <div class="row">
      <button type="button" onclick={onClose}>Cancel</button>
      <button class="is-action" type="submit" disabled={isPending()}>Create</button>
    </div>
  </form>
</dialog>
