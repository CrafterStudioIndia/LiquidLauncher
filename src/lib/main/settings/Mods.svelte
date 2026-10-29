<script context="module">
    // The options keep the mods installed from Modrinth by instance and Minecraft version.
    export function track(options, instanceId, entry) {
        const branch = options.version.options[instanceId] ??= {modStates: {}, customModStates: {}};
        branch.modrinthMods ??= {};
        const mods = branch.modrinthMods["1.21.11"] ??= [];
        const index = mods.findIndex(mod => mod.projectId === entry.projectId);
        if (index === -1) {
            mods.push(entry);
        } else {
            mods[index] = entry;
        }
        return options.store();
    }
</script>

<script>
    import { createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
    import SettingWrapper from "../../settings/SettingWrapper.svelte";
    import ModSetting from "../../settings/ModSetting.svelte";
    import IconButtonSetting from "../../settings/IconButtonSetting.svelte";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";
    import Description from "../../settings/Description.svelte";

    export let options;
    export let info = null;
    export let customMods = [];

    const dispatch = createEventDispatcher();

    let updating = new Set();

    $: builtinMods = info ? [
        {
            name: "LiquidBounce",
            title: `LiquidBounce v${info.version.liquidbounce}`,
            enabled: true,
            required: true,
            line: "Local jar"
        },
        {
            name: "Fabric API",
            title: "Fabric API",
            enabled: true,
            required: true,
            line: `Fabric ${info.version.fabricApi}`
        }
    ] : [];

    async function addFile() {
        try {
            const selected = await dialogOpen({
                directory: false,
                multiple: true,
                filters: [{ name: "Minecraft mod", extensions: ["jar"] }],
                title: "Select a mod to install"
            });

            if (selected) {
                for (const file of selected) {
                    await invoke("install_custom_mod", {options, path: file});
                }
                dispatch("refreshMods");
            }
        } catch (e) {
            console.error("Failed to install mod:", e);
            alert(`Failed to install mod: ${e}`);
        }
    }

    function deleteFile(fileName) {
        return invoke("delete_custom_mod", {options, modName: fileName});
    }

    async function remove(mod) {
        try {
            if (mod.source.type === "local") {
                await deleteFile(mod.source.fileName);
            } else {
                const instanceId = info.selected.id;
                const branch = options.version.options[instanceId];
                const mods = branch?.modrinthMods?.["1.21.11"] ?? [];
                const index = mods.findIndex(entry => entry.projectId === mod.modrinth.projectId);
                if (index !== -1) {
                    mods.splice(index, 1);
                }
                await options.store();
            }
        } catch (e) {
            console.error("Failed to delete mod:", e);
            alert(`Failed to delete mod: ${e}`);
        }
        dispatch("refreshMods");
    }

    // A file Modrinth knows becomes a Modrinth mod on the newer version.
    async function update(mod) {
        const key = modKey(mod);
        updating = new Set(updating).add(key);
        try {
            const entry = await invoke("modrinth_install", {projectId: mod.modrinth.projectId});
            await track(options, info.selected.id, entry);
            if (mod.source.type === "local") {
                await deleteFile(mod.source.fileName);
            }
        } catch (e) {
            console.error("Failed to update mod:", e);
            alert(`${e}`);
        }
        updating.delete(key);
        updating = updating;
        dispatch("refreshMods");
    }

    function modKey(mod) {
        return `${mod.source.type}:${mod.name}`;
    }

    /// Remembers whether a mod of this instance is on, keyed by the name the launch list uses.
    function toggle(mod, event) {
        const instanceId = info.selected.id;
        const branch = options.version.options[instanceId] ??= { modStates: {}, customModStates: {} };
        branch.customModStates ??= {};
        branch.customModStates[mod.name] = event?.detail?.target?.checked ?? mod.enabled;
        options.store();
    }
</script>

<Description description="LiquidBounce and Fabric API always load. Anything else here is copied into this instance's mods folder when it launches." />

{#if info}
    <SettingWrapper title="Mods - Fabric {info.version.minecraft}" unbounded>
        <div slot="title-element" class="actions">
            <IconButtonSetting text="Add file" icon="icon-plus" on:click={addFile} />
            <IconButtonSetting text="Browse" icon="icon-plus" on:click={() => dispatch("browse")} />
        </div>
        {#each builtinMods as mod}
            <ModSetting
                    title={mod.title}
                    bind:value={mod.enabled}
                    disabled={true}
                    removable={false}
            >
                <svelte:fragment slot="line">{mod.line}</svelte:fragment>
            </ModSetting>
        {/each}
        {#each customMods as mod (modKey(mod))}
            <ModSetting
                    title={mod.title}
                    bind:value={mod.enabled}
                    on:change={event => toggle(mod, event)}
                    on:delete={() => remove(mod)}
            >
                <svelte:fragment slot="line">
                    {mod.version ?? "Unknown version"}{#if mod.modrinth?.update}{" · "}<span class="strong">{mod.modrinth.update} available</span>{/if}
                </svelte:fragment>
                <svelte:fragment slot="side">
                    {#if mod.modrinth?.update}
                        <ButtonSetting
                                small
                                text={updating.has(modKey(mod)) ? "Updating" : "Update"}
                                disabled={updating.has(modKey(mod))}
                                on:click={() => update(mod)}
                        />
                    {/if}
                </svelte:fragment>
            </ModSetting>
        {/each}
        {#if customMods.length === 0}
            <div class="empty">No mods added to this instance yet.</div>
        {/if}
    </SettingWrapper>
{/if}

<style>
    .actions {
        display: flex;
        column-gap: 15px;
    }

    .strong {
        color: white;
    }

    .empty {
        font-size: 12px;
        color: rgba(255, 255, 255, .5);
        padding: 4px 0;
    }
</style>
