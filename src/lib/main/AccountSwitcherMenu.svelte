<script>
    import {createEventDispatcher} from "svelte";
    import {invoke} from "@tauri-apps/api/core";
    import ModalInput from "../login/loginmodal/ModalInput.svelte";
    import {activeSkin, getSkinHeadDataUrl} from "../skins/skin_utils.js";

    export let options;
    export let open = false;

    const dispatch = createEventDispatcher();

    let showAddModal = false;
    let newUsername = "";
    let addError = null;
    let adding = false;

    let showChangeModal = false;
    let changeUsername = "";
    let changeError = null;
    let changing = false;

    let avatarMap = {};

    $: currentAccount = options?.start?.account ?? null;
    $: currentUsername = currentAccount?.name ?? "";
    $: accountsList = getAccountsList(options);

    function getAccountsList(opts) {
        let list = opts?.start?.accounts || [];
        if (opts?.start?.account) {
            const hasCurrent = list.some(a => a.name?.toLowerCase() === opts.start.account.name?.toLowerCase());
            if (!hasCurrent) {
                list = [opts.start.account, ...list];
            }
        }
        return list;
    }

    async function updateAvatars() {
        if ($activeSkin?.dataUrl) {
            const head = await getSkinHeadDataUrl($activeSkin.dataUrl, 48);
            if (currentUsername) {
                avatarMap[currentUsername] = head;
            }
        }
    }

    $: if ($activeSkin || currentUsername) {
        updateAvatars();
    }

    async function switchAccount(acc) {
        try {
            options.start.account = acc;
            // update accounts array in options
            let list = options.start.accounts || [];
            if (!list.some(a => a.name?.toLowerCase() === acc.name?.toLowerCase())) {
                list = [acc, ...list];
                options.start.accounts = list;
            }
            await options.store();
            open = false;
            dispatch("accountSwitched", acc);
        } catch (e) {
            console.error("Failed to switch account:", e);
        }
    }

    async function handleAddAccount() {
        if (!newUsername.trim()) return;
        adding = true;
        addError = null;
        try {
            const created = await invoke("login_offline", {username: newUsername.trim()});
            let list = options.start.accounts || [];
            if (!list.some(a => a.name?.toLowerCase() === created.name?.toLowerCase())) {
                list = [...list, created];
                options.start.accounts = list;
            }
            options.start.account = created;
            await options.store();
            newUsername = "";
            showAddModal = false;
            open = false;
            dispatch("accountSwitched", created);
        } catch (e) {
            addError = `${e}`;
        } finally {
            adding = false;
        }
    }

    async function handleChangeUsername() {
        if (!changeUsername.trim()) return;
        changing = true;
        changeError = null;
        try {
            const created = await invoke("login_offline", {username: changeUsername.trim()});
            let list = options.start.accounts || [];
            // Remove previous username if found
            list = list.filter(a => a.name?.toLowerCase() !== currentUsername.toLowerCase());
            list = [created, ...list];
            options.start.accounts = list;
            options.start.account = created;
            await options.store();
            changeUsername = "";
            showChangeModal = false;
            open = false;
            dispatch("accountSwitched", created);
        } catch (e) {
            changeError = `${e}`;
        } finally {
            changing = false;
        }
    }

    async function handleSignOut() {
        try {
            await invoke("logout");
            options.start.account = null;
            await options.store();
            open = false;
            dispatch("accountSignedOut");
        } catch (e) {
            console.error("Sign out failed:", e);
        }
    }

    function handleMicrosoftClick() {
        alert("Microsoft Account login requires online Mojang authentication. This de-bloated offline client uses local and cracked accounts.");
    }
</script>

<svelte:window on:click={(e) => {
    if (open && !e.target.closest('.account-dropdown-wrapper')) {
        open = false;
    }
}} />

{#if open}
    <div class="account-menu" role="menu" tabindex="-1">
        <!-- Accounts List Header & Items -->
        <div class="accounts-list">
            {#each accountsList as acc (acc.name)}
                {@const isSelected = currentUsername.toLowerCase() === acc.name.toLowerCase()}
                <div
                    class="account-item"
                    class:selected={isSelected}
                    role="menuitem"
                    tabindex="0"
                    on:click={() => switchAccount(acc)}
                    on:keydown={(e) => (e.key === "Enter" || e.key === " ") && switchAccount(acc)}
                >
                    <div class="avatar-box">
                        <img src={avatarMap[acc.name] || "img/steve.png"} alt="avatar" class="mini-avatar" />
                    </div>
                    <div class="acc-text">
                        <span class="acc-name">{acc.name}</span>
                        <span class="acc-type">Offline</span>
                    </div>
                    {#if isSelected}
                        <svg class="check-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                            <polyline points="20 6 9 17 4 12"/>
                        </svg>
                    {/if}
                </div>
            {/each}
        </div>

        <div class="divider"></div>

        <!-- Menu Actions matching the reference screenshot -->
        <div class="menu-actions">
            <button type="button" class="action-btn" on:click={handleMicrosoftClick}>
                Add Microsoft Account...
            </button>
            <button type="button" class="action-btn" on:click={() => { showAddModal = true; newUsername = ""; addError = null; }}>
                Add Offline Account...
            </button>
            <button type="button" class="action-btn" on:click={() => { showChangeModal = true; changeUsername = currentUsername; changeError = null; }}>
                Change Username...
            </button>
            <button type="button" class="action-btn sign-out" on:click={handleSignOut}>
                Sign Out
            </button>
        </div>
    </div>
{/if}

<!-- Add Offline Account Modal -->
{#if showAddModal}
    <div
        class="modal-backdrop"
        on:click={(e) => e.target === e.currentTarget && (showAddModal = false)}
        on:keydown={(e) => e.key === 'Escape' && (showAddModal = false)}
        role="presentation"
    >
        <div class="modal-card" role="dialog" aria-modal="true">
            <div class="modal-title">Add Offline Account</div>
            <ModalInput
                icon="person"
                placeholder="Username"
                characterLimit={16}
                bind:value={newUsername}
            />
            {#if addError}
                <div class="error-text">{addError}</div>
            {/if}
            <div class="modal-buttons">
                <button type="button" class="btn-cancel" on:click={() => showAddModal = false}>Cancel</button>
                <button type="button" class="btn-confirm" disabled={adding || !newUsername.trim()} on:click={handleAddAccount}>
                    {adding ? "Adding..." : "Add Account"}
                </button>
            </div>
        </div>
    </div>
{/if}

<!-- Change Username Modal -->
{#if showChangeModal}
    <div
        class="modal-backdrop"
        on:click={(e) => e.target === e.currentTarget && (showChangeModal = false)}
        on:keydown={(e) => e.key === 'Escape' && (showChangeModal = false)}
        role="presentation"
    >
        <div class="modal-card" role="dialog" aria-modal="true">
            <div class="modal-title">Change Username</div>
            <ModalInput
                icon="person"
                placeholder="New Username"
                characterLimit={16}
                bind:value={changeUsername}
            />
            {#if changeError}
                <div class="error-text">{changeError}</div>
            {/if}
            <div class="modal-buttons">
                <button type="button" class="btn-cancel" on:click={() => showChangeModal = false}>Cancel</button>
                <button type="button" class="btn-confirm" disabled={changing || !changeUsername.trim()} on:click={handleChangeUsername}>
                    {changing ? "Saving..." : "Change Username"}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    .account-menu {
        position: absolute;
        top: 100%;
        right: 0;
        margin-top: 8px;
        width: 250px;
        background: #1c1e24;
        border: 1px solid #2e313a;
        border-radius: 8px;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.65);
        z-index: 1000;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        animation: fadeIn 0.15s ease-out;
        user-select: none;
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(-4px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .accounts-list {
        display: flex;
        flex-direction: column;
        max-height: 160px;
        overflow-y: auto;
    }

    .account-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 14px;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .account-item:hover {
        background: #252830;
    }

    .account-item.selected {
        background: #2a2e38;
    }

    .avatar-box {
        width: 28px;
        height: 28px;
        border-radius: 5px;
        background: #121316;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }

    .mini-avatar {
        width: 24px;
        height: 24px;
        image-rendering: pixelated;
    }

    .acc-text {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    .acc-name {
        font-size: 13px;
        font-weight: 600;
        color: #ffffff;
    }

    .acc-type {
        font-size: 11px;
        color: #9ca3af;
    }

    .check-icon {
        color: #ffffff;
    }

    .divider {
        height: 1px;
        background: #2e313a;
        margin: 2px 0;
    }

    .menu-actions {
        display: flex;
        flex-direction: column;
        padding: 4px 0;
    }

    .action-btn {
        background: transparent;
        border: none;
        color: #d1d5db;
        padding: 9px 14px;
        text-align: left;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: background 0.15s ease, color 0.15s ease;
    }

    .action-btn:hover {
        background: #252830;
        color: #ffffff;
    }

    .action-btn.sign-out {
        color: #f87171;
    }

    .action-btn.sign-out:hover {
        background: rgba(239, 68, 68, 0.15);
        color: #ef4444;
    }

    /* Modal Backdrop and Card */
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.65);
        backdrop-filter: blur(4px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2000;
    }

    .modal-card {
        background: #1e2026;
        border: 1px solid #323642;
        border-radius: 10px;
        padding: 22px;
        width: 320px;
        display: flex;
        flex-direction: column;
        gap: 16px;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
    }

    .modal-title {
        font-size: 16px;
        font-weight: 600;
        color: #ffffff;
    }

    .error-text {
        font-size: 12px;
        color: #fca5a5;
    }

    .modal-buttons {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        margin-top: 6px;
    }

    .btn-cancel {
        background: transparent;
        border: 1px solid #373b47;
        color: #9ca3af;
        border-radius: 6px;
        padding: 6px 12px;
        font-size: 12px;
        cursor: pointer;
    }

    .btn-cancel:hover {
        color: #ffffff;
        border-color: #4b5563;
    }

    .btn-confirm {
        background: #2563eb;
        border: none;
        color: #ffffff;
        border-radius: 6px;
        padding: 6px 14px;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
    }

    .btn-confirm:hover:not(:disabled) {
        background: #1d4ed8;
    }

    .btn-confirm:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
