<script lang="ts">
  import type { SearchResult } from "@/bindings/routes/search";
  import type { SearchRequest } from "@/bindings/shared";
  import * as searchAPI from "@/lib/api/search";
  import { dialogFly } from "@/lib/display/transitions";
  import { MSG_ERROR } from "@/lib/labels";
  import * as rooms from "@/lib/rooms/data.svelte";
  import * as users from "@/lib/users/data.svelte";
  import { failureText } from "@/lib/utils";
  import icon_x from "@/assets/icons/x.svg?raw";
  import icon_back from "@/assets/icons/back.svg?raw";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);

  // Filter inputs, blank means not used
  let msg: string = $state("");
  let file: string = $state("");
  let room: string = $state("");
  let author: string = $state("");
  let fromDay: string = $state("");
  let toDay: string = $state("");

  let active: SearchRequest | null = $state(null);
  let pages: SearchResult[][] = $state([]);
  let current: number = $state(0);
  let searched: boolean = $state(false);
  let more: boolean = $state(false);
  let pending: boolean = $state(false);
  let failure: string | null = $state(null);
  let showResults: boolean = $state(false);

  const roomChoices = $derived(rooms.list().toSorted((a, b) => a.name.localeCompare(b.name)));

  $effect(() => {
    dialog?.showModal();
  });

  function cancel(event: Event) {
    if (event.target === dialog) {
      event.preventDefault();
      onClose();
    }
  }

  function pressAway(event: PointerEvent) {
    if (event.target === dialog) {
      onClose();
    }
  }

  // Local midnight of a picked day, moved by a number of days
  function dayStart(day: string, offset: number): number {
    const date = new Date(`${day}T00:00`);
    date.setDate(date.getDate() + offset);
    return date.getTime();
  }

  // Starts a new search in place of the current one
  async function search(event: SubmitEvent): Promise<void> {
    event.preventDefault();

    // To includes its whole day so it sends the next midnight
    active = {
      text: msg.trim() || null,
      file: file.trim() || null,
      room: room || null,
      author: author.trim() || null,
      from: fromDay ? dayStart(fromDay, 0) : null,
      to: toDay ? dayStart(toDay, 1) : null,
      before: null,
    };

    pages = [];
    current = 0;
    searched = false;
    more = false;
    showResults = true;
    await loadMore();
  }

  // Adds the next batch of matches
  async function loadMore(): Promise<void> {
    if (active === null) {
      return;
    }

    const request = active;
    pending = true;
    failure = null;

    try {
      const batch = await searchAPI.messages({ ...request, before: pages.at(-1)?.at(-1)?.seq ?? null });

      // Drop batch from a replaced search
      if (request !== active) {
        return;
      }

      // Load authors not yet known
      const authors: string[] = [];
      for (const result of batch) {
        authors.push(result.author_id);
      }
      await users.load(authors);

      // Each batch is a page, shown once loaded
      if (batch.length > 0) {
        pages.push(batch);
        current = pages.length - 1;
      }

      // Full batch means more may exist
      searched = true;
      more = batch.length === searchAPI.pageSize();
    } catch (error) {
      failure = failureText(error);
    } finally {
      pending = false;
    }
  }
</script>

<style>
  .columns {
    display: flex;
    gap: var(--section-gap);
    flex: 1;
    min-height: 0;
  }

  .filters {
    display: flex;
    flex-direction: column;
    width: fit-content;
    flex: 1;
  }

  .filters form {
    border: none;
    display: grid;
    grid-template-columns: auto 1fr;
    align-content: start;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    min-height: 0;
  }

  .filters button {
    grid-column: 1 / -1;
  }

  .results {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  .end {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .notify {
    color: var(--text-dim);
    font-style: italic;
  }

  .empty {
    position: absolute;
    inset: 0;
    justify-content: center;
    border: none;
  }

  .table {
    position: relative;
    display: grid;
    grid-template-columns: 1fr 1fr 3fr auto;
    flex: 1;
    min-height: 0;
  }

  .message {
    flex-direction: column;
    align-items: flex-start;
    overflow-wrap: anywhere;
  }

  .files, .date {
    color: var(--text-dim);
  }

  .files {
    font-size: 0.85rem;
  }

  .pages {
    display: flex;
    gap: 0.25rem;
    margin-top: var(--section-gap);
    align-self: center;
  }

  @media (width >= 50rem) {
    .filters {
      flex: none;
    }

    .back {
      display: none;
    }
  }

  @media (width < 50rem) {
    .hidden {
      display: none;
    }
  }
</style>

<dialog class="menu" bind:this={dialog} onclose={onClose} oncancel={cancel} onpointerdown={pressAway} transition:dialogFly={{ y: "1rem" }}>
  <div class="header">
    <h2>Search Messages</h2>
    <button class="icon-btn" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <div class="columns">

    <!-- Filters -->
    <div class="filters" class:hidden={showResults}>
      <div class="grid-title">
        <h3>Filters</h3>
      </div>

      <form class="grid" onsubmit={search}>
        <label for="search-message">Message</label>
        <input id="search-message" bind:value={msg} minlength="3"/>

        <label for="search-file">File</label>
        <input id="search-file" bind:value={file} minlength="3"/>

        <label for="search-room">Room</label>
        <select id="search-room" bind:value={room}>
          <option value="">Any room</option>
          {#each roomChoices as choice (choice.id)}
            <option value={choice.name}>{choice.name}</option>
          {/each}
        </select>

        <label for="search-author">Author</label>
        <input id="search-author" placeholder="username" autocapitalize="none" spellcheck="false" bind:value={author}/>

        <label for="search-from">From</label>
        <input id="search-from" type="date" bind:value={fromDay}/>

        <label for="search-to">To</label>
        <input id="search-to" type="date" bind:value={toDay}/>

        <button class="is-action" type="submit" disabled={pending}>Search</button>
      </form>
    </div>

    <!-- Results -->
    <div class="results" class:hidden={!showResults}>
      <div class="grid-title">
        <h3>Results</h3>
        <span class="end">
          {#if failure !== null}
            <span class="notify">{MSG_ERROR}: {failure}</span>
          {/if}
          <button class="icon-btn back" aria-label="Back to filters" title="Back to filters" onclick={() => showResults = false}>
            <span class="icon" aria-hidden="true">{@html icon_back}</span>
          </button>
        </span>
      </div>

      <div class="grid table">
        <span class="grid-heading">Room</span>
        <span class="grid-heading">Author</span>
        <span class="grid-heading">Message</span>
        <span class="grid-heading">Date</span>

        {#if searched && pages.length === 0}
          <span class="notify empty">No Results</span>
        {/if}

        {#each pages[current] ?? [] as result (result.id)}
          <span>{rooms.get(result.room_id)?.name ?? ""}</span>
          <span>{users.getDisplayName(result.author_id)}</span>
          <span class="message">
            {result.body ?? ""}
            {#if result.files.length > 0}
              <span class="files">{result.files.join(", ")}</span>
            {/if}
          </span>
          <span class="date">{new Date(result.created_at).toLocaleDateString()}</span>
        {/each}
      </div>

      <div class="pages">
        {#each pages as _, index}
          <button class:is-active={index === current} onclick={() => current = index}>{index + 1}</button>
        {/each}

        {#if more}
          <button onclick={loadMore} disabled={pending}>Load More</button>
        {/if}
      </div>
    </div>
  </div>
</dialog>
