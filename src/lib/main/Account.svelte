<script>
    import { onMount, createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { activeSkin, getSkinHeadDataUrl } from "../skins/skin_utils.js";

    export let username;
    export let accountType;

    const dispatch = createEventDispatcher();
    let avatarUrl = "img/steve.png";

    async function updateAvatar(skin) {
        if (skin?.dataUrl) {
            avatarUrl = await getSkinHeadDataUrl(skin.dataUrl, 64);
        } else {
            avatarUrl = "img/steve.png";
        }
    }

    $: if ($activeSkin) {
        updateAvatar($activeSkin);
    }

    onMount(async () => {
        try {
            const active = await invoke("get_active_skin");
            if (active) {
                activeSkin.set(active);
                updateAvatar(active);
            }
        } catch (e) {
            console.error("Failed to fetch active skin:", e);
        }
    });
</script>

<div class="account">
    <div
        class="avatar-btn"
        role="button"
        tabindex="0"
        title="Open Skin Switcher"
        on:click={() => dispatch("showSkins")}
        on:keydown={(e) => e.key === "Enter" && dispatch("showSkins")}
    >
        <img src={avatarUrl} alt="avatar" class="avatar"/>
        <div class="skin-badge" title="Skin Switcher">
            <svg width="8" height="8" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2a5 5 0 0 0-5 5v1H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2h-2V7a5 5 0 0 0-5-5zm-3 5a3 3 0 0 1 6 0v1H9V7z"/>
            </svg>
        </div>
    </div>
    <div
        class="details"
        role="button"
        tabindex="0"
        on:click={() => dispatch("showSkins")}
        on:keydown={(e) => e.key === "Enter" && dispatch("showSkins")}
    >
        <div class="username">{username}</div>
        <div class="type">{accountType === "Offline" ? "Offline" : accountType} &bull; {$activeSkin?.name || "Skin"}</div>
    </div>
    <button class="button-settings" type="button" title="Settings" on:click={() => dispatch("showSettings")} >
        <img class="icon" src="img/icon/icon-cog.svg" alt="settings">
    </button>
</div>

<style>
    .account {
        display: grid;
        grid-template-columns: max-content auto max-content;
        background-color: rgba(0, 0, 0, .36);
        height: 50px;
        border-radius: 6px;
        padding: 8px 12px;
        gap: 8px;
        align-items: center;
        min-width: 200px;
    }

    .avatar-btn {
        position: relative;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .avatar {
        border-radius: 6px;
        height: 35px;
        width: 35px;
        overflow: hidden;
        background-color: rgba(255, 255, 255, .1);
        image-rendering: pixelated;
        border: 1px solid rgba(255, 255, 255, 0.15);
        transition: transform 0.15s ease, border-color 0.15s ease;
    }

    .avatar-btn:hover .avatar {
        transform: scale(1.05);
        border-color: #3b82f6;
    }

    .skin-badge {
        position: absolute;
        bottom: -2px;
        right: -2px;
        width: 13px;
        height: 13px;
        border-radius: 50%;
        background: #2563eb;
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid #111827;
    }

    .details {
        font-size: 11px;
        cursor: pointer;
    }

    .details .username {
        color: white;
        font-weight: 600;
    }

    .details .type {
        color: rgba(255, 255, 255, .5);
        font-weight: 500;
    }

    .details:hover .username {
        color: #93c5fd;
    }

    .button-settings {
        background-color: transparent;
        border: none;
        display: flex;
        align-items: center;
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        transition: background-color 0.15s ease;
    }

    .button-settings:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }
</style>
