<script>
    import {invoke} from "@tauri-apps/api/core";
    import {onMount} from "svelte";
    import MainScreen from "./main/MainScreen.svelte";
    import LoginScreen from "./login/LoginScreen.svelte";
    import LoadingScreen from "./main/LoadingScreen.svelte";
    import ErrorScreen from "./main/ErrorScreen.svelte";
    import {applyAppearance, backgroundImage, tint} from "./appearance.js";

    let loading = true;
    let error = null;
    let options = null;

    async function setupOptions() {
        try {
            options = {
                store: async function () {
                    console.debug("Storing options...", options);
                    try {
                        await invoke("store_options", {options});
                    } catch (error) {
                        console.error("Failed to store options:", error);
                        throw error;
                    }
                },
                ...await invoke("get_options")
            };
            console.debug("Options loaded:", options);
        } catch (e) {
            console.error("Failed to load options:", e);

            error = {
                message: "Failed to load launcher options",
                error: e
            };
        }
    }

    async function loadBackground() {
        try {
            backgroundImage.set(await invoke("get_background"));
        } catch (e) {
            // A background is cosmetic; a broken one must not stop the launcher.
            console.error("Failed to load the background:", e);
        }
    }

    async function checkSystem() {
        try {
            await invoke("check_system");
        } catch (e) {
            // We want to continue allowing the user to use the launcher even
            // if the system check fails
            alert("Looks like there is a configuration issue with your system.\n\n" + e);
        }
    }

    onMount(async () => {
        await setupOptions();
        if (options) {
            applyAppearance(options);
            await loadBackground();
        }
        await checkSystem();
        loading = false;
    });
</script>

<div class="window" class:has-background={!!$backgroundImage.dataUrl}>
    {#if $backgroundImage.dataUrl}
        <div
                class="background"
                style="background-image: url('{$backgroundImage.dataUrl}'); filter: blur({$tint.blur}px);"
        ></div>
        <div
                class="tint"
                style="background-color: {$tint.color}; opacity: {$tint.opacity};"
        ></div>
    {/if}

    <div class="drag-area" data-tauri-drag-region></div>

    <div class="content">
        {#if error}
            <ErrorScreen {error} />
        {:else if loading}
            <LoadingScreen />
        {:else if options}
            {#if options.start.account}
                <MainScreen bind:options />
            {:else}
                <LoginScreen bind:options />
            {/if}
        {/if}
    </div>
</div>

<style>
    .window {
        background-color: rgba(0, 0, 0, 0.6);
        width: 100vw;
        height: 100vh;
        padding: 32px;
        overflow: hidden;
        position: relative;
    }

    /* The tint over the picture already darkens the window. */
    .window.has-background {
        background-color: transparent;
    }

    .background {
        position: absolute;
        inset: -40px;
        background-size: cover;
        background-position: center;
        z-index: 0;
    }

    .tint {
        position: absolute;
        inset: 0;
        z-index: 1;
    }

    .content {
        position: relative;
        z-index: 2;
        height: 100%;
    }

    .drag-area {
        position: fixed;
        top: 0;
        left: 0;
        width: calc(100vw - 150px);
        height: 100px;
        z-index: 100;
    }

    @media (prefers-color-scheme: light) {
        .window {
            background-color: rgba(0, 0, 0, 0.8);
        }
    }
</style>
