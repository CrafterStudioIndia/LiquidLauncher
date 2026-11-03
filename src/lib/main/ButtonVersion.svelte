<script>
    import { createEventDispatcher } from "svelte";

    export let icon;
    export let title;
    export let subtitle;

    const dispatch = createEventDispatcher();
</script>

<button class="button-version" on:click={e => dispatch("click", e)}>
    <img class="icon" src="img/icon/icon-version-{icon}.png" alt={title}>
    <div class="title">{title}</div>
    <div class="subtitle">{subtitle}</div>
</button>

<style>
    .button-version {
        background: rgba(15, 23, 42, 0.65);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border-radius: 10px;
        text-align: center;
        border: 1px solid rgba(255, 255, 255, 0.08);
        padding: 12px 10px;
        transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
        cursor: pointer;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05);
    }

    .button-version:hover {
        border-color: rgba(99, 102, 241, 0.5);
        transform: translateY(-2px);
        background: rgba(30, 41, 59, 0.8);
        box-shadow: 0 8px 20px rgba(59, 130, 246, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.1);
    }

    .button-version:active {
        transform: translateY(0);
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
    }

    .button-version .icon {
        margin-bottom: 6px;
        transition: transform 0.2s;
    }

    .button-version:hover .icon {
        transform: scale(1.08);
    }

    .button-version .title {
        color: white;
        font-size: 12px;
        font-weight: 600;
        margin-bottom: 2px;
    }

    .button-version .subtitle {
        font-size: 10px;
        color: rgba(255, 255, 255, 0.6);
        font-weight: 500;
    }
</style>