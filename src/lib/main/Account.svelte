<script>
    import { onMount, createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { activeSkin, getSkinHeadDataUrl } from "../skins/skin_utils.js";
    import AccountSwitcherMenu from "./AccountSwitcherMenu.svelte";

    export let username;
    export let accountType;
    export let options;

    const dispatch = createEventDispatcher();
    let avatarUrl = "img/steve.png";
    let menuOpen = false;

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

    function toggleMenu() {
        menuOpen = !menuOpen;
    }
</script>

<div class="account-dropdown-wrapper">
    <div class="account">
        <div
            class="avatar-btn"
            role="button"
            tabindex="0"
            title="Switch Account"
            on:click={toggleMenu}
            on:keydown={(e) => e.key === "Enter" && toggleMenu()}
        >
            <img src={avatarUrl} alt="avatar" class="avatar"/>
            <div class="switch-badge" title="Account Switcher">
                <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                    <polyline points="6 9 12 15 18 9"/>
                </svg>
            </div>
        </div>
        <div
            class="details"
            role="button"
            tabindex="0"
            title="Switch Account"
            on:click={toggleMenu}
            on:keydown={(e) => e.key === "Enter" && toggleMenu()}
        >
            <div class="username-row">
                <span class="username">{username}</span>
                <span class="caret">▾</span>
            </div>
            <div class="type">{accountType === "Offline" ? "Offline" : accountType} &bull; {$activeSkin?.name || "Skin"}</div>
        </div>
        <button class="button-settings" type="button" title="Settings" on:click={() => dispatch("showSettings")} >
            <img class="icon" src="img/icon/icon-cog.svg" alt="settings">
        </button>
    </div>

    <AccountSwitcherMenu
        bind:options
        bind:open={menuOpen}
        on:accountSwitched={(e) => dispatch("accountSwitched", e.detail)}
        on:accountSignedOut={() => dispatch("accountSignedOut")}
    />
</div>

<style>
    .account-dropdown-wrapper {
        position: relative;
    }

    .account {
        display: grid;
        grid-template-columns: max-content auto max-content;
        background: rgba(15, 23, 42, 0.65);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        height: 52px;
        border-radius: 10px;
        padding: 8px 14px;
        gap: 10px;
        align-items: center;
        min-width: 210px;
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05);
        transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .account:hover {
        border-color: rgba(99, 102, 241, 0.45);
        background: rgba(30, 41, 59, 0.8);
        transform: translateY(-1px);
        box-shadow: 0 8px 24px rgba(59, 130, 246, 0.2), inset 0 1px 0 rgba(255, 255, 255, 0.1);
    }

    .avatar-btn {
        position: relative;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .avatar {
        border-radius: 8px;
        height: 36px;
        width: 36px;
        overflow: hidden;
        background-color: rgba(255, 255, 255, .1);
        image-rendering: pixelated;
        border: 1px solid rgba(255, 255, 255, 0.15);
        transition: transform 0.2s ease, border-color 0.2s ease;
    }

    .avatar-btn:hover .avatar {
        transform: scale(1.08);
        border-color: #3b82f6;
    }

    .switch-badge {
        position: absolute;
        bottom: -2px;
        right: -2px;
        width: 14px;
        height: 14px;
        border-radius: 50%;
        background: linear-gradient(135deg, #3b82f6 0%, #6366f1 100%);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid #0f172a;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
    }

    .details {
        font-size: 11px;
        cursor: pointer;
    }

    .username-row {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .details .username {
        color: white;
        font-weight: 600;
    }

    .caret {
        color: #9ca3af;
        font-size: 10px;
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
