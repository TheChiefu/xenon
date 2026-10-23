<script lang="ts">
  // Modal window showing a user's public profile, for the user ID stored by the viewer

  import type { UserProfileResponse } from "@/bindings/routes/users";
  import ProfileCard from "./card.svelte";
  import * as usersAPI from "@/lib/api/users";
  import * as users from "@/lib/users/data.svelte";
  import * as profile_viewer from "@/lib/users/profile_viewer.svelte";
  import { dialogFly } from "@/lib/display/transitions";
  import { failureText } from "@/lib/utils";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);

  let profile: UserProfileResponse | null = $state(null);
  let failure: string | null = $state(null);

  const userId: string | null = $derived(profile_viewer.getUserId());

  $effect(() => {
    dialog?.showModal();
  });

  $effect(() => {
    if (userId === null) {
      return;
    }

    load(userId);
  });

  async function load(id: string): Promise<void> {
    try {
      profile = await usersAPI.get(id);
      failure = null;

      await users.load([id]);
    } catch (error) {
      failure = failureText(error);
    }
  }

  function cancel(event: Event) {
    event.preventDefault();
    onClose();
  }

  function clickAway(event: MouseEvent) {
    if (event.target !== dialog || dialog === null) {
      return;
    }

    const rect = dialog.getBoundingClientRect();
    const inside = event.clientX >= rect.left && event.clientX <= rect.right
      && event.clientY >= rect.top && event.clientY <= rect.bottom;

    if (!inside) {
      onClose();
    }
  }

  function formatDate(time: number): string {
    return new Date(time).toLocaleDateString();
  }
</script>

<style>
  dialog {
    padding: var(--dialog-padding);
    border: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
  }
</style>

<dialog bind:this={dialog} onclose={onClose} oncancel={cancel} onclick={clickAway} transition:dialogFly={{ y: "1rem" }}>

  {#if failure !== null}
    <p>{failure}</p>
  {:else if profile !== null}
    {@const presence = users.getPresence(profile.id)}
    <ProfileCard
      id={profile.id}
      display_name={profile.display_name}
      username={profile.username}
      description={profile.description}
      status={presence.status}
      device={presence.device}
      game={users.getActivity(profile.id) ?? null}
      role={profile.global_role}
      links={profile.linked_accounts}
      created_at={formatDate(profile.created_at)}
      avatar={users.getAvatar(profile.id) ?? null}
      banner={users.getBanner(profile.id) ?? null}
    />
  {/if}
</dialog>
