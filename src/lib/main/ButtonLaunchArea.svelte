<script>
    import { createEventDispatcher } from "svelte";


    export let text;
    export let active;
    export let disabled = false;

    const dispatch = createEventDispatcher();
</script>

<button
        class="button"
        class:active
        type="button"
        disabled={disabled}
        on:click={e => dispatch("click", e)}
>
    {text}
</button>

<style>
    .button {
        background: unset;
        border: 1px solid rgba(255, 255, 255, 0.18);
        background: linear-gradient(135deg, #3b82f6 0%, #6366f1 50%, #8b5cf6 100%);
        color: white;
        font-size: 14px;
        border-radius: 10px;
        display: block;
        padding: 16px;
        transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
        font-family: "Inter", sans-serif;
        cursor: pointer;
        width: 100%;
        font-weight: 700;
        letter-spacing: 0.02em;
        box-shadow: 0 4px 20px rgba(99, 102, 241, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.25);
    }

    .button:hover {
        background: linear-gradient(135deg, #2563eb 0%, #4f46e5 50%, #7c3aed 100%);
        transform: translateY(-2px);
        box-shadow: 0 8px 30px rgba(99, 102, 241, 0.65), inset 0 1px 0 rgba(255, 255, 255, 0.35);
        filter: brightness(1.08);
    }

    .button:active {
        transform: translateY(0);
        box-shadow: 0 4px 14px rgba(99, 102, 241, 0.4);
    }

    .button.active {
        background: linear-gradient(135deg, #ef4444 0%, #dc2626 50%, #991b1b 100%);
        box-shadow: 0 4px 20px rgba(239, 68, 68, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.25);
    }

    .button.active:hover {
        background: linear-gradient(135deg, #dc2626 0%, #b91c1c 50%, #7f1d1d 100%);
        box-shadow: 0 8px 30px rgba(239, 68, 68, 0.65);
    }

    .button:disabled {
        opacity: 0.55;
        cursor: not-allowed;
        transform: none;
        box-shadow: none;
    }
</style>
