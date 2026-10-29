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

    onMount(refresh);
</script>

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
            {running}
            {progressState}
            on:showSettings={() => showSettings("General")}
            on:showSkins={() => showSettings("Skins")}
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
