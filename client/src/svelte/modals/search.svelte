<script lang="ts">
  import type { SearchResult } from "@/bindings/routes/search";
  import type { SearchRequest } from "@/bindings/shared";
  import * as searchAPI from "@/lib/api/search";
  import { dialogFly } from "@/lib/display/transitions";
  import { modal } from "@/lib/display/modal";
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

  // Filter inputs, blank means not used
  let msg: string = $state("");
  let file: string = $state("");
  let room: string = $state("");
  let author: string = $state("");
  let fromDay: string = $state("");
  let toDay: string = $state("");

  let active: SearchRequest | null = $state(null);
  let results: SearchResult[] = $state([]);
  let total: number = $state(0);
  let current: number = $state(1);
  let searched: boolean = $state(false);
  let pending: boolean = $state(false);
  let failure: string | null = $state(null);
  let showResults: boolean = $state(false);

  const roomChoices = $derived(rooms.list().toSorted((a, b) => a.name.localeCompare(b.name)));
  const pageCount = $derived(Math.ceil(total / (searchAPI.pageSize() ?? 1)));

  // Local midnight of a picked day, moved by a number of days
  function dayStart(day: string, offset: number): number {
    const date = new Date(`${day}T00:00`);
    date.setDate(date.getDate() + offset);
    return date.getTime();
  }

  // Starts a new search in place of the current one
  async function search(event: SubmitEvent): Promise<void> {
    event.preventDefault();

    active = {
      text: msg.trim() || null,
      file: file.trim() || null,
      room: room || null,
      author: author.trim() || null,
      from: fromDay ? dayStart(fromDay, 0) : null,
      to: toDay ? dayStart(toDay, 1) : null, // Next midnight, includes whole day
      page: 1,
    };

    results = [];
    total = 0;
    searched = false;
    showResults = true;
    await loadPage(1);
  }

  // Shows one page of the current search
  async function loadPage(page: number): Promise<void> {
    if (active === null) {
      return;
    }

    const request = active;
    pending = true;
    failure = null;

    try {
      const response = await searchAPI.messages({ ...request, page });

      // Drop page from a replaced search
      if (request !== active) {
        return;
      }

      // Load authors not yet known
      const authors: (string | null)[] = [];
      for (const result of response.results) {
        authors.push(result.author_id);
      }
      await users.load(authors);

      results = response.results;
      total = response.total;
      current = page;
      searched = true;
    } catch (error) {
      failure = failureText(error);
    } finally {
      pending = false;
    }
  }
</script>

<style>
  dialog {
    width: 55rem;
    height: 36rem;
  }

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
    grid-template-columns: auto minmax(10rem, 1fr);
    align-content: start;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    min-height: 0;
  }

  .filters input, .filters select, .filters textarea {
    width: 0;
    min-width: 100%;
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

  .hidden {
    display: none;
  }
  
</style>

<dialog class="menu" {@attach modal(onClose)} onclose={onClose} transition:dialogFly={{ y: "1rem" }}>
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
        <textarea id="search-message" bind:value={msg} minlength="3"></textarea>

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

        {#if searched && total === 0}
          <span class="notify empty">No Results</span>
        {/if}

        {#each results as result (result.id)}
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

      {#if pageCount > 1}
        <div class="pages">
          {#each { length: pageCount }, index}
            <button class:is-active={index + 1 === current} onclick={() => loadPage(index + 1)} disabled={pending}>{index + 1}</button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</dialog>
