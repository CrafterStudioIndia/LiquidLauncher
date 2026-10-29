<script>
    import {createEventDispatcher} from "svelte";
    import GeneralSettings from "./GeneralSettings.svelte";
    import Instances from "./Instances.svelte";
    import Mods from "./Mods.svelte";
    import Modrinth from "./Modrinth.svelte";
    import MinecraftSettings from "./MinecraftSettings.svelte";
    import AppearanceSettings from "./Appearance.svelte";
    import SkinsSettings from "./Skins.svelte";
    import SettingsContainer from "../../settings/SettingsContainer.svelte";
    import Tabs from "../../settings/tab/Tabs.svelte";

    export let options;
    export let info = null;
    export let customMods = [];
    export let activeTab = "General";

    const dispatch = createEventDispatcher();

    let modView = "list";

    $: if (activeTab !== "Mods") modView = "list";
</script>

<SettingsContainer
        title="Settings"
        fill={activeTab === "Mods" || activeTab === "Skins"}
        on:hideSettings={() => dispatch('hide')}
>
    <Tabs
            tabs={["General", "Instances", "Skins", "Mods", "Minecraft", "Appearance"]}
            bind:activeTab
            slot="tabs"
    />

    {#if activeTab === "General"}
        <GeneralSettings
                bind:options
                on:refreshInfo={() => dispatch("refreshInfo")}
        />
    {:else if activeTab === "Instances"}
        <Instances
                bind:options
                {info}
                on:refreshInfo={() => dispatch("refreshInfo")}
        />
    {:else if activeTab === "Skins"}
        <SkinsSettings
                bind:options
                on:refreshInfo={() => dispatch("refreshInfo")}
        />
    {:else if activeTab === "Mods"}
        {#if modView === "browse"}
            <Modrinth
                    {options}
                    {info}
                    on:back={() => modView = "list"}
                    on:updateMods={() => dispatch("refreshMods")}
            />
        {:else}
            <Mods
                    {options}
                    {info}
                    {customMods}
                    on:browse={() => modView = "browse"}
                    on:refreshMods={() => dispatch("refreshMods")}
            />
        {/if}
    {:else if activeTab === "Minecraft"}
        <MinecraftSettings bind:options />
    {:else if activeTab === "Appearance"}
        <AppearanceSettings bind:options />
    {/if}
</SettingsContainer>
