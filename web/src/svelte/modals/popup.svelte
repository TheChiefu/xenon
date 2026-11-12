<script lang="ts">

    interface Props {
        message: string,
        choices: string[];
        onChoose: (choice: number) => void;
        onCancel: () => void;
    }
    let { message, choices, onChoose, onCancel }: Props = $props();
    let dialog = $state<HTMLDialogElement | null>(null);

    $effect(() => {
        dialog?.showModal();
    });

</script>

<style>
    dialog {
        max-width: 18rem;
        text-align: center;
    }

    .choices {
        display: flex;
        width: 100%;
        gap: var(--inline-gap);
        align-items: center;
        justify-content: center;
    }

    .choices.is-column {
        flex-direction: column;
        align-items: stretch;
    }

    p {
        margin: 0;
        padding: 0.1rem;
        white-space: pre-line;
    }

</style>

<dialog bind:this={dialog} oncancel={onCancel}>
    <p>{message}</p>
    <hr/>
    <div class="choices" class:is-column={choices.length > 2}>
        {#each choices as choice, index (index)}
        <button onclick={() => onChoose(index)}>{choice}</button>
        {/each}
    </div>

</dialog>