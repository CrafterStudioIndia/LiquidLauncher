<script>
    import { createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";
    import ItemRow from "../../settings/ItemRow.svelte";
    import Message from "../../settings/Message.svelte";
    import ListView from "../../settings/ListView.svelte";
    import { track } from "./Mods.svelte";

    export let options;
    export let info = null;

    const dispatch = createEventDispatcher();

    let view;
    let query = "";
    let busy = new Set();

    const request = query => invoke("modrinth_search", {options, query});

    async function install(hit) {
        busy = new Set(busy).add(hit.projectId);
        try {
            const entry = await invoke("modrinth_install", {projectId: hit.projectId});
            await track(options, info.selected.id, entry);
            dispatch("updateMods");
        } catch (e) {
            console.error("Failed to install:", e);
            alert(`${e}`);
        }
        await view.load(true);
        busy.delete(hit.projectId);
        busy = busy;
    }
</script>

<ListView
        bind:this={view}
        bind:query
        placeholder="Search Modrinth mods"
        title="Modrinth"
        {request}
        on:back
        let:result
>
    <svelte:fragment slot="aside" let:result>{result?.target ?? ""}</svelte:fragment>

    {#each result.hits as hit (hit.projectId)}
        <ItemRow name={hit.title} description={hit.description} openable={false} status={hit.installed ? "Installed" : null}>
            {hit.line}
            <svelte:fragment slot="side">
                {#if !hit.installed}
                    <ButtonSetting
                            small
                            text={busy.has(hit.projectId) ? "Installing" : "Install"}
                            disabled={busy.has(hit.projectId)}
                            on:click={() => install(hit)}
                    />
                {/if}
            </svelte:fragment>
        </ItemRow>
    {:else}
        {#if result.empty}
            <Message title={result.empty} clearable on:clear={() => query = ""} />
        {/if}
    {/each}
</ListView>
