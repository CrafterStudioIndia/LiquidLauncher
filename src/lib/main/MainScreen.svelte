<!-- lib/main/MainScreen.svelte -->
<script>
    import {invoke} from "@tauri-apps/api/core";
    import {listen} from "@tauri-apps/api/event";
    import {confirm} from "@tauri-apps/plugin-dialog";
    import {onMount} from "svelte";
    import VerticalFlexWrapper from "../common/VerticalFlexWrapper.svelte";
    import MainHeader from "./MainHeader.svelte";
    import ContentWrapper from "./ContentWrapper.svelte";
    import LaunchArea from "./LaunchArea.svelte";
    import BuildInfo from "./BuildInfo.svelte";
    import ClientLog from "./log/ClientLog.svelte";
    import Settings from "./settings/Settings.svelte";
    import FirstRunWarning from "./FirstRunWarning.svelte";

    export let options;

    let running = false;

    let info = null;
    let infoError = null;
    let loadingInfo = false;
    let customMods = [];

    let logShown = false;
    let settingsShown = false;
    let settingsTab = "General";
    let firstRunWarningShown = false;
    let log = [];

    let progressState = {
        max: 0,
        value: 0,
        text: "",
        speed: 0
    };

    $: selectedInstance = info?.selected ?? null;

    async function refreshInfo() {
        loadingInfo = true;
        try {
            info = await invoke("get_launch_info", {options});
            infoError = null;

            if (info.instances.length > 0 && options.launcher.instance !== info.selected.id) {
                options.launcher.instance = info.selected.id;
                await options.store();
            }
        } catch (e) {
            console.error("Failed to read the local version:", e);
            infoError = `${e}`;
        } finally {
            loadingInfo = false;
        }
    }

    async function refreshMods(check = false) {
        try {
            customMods = await invoke("get_custom_mods", {options, check});
        } catch (e) {
            console.error("Failed to read the mods:", e);
        }
    }

    async function refresh() {
        await refreshInfo();
        await refreshMods(false);
    }

    async function selectInstance(id) {
        if (!id || id === options.launcher.instance) return;
        options.launcher.instance = id;
        await options.store();
        await refresh();
    }

    const WARNING_MEMORY = 4096;

    async function runClient() {
        if (options.launcher.firstRun) {
            firstRunWarningShown = true;
            return;
        }

        if (running || !info) return;

        log = [];

        try {
            running = true;
            progressState = {max: 0, value: 0, text: "Starting client...", speed: 0};

            await options.store();

            if (!await checkMemory()) {
                running = false;
                return;
            }

            await launchClient();
        } catch (e) {
            console.error("Failed to start client:", e);
            log = [...log, `Failed to start client: ${e}`];
            running = false;
            logShown = true;
        }
    }

    /// Asks before launching with less memory than the game comfortably needs.
    async function checkMemory() {
        if (options.start.memory >= WARNING_MEMORY) return true;

        return await confirm(
            `You are about to launch the client with less than ${WARNING_MEMORY} MB of memory. This may cause performance issues. Do you want to continue?`
        );
    }

    async function launchClient() {
        await options.store();
        await invoke("run_client", {
            options,
            mods: customMods.filter(mod => mod.required || mod.enabled)
        });
    }

    async function terminateClient() {
        await invoke("terminate");
    }

    function showSettings(tab) {
        settingsTab = tab;
        settingsShown = true;
    }

    async function continueAfterFirstRun() {
        await hideFirstRunWarning();
        await runClient();
    }

    async function hideFirstRunWarning() {
        firstRunWarningShown = false;
        if (options.launcher.firstRun) {
            options.launcher.firstRun = false;
            await options.store();
        }
    }

    listen("process-output", (event) => {
        log = [...log, event.payload];
    });

    listen("progress-update", (event) => {
        const {type, value} = event.payload;
        switch (type) {
            case "max":
                progressState.max = value;
                break;
            case "progress":
                progressState.value = value;
                break;
            case "label":
                progressState.text = value;
                break;
            case "speed":
                progressState.speed = value;
                break;
        }
    });

    listen("client-exited", () => {
        running = false;
        refresh();
    });

    listen("client-error", () => {
        logShown = true;
    });

    let jarPrompt = null;

    listen("jar-release-updated", (event) => {
        if (event.payload) {
            jarPrompt = event.payload;
        }
    });

    onMount(async () => {
        await refresh();
        try {
            const update = await invoke("check_jar_release_update", {options});
            if (update) {
                jarPrompt = update;
            }
        } catch (e) {
            console.warn("Release update check:", e);
        }
    });
</script>

{#if jarPrompt}
    <div
        class="jar-prompt-backdrop"
        role="presentation"
        on:click={(e) => e.target === e.currentTarget && (jarPrompt = null)}
    >
        <div class="jar-prompt-card" role="dialog" aria-modal="true">
            <div class="jar-prompt-header">
                <div class="jar-prompt-icon">
                    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                        <polyline points="7 10 12 15 17 10"/>
                        <line x1="12" y1="15" x2="12" y2="3"/>
                    </svg>
                </div>
                <div class="jar-prompt-title">LiquidBounce Updated</div>
            </div>
            <div class="jar-prompt-message">
                Fetched <strong>{jarPrompt.jarName || 'LiquidBounce jar'}</strong> from release <span class="tag-badge">{jarPrompt.tag}</span> (uploaded {jarPrompt.timeAgo}).
            </div>
            <div class="jar-prompt-actions">
                <button type="button" class="btn-dismiss" on:click={() => jarPrompt = null}>
                    Got it
                </button>
            </div>
        </div>
    </div>
{/if}

{#if firstRunWarningShown}
    <FirstRunWarning
            on:hide={hideFirstRunWarning}
            on:continue={continueAfterFirstRun}
    />
{/if}

{#if logShown}
    <ClientLog messages={log} on:hideClientLog={() => logShown = false} />
{/if}

{#if settingsShown}
    <Settings
            bind:options
            {info}
            {customMods}
            bind:activeTab={settingsTab}
            on:hide={async () => {
                settingsShown = false;
                await options.store();
                await refresh();
            }}
            on:refreshInfo={refresh}
            on:refreshMods={() => refreshMods(true)}
    />
{/if}

<VerticalFlexWrapper blur={settingsShown || logShown || firstRunWarningShown}>
    <MainHeader
            account={options.start.account}
            bind:options
            {running}
            {progressState}
            on:showSettings={() => showSettings("General")}
            on:showSkins={() => showSettings("Skins")}
            on:accountSwitched={(e) => {
                if (e.detail) {
                    options.start.account = e.detail;
                }
                options = options;
                refresh();
            }}
            on:accountSignedOut={() => {
                options.start.account = null;
                options = options;
            }}
    />

    <ContentWrapper>
        <BuildInfo {info} {infoError} {loadingInfo} {customMods} on:reload={refresh}/>
        <LaunchArea
                versionInfo={{
                    bannerUrl: "img/banner.png",
                    title: info ? `LiquidBounce v${info.version.liquidbounce}` : "Loading...",
                    date: info ? `Minecraft ${info.version.minecraft}` : "Loading..."
                }}
                mcVersion={info?.version.minecraft || "Loading..."}
                lbVersion={info?.version.liquidbounce || "Loading..."}
                instances={info?.instances ?? []}
                instance={selectedInstance}
                canLaunch={!!info}
                {running}
                on:selectInstance={e => selectInstance(e.detail)}
                on:showVersionSelect={() => showSettings("Instances")}
                on:manageInstances={() => showSettings("Instances")}
                on:showClientLog={() => logShown = true}
                on:launch={runClient}
                on:terminate={terminateClient}
        />
    </ContentWrapper>
</VerticalFlexWrapper>

<style>
    .jar-prompt-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.7);
        backdrop-filter: blur(5px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2500;
        animation: fadeIn 0.15s ease-out;
    }

    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    .jar-prompt-card {
        background: #181a20;
        border: 1px solid #2d313c;
        border-radius: 10px;
        padding: 22px 24px;
        width: 360px;
        display: flex;
        flex-direction: column;
        gap: 14px;
        box-shadow: 0 12px 35px rgba(0, 0, 0, 0.75);
    }

    .jar-prompt-header {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .jar-prompt-icon {
        width: 34px;
        height: 34px;
        border-radius: 8px;
        background: rgba(59, 130, 246, 0.15);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .jar-prompt-title {
        font-size: 15px;
        font-weight: 600;
        color: #ffffff;
    }

    .jar-prompt-message {
        font-size: 13px;
        color: #d1d5db;
        line-height: 1.5;
    }

    .tag-badge {
        background: #2563eb;
        color: #ffffff;
        padding: 2px 7px;
        border-radius: 4px;
        font-size: 12px;
        font-weight: 600;
    }

    .jar-prompt-actions {
        display: flex;
        justify-content: flex-end;
        margin-top: 4px;
    }

    .btn-dismiss {
        background: #2563eb;
        border: none;
        color: #ffffff;
        padding: 7px 18px;
        border-radius: 6px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s;
    }

    .btn-dismiss:hover {
        background: #1d4ed8;
    }
</style>
