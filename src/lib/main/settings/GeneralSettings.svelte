<script>
    import {createEventDispatcher} from "svelte";
    import SelectSetting from "../../settings/SelectSetting.svelte";
    import FileSelectorSetting from "../../settings/FileSelectorSetting.svelte";
    import DirectorySelectorSetting from "../../settings/DirectorySelectorSetting.svelte";
    import RangeSetting from "../../settings/RangeSetting.svelte";
    import ToggleSetting from "../../settings/ToggleSetting.svelte";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";
    import Description from "../../settings/Description.svelte";
    import LauncherVersion from "../../settings/LauncherVersion.svelte";
    import {onMount} from "svelte";
    import {invoke} from "@tauri-apps/api/core";

    export let options;

    const dispatch = createEventDispatcher();

    let launcherVersion = "";
    let defaultDataFolder = "";
    let systemMemory = options.start.memory;

    let showAddModal = false;
    let newAccountUsername = "";
    let addingAccount = false;
    let addAccountError = null;

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

    async function switchAccount(acc) {
        try {
            options.start.account = acc;
            await options.store();
            dispatch("refreshInfo");
        } catch (error) {
            console.error("Failed to switch account:", error);
        }
    }

    async function handleAddAccount() {
        if (!newAccountUsername.trim()) return;
        addingAccount = true;
        addAccountError = null;
        try {
            const created = await invoke("login_offline", {username: newAccountUsername.trim()});
            let list = options.start.accounts || [];
            if (!list.some(a => a.name?.toLowerCase() === created.name?.toLowerCase())) {
                list = [...list, created];
                options.start.accounts = list;
            }
            options.start.account = created;
            await options.store();
            newAccountUsername = "";
            showAddModal = false;
            dispatch("refreshInfo");
        } catch (e) {
            addAccountError = `${e}`;
        } finally {
            addingAccount = false;
        }
    }

    async function removeAccount(acc) {
        let list = options.start.accounts || [];
        list = list.filter(a => a.name?.toLowerCase() !== acc.name?.toLowerCase());
        options.start.accounts = list;
        if (options.start.account?.name?.toLowerCase() === acc.name?.toLowerCase()) {
            options.start.account = list[0] || null;
        }
        await options.store();
        dispatch("refreshInfo");
    }

    async function clearData() {
        try {
            await invoke("clear_data", {options});
            alert("Data cleared.");
            dispatch("refreshInfo");
        } catch (error) {
            console.error("Failed to clear data:", error);
            alert(`Failed to clear data: ${error}`);
        }
    }

    async function logout() {
        try {
            await invoke("logout");
            options.start.account = null;
            await options.store();
        } catch (error) {
            console.error("Logout failed:", error);
            alert("Failed to logout properly. Please try again.");
        }
    }

    async function useBundledJar() {
        options.start.clientJar = "";
        await options.store();
        dispatch("refreshInfo");
    }

    function resetJavaDistribution() {
        if (options.start.javaDistribution.type === "custom") {
            options.start.javaDistribution.value = "";
        } else if (options.start.javaDistribution.type === "manual") {
            options.start.javaDistribution.value = "temurin";
        }
    }

    onMount(async () => {
        const [version, folder, memory] = await Promise.all([
            invoke("get_launcher_version"),
            invoke("default_data_folder_path"),
            invoke("sys_memory"),
        ]);

        systemMemory = memory;
        launcherVersion = version;
        defaultDataFolder = folder;
    });
</script>

<SelectSetting
    title="JVM Distribution"
    items={[
        { value: "automatic", text: "Automatic" },
        { value: "manual", text: "Manual" },
        { value: "custom", text: "Custom" },
    ]}
    on:change={resetJavaDistribution}
    bind:value={options.start.javaDistribution.type}
/>

{#if options.start.javaDistribution.type === "manual"}
    <SelectSetting
        title="Distribution"
        items={[
            { value: "temurin", text: "Eclipse Temurin" },
            { value: "graalvm", text: "GraalVM" },
            { value: "zulu", text: "Azul Zulu" },
        ]}
        bind:value={options.start.javaDistribution.value}
    />
{/if}

{#if options.start.javaDistribution.type === "custom"}
    <FileSelectorSetting
        title="Custom JVM Path"
        placeholder="Select Java wrapper location"
        bind:value={options.start.javaDistribution.value}
        filters={[{ name: "java", extensions: [] }]}
        windowTitle="Select custom Java wrapper"
    />
{/if}

<DirectorySelectorSetting
    title="Data Location"
    placeholder={defaultDataFolder}
    bind:value={options.start.customDataPath}
    windowTitle="Select custom data directory"
/>

<RangeSetting
    title="Memory"
    min={2048}
    max={systemMemory}
    bind:value={options.start.memory}
    valueSuffix=" MB"
    step={128}
/>

<RangeSetting
    title="Concurrent Downloads"
    min={1}
    max={50}
    bind:value={options.launcher.concurrentDownloads}
    valueSuffix=" connections"
    step={1}
/>

<ToggleSetting
    title="Keep launcher running"
    disabled={false}
    bind:value={options.launcher.keepLauncherOpen}
/>

<Description description="LiquidBounce is launched from this jar. Leave it empty to use the jar bundled with the launcher." />

<FileSelectorSetting
        title="LiquidBounce jar"
        placeholder="Bundled jar"
        bind:value={options.start.clientJar}
        filters={[{ name: "Minecraft mod", extensions: ["jar"] }]}
        windowTitle="Select a LiquidBounce jar"
        on:change={() => dispatch("refreshInfo")}
/>

{#if options.start.clientJar}
    <ButtonSetting text="Use the bundled jar" color="#5b5b5b" on:click={useBundledJar} />
{/if}

<div class="account-switcher-section">
    <div class="section-header">
        <div class="section-title">Minecraft Accounts</div>
        <button type="button" class="btn-add-acc" on:click={() => { showAddModal = true; newAccountUsername = ""; addAccountError = null; }}>
            + Add Account
        </button>
    </div>

    <div class="accounts-container">
        {#each accountsList as acc (acc.name)}
            {@const isSelected = options.start.account?.name?.toLowerCase() === acc.name.toLowerCase()}
            <div class="acc-row" class:active={isSelected}>
                <div class="acc-info" role="button" tabindex="0" on:click={() => switchAccount(acc)} on:keydown={(e) => (e.key === "Enter" || e.key === " ") && switchAccount(acc)}>
                    <span class="acc-name">{acc.name}</span>
                    <span class="acc-badge">{isSelected ? "Active" : "Offline"}</span>
                </div>
                <div class="acc-actions">
                    {#if !isSelected}
                        <button type="button" class="btn-switch" on:click={() => switchAccount(acc)}>Switch</button>
                    {/if}
                    <button type="button" class="btn-remove" title="Remove account" on:click={() => removeAccount(acc)}>✕</button>
                </div>
            </div>
        {/each}
    </div>
</div>

{#if showAddModal}
    <div
        class="modal-backdrop"
        on:click={(e) => e.target === e.currentTarget && (showAddModal = false)}
        on:keydown={(e) => e.key === 'Escape' && (showAddModal = false)}
        role="presentation"
    >
        <div class="modal-card" role="dialog" aria-modal="true">
            <div class="modal-title">Add Offline Account</div>
            <input
                type="text"
                class="modal-input"
                placeholder="Username (1-16 chars)"
                maxlength="16"
                bind:value={newAccountUsername}
            />
            {#if addAccountError}
                <div class="error-text">{addAccountError}</div>
            {/if}
            <div class="modal-buttons">
                <button type="button" class="btn-cancel" on:click={() => showAddModal = false}>Cancel</button>
                <button type="button" class="btn-confirm" disabled={addingAccount || !newAccountUsername.trim()} on:click={handleAddAccount}>
                    {addingAccount ? "Adding..." : "Add Account"}
                </button>
            </div>
        </div>
    </div>
{/if}

<ButtonSetting
    text="Sign out of Minecraft Account"
    on:click={logout}
    color="#4677FF"
/>

<ButtonSetting text="Clear Data" on:click={clearData} color="#B83529" />

<LauncherVersion version={launcherVersion} />

<style>
    .account-switcher-section {
        background: rgba(0, 0, 0, 0.22);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        padding: 14px;
        margin: 10px 0;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .section-title {
        font-size: 14px;
        font-weight: 600;
        color: #ffffff;
    }

    .btn-add-acc {
        background: #2563eb;
        border: none;
        color: white;
        padding: 5px 10px;
        font-size: 12px;
        border-radius: 5px;
        font-weight: 500;
        cursor: pointer;
        transition: background 0.15s;
    }

    .btn-add-acc:hover {
        background: #1d4ed8;
    }

    .accounts-container {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .acc-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 6px;
        padding: 8px 12px;
        transition: all 0.15s;
    }

    .acc-row.active {
        border-color: #3b82f6;
        background: rgba(59, 130, 246, 0.12);
    }

    .acc-info {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        flex: 1;
    }

    .acc-name {
        font-size: 13px;
        font-weight: 600;
        color: #f3f4f6;
    }

    .acc-badge {
        font-size: 11px;
        color: #9ca3af;
        background: rgba(0, 0, 0, 0.35);
        padding: 2px 6px;
        border-radius: 4px;
    }

    .acc-row.active .acc-badge {
        color: #60a5fa;
        background: rgba(59, 130, 246, 0.2);
    }

    .acc-actions {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .btn-switch {
        background: #374151;
        border: none;
        color: #f3f4f6;
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 11px;
        cursor: pointer;
    }

    .btn-switch:hover {
        background: #4b5563;
    }

    .btn-remove {
        background: transparent;
        border: none;
        color: #9ca3af;
        font-size: 13px;
        cursor: pointer;
        padding: 4px;
    }

    .btn-remove:hover {
        color: #ef4444;
    }

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
        gap: 14px;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
    }

    .modal-title {
        font-size: 16px;
        font-weight: 600;
        color: #ffffff;
    }

    .modal-input {
        background: #121316;
        border: 1px solid #2e323b;
        color: #f3f4f6;
        padding: 8px 12px;
        border-radius: 6px;
        font-size: 13px;
        outline: none;
    }

    .modal-input:focus {
        border-color: #3b82f6;
    }

    .error-text {
        font-size: 12px;
        color: #fca5a5;
    }

    .modal-buttons {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
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
