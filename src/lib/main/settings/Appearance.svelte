<script>
    import {invoke} from "@tauri-apps/api/core";
    import {open as dialogOpen} from "@tauri-apps/plugin-dialog";
    import {backgroundImage, DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_OPACITY, applyAppearance, tint} from "../../appearance.js";
    import Description from "../../settings/Description.svelte";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";

    export let options;

    let busy = false;

    $: appearance = options.appearance;

    async function choose() {
        if (busy) return;
        const selected = await dialogOpen({
            directory: false,
            multiple: false,
            filters: [{name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"]}],
            title: "Choose a launcher background"
        });
        if (!selected) return;

        busy = true;
        try {
            backgroundImage.set(await invoke("choose_background", {path: selected}));
        } catch (e) {
            console.error("Failed to set the background:", e);
            alert(`${e}`);
        } finally {
            busy = false;
        }
    }

    async function clear() {
        try {
            backgroundImage.set(await invoke("clear_background"));
        } catch (e) {
            console.error("Failed to clear the background:", e);
            alert(`${e}`);
        }
    }

    function save() {
        appearance.backgroundOpacity = Number(appearance.backgroundOpacity);
        appearance.backgroundBlur = Number(appearance.backgroundBlur);
        tint.set({
            color: appearance.backgroundColor,
            opacity: appearance.backgroundOpacity,
            blur: appearance.backgroundBlur
        });
        options.store();
    }

    async function reset() {
        appearance.backgroundColor = DEFAULT_BACKGROUND_COLOR;
        appearance.backgroundOpacity = DEFAULT_BACKGROUND_OPACITY;
        appearance.backgroundBlur = 0;
        await options.store();
        applyAppearance(options);
    }
</script>

<Description description="Use your own picture behind the launcher and tint it to match. The picture is copied into the launcher's config folder, so moving or deleting the original does not break it." />

<div class="preview" style={$backgroundImage.dataUrl ? `background-image: url('${$backgroundImage.dataUrl}')` : ""}>
    {#if !$backgroundImage.dataUrl}
        <span>No background picture</span>
    {/if}
</div>

<div class="buttons">
    <ButtonSetting text={busy ? "Choosing..." : "Choose picture"} disabled={busy} on:click={choose} />
    {#if $backgroundImage.dataUrl}
        <ButtonSetting text="Remove picture" color="#5b5b5b" on:click={clear} />
    {/if}
</div>

<div class="colour">
    <div class="label">Tint colour</div>
    <input class="picker" type="color" bind:value={appearance.backgroundColor} on:input={save} />
</div>

<div class="slider-setting">
    <div class="row">
        <span>Tint opacity</span>
        <span>{Math.round(appearance.backgroundOpacity * 100)}%</span>
    </div>
    <input class="slider" type="range" min="0" max="1" step="0.05" bind:value={appearance.backgroundOpacity} on:input={save} />
</div>

<div class="slider-setting">
    <div class="row">
        <span>Blur</span>
        <span>{appearance.backgroundBlur} px</span>
    </div>
    <input class="slider" type="range" min="0" max="30" step="1" bind:value={appearance.backgroundBlur} on:input={save} />
</div>

<ButtonSetting text="Reset appearance" color="#5b5b5b" on:click={reset} />

<style>
    .preview {
        height: 120px;
        border-radius: 6px;
        background-color: rgba(0, 0, 0, .26);
        background-size: cover;
        background-position: center;
        display: flex;
        align-items: center;
        justify-content: center;
        color: rgba(255, 255, 255, .5);
        font-size: 12px;
    }

    .buttons {
        display: flex;
        gap: 10px;
    }

    .colour {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .label {
        color: white;
    }

    .picker {
        width: 60px;
        height: 30px;
        border: none;
        background: transparent;
        cursor: pointer;
    }

    .slider-setting {
        display: flex;
        flex-direction: column;
        row-gap: 6px;
    }

    .row {
        display: flex;
        justify-content: space-between;
        color: white;
    }

    .row > span:last-child {
        color: rgba(255, 255, 255, .5);
    }

    .slider {
        width: 100%;
        height: 6px;
        border-radius: 3px;
        background-color: rgba(255, 255, 255, .2);
        cursor: pointer;
    }

    .slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background-color: #4677FF;
    }
</style>
