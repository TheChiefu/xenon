<script lang="ts">

    interface Props {
        message: string,
        onConfirm: () => void;
        onCancel: () => void;
    }
    let { message, onConfirm, onCancel }: Props = $props();
    let dialog = $state<HTMLDialogElement | null>(null);

    $effect(() => {
        dialog?.showModal();
    });

</script>

<style>
    .row {
        display: flex;
        width: 100%;
        gap: 1.0rem;
        align-items: center;
        justify-content: center;
    }

    p {
        margin: 0;
    }

</style>

<dialog bind:this={dialog}>
    <p>{message}</p>
    <hr/>
    <div class="row">
        {#if onConfirm != null}
        <button onclick={onConfirm}>Yes</button>
        {/if}
        {#if onCancel != null}
        <button onclick={onCancel}>No</button>
        {/if}
    </div>

</dialog>