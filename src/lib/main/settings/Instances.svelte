<script>
    import {createEventDispatcher} from "svelte";
    import {invoke} from "@tauri-apps/api/core";
    import {confirm} from "@tauri-apps/plugin-dialog";
    import SettingWrapper from "../../settings/SettingWrapper.svelte";
    import IconButtonSetting from "../../settings/IconButtonSetting.svelte";
    import TextSetting from "../../settings/TextSetting.svelte";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";
    import Description from "../../settings/Description.svelte";

    export let options;
    export let info = null;

    const dispatch = createEventDispatcher();

    let newName = "";
    let renaming = null;
    let renameValue = "";
    let busy = false;

    $: instances = info?.instances ?? [];
    $: selected = info?.selected?.id ?? options.launcher.instance;

    async function select(id) {
        options.launcher.instance = id;
        await options.store();
        dispatch("refreshInfo");
    }

    async function create() {
        if (busy || !newName.trim()) return;
        busy = true;
        try {
            const instance = await invoke("create_instance", {options, name: newName});
            options.launcher.instance = instance.id;
            await options.store();
            newName = "";
            dispatch("refreshInfo");
        } catch (e) {
            console.error("Failed to create the instance:", e);
            alert(`${e}`);
        } finally {
            busy = false;
        }
    }

    function startRename(instance) {
        renaming = instance.id;
        renameValue = instance.name;
    }

    async function rename() {
        if (!renaming || !renameValue.trim()) return;
        try {
            await invoke("rename_instance", {options, id: renaming, name: renameValue});
            renaming = null;
            dispatch("refreshInfo");
        } catch (e) {
            console.error("Failed to rename the instance:", e);
            alert(`${e}`);
        }
    }

    async function remove(instance) {
        if (!await confirm(`Delete "${instance.name}" and its worlds, mods and settings?`)) {
            return;
        }
        try {
            const remaining = await invoke("delete_instance", {options, id: instance.id});
            delete options.version.options[instance.id];
            if (options.launcher.instance === instance.id) {
                options.launcher.instance = remaining[0]?.id ?? "";
            }
            await options.store();
            if (renaming === instance.id) renaming = null;
            dispatch("refreshInfo");
        } catch (e) {
            console.error("Failed to delete the instance:", e);
            alert(`${e}`);
        }
    }
</script>

<Description description="Every instance has its own worlds, resource packs and mods. Minecraft, Fabric and Java are shared, so a new instance does not download the game again." />

<SettingWrapper title="Instances">
    {#each instances as instance (instance.id)}
        <div class="instance" class:selected={instance.id === selected}>
            {#if renaming === instance.id}
                <div class="rename">
                    <TextSetting bind:value={renameValue} placeholder="Instance name" />
                    <ButtonSetting small text="Save" on:click={rename} />
                    <ButtonSetting small text="Cancel" color="#5b5b5b" on:click={() => renaming = null} />
                </div>
            {:else}
                <div class="details">
                    <div class="name">{instance.name}</div>
                    <div class="line">
                        Minecraft {instance.minecraft}{instance.id === selected ? " · Selected" : ""}
                    </div>
                </div>
                <div class="actions">
                    {#if instance.id !== selected}
                        <IconButtonSetting text="Select" icon="icon-person" on:click={() => select(instance.id)} />
                    {/if}
                    <IconButtonSetting text="Rename" icon="icon-cog" on:click={() => startRename(instance)} />
                    <button class="delete" type="button" title="Delete instance" on:click={() => remove(instance)}>
                        <img src="img/icon/icon-button-close.svg" alt="delete">
                    </button>
                </div>
            {/if}
        </div>
    {/each}
</SettingWrapper>

<div class="create">
    <TextSetting title="New instance" placeholder="Name" bind:value={newName} />
    <ButtonSetting text="Create instance" disabled={busy || !newName.trim()} on:click={create} />
</div>

<style>
    .instance {
        display: grid;
        grid-template-columns: 1fr max-content;
        gap: 10px;
        align-items: center;
        padding: 6px;
        border-radius: 6px;
        border: solid 1px transparent;
    }

    .instance.selected {
        border-color: #4677FF;
    }

    .details {
        min-width: 0;
    }

    .name {
        color: white;
        font-size: 14px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .line {
        font-size: 12px;
        color: rgba(255, 255, 255, .5);
    }

    .actions {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .delete {
        background: transparent;
        border: none;
        cursor: pointer;
        display: flex;
        align-items: center;
        padding: 0;
    }

    .delete img {
        height: 10px;
    }

    .rename {
        grid-column: 1 / -1;
        display: flex;
        align-items: flex-end;
        gap: 8px;
    }

    .rename > :global(.text-setting) {
        flex: 1;
    }

    .create {
        display: flex;
        flex-direction: column;
        row-gap: 10px;
    }
</style>
