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
    import AccountSwitcherMenu from "../AccountSwitcherMenu.svelte";
    import {onMount} from "svelte";
    import {invoke} from "@tauri-apps/api/core";

    export let options;

    const dispatch = createEventDispatcher();

    let launcherVersion = "";
    let defaultDataFolder = "";
    let systemMemory = options.start.memory;

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

<div class="settings-account-card">
    <div class="account-card-title">Account</div>
    <AccountSwitcherMenu
        bind:options
        inline={true}
        on:accountSwitched={() => dispatch("refreshInfo")}
        on:accountSignedOut={() => dispatch("refreshInfo")}
    />
</div>

<ButtonSetting text="Clear Data" on:click={clearData} color="#B83529" />

<LauncherVersion version={launcherVersion} />

<style>
    .settings-account-card {
        margin: 12px 0 16px 0;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .account-card-title {
        font-size: 13px;
        font-weight: 600;
        color: #ffffff;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
</style>
