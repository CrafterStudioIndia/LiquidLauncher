<script>
    import { createEventDispatcher } from "svelte";
    import ButtonLaunchArea from "./ButtonLaunchArea.svelte";
    import ButtonVersion from "./ButtonVersion.svelte";

    export let versionInfo;
    export let mcVersion;
    export let lbVersion;
    export let instance = null;
    export let instances = [];
    export let running;
    export let canLaunch = true;

    const dispatch = createEventDispatcher();

    function selectInstance(event) {
        dispatch("selectInstance", event.currentTarget.value);
    }
</script>

<div class="launch-area">
    <div class="version-info">
        <div class="banner" style="background-image: linear-gradient(to bottom, transparent, #4677ffc5), url({versionInfo.bannerUrl});">
            <div class="title">{versionInfo.title}</div>
            <div class="date">{versionInfo.date}</div>
        </div>
    </div>

    <div class="instance-selector">
        <label class="label" for="instance">Instance</label>
        <div class="instance-row">
            <select id="instance" class="select" value={instance?.id ?? ""} on:change={selectInstance} disabled={running || instances.length === 0}>
                {#each instances as entry (entry.id)}
                    <option value={entry.id}>{entry.name}</option>
                {/each}
            </select>
            <button class="manage" type="button" on:click={() => dispatch("manageInstances")}>Manage</button>
        </div>
    </div>

    <div class="version-selector">
        <ButtonVersion icon="lb" title={lbVersion} subtitle="LiquidBounce" on:click={() => dispatch("showVersionSelect")} />
        <ButtonVersion icon="mc" title={mcVersion} subtitle="Minecraft" on:click={() => dispatch("showVersionSelect")} />
    </div>

    {#if running}
        <div class="running-button-wrapper">
            <ButtonLaunchArea text="Terminate" active={true} on:click={() => dispatch("terminate")} />
            <ButtonLaunchArea text="Log" active={false} on:click={() => dispatch("showClientLog")} />
        </div>
    {:else}
        <ButtonLaunchArea
                text="Launch LiquidBounce"
                active={false}
                disabled={!canLaunch}
                on:click={() => dispatch("launch")}
        />
    {/if}
</div>

<style>
    .launch-area {
        flex-direction: column;
        display: flex;
        row-gap: 20px;
        max-height: 100%;
        overflow: hidden;
    }

    .version-info .banner {
        border-radius: 12px;
        height: 100px;
        background-size: cover;
        overflow: hidden;
        position: relative;
        display: flex;
        align-items: flex-end;
    }

    .version-info .banner .title {
        font-weight: 800;
        color: white;
        font-size: 18px;
        margin: 10px 20px;
    }

    .version-info .date {
        font-size: 12px;
        padding: 5px 12px;
        background-color: rgba(0, 0, 0, .58);
        color: white;
        border-radius: 4px;
        position: absolute;
        top: 8px;
        right: 8px;
    }

    .instance-selector {
        display: flex;
        flex-direction: column;
        row-gap: 5px;
    }

    .label {
        font-size: 12px;
        color: rgba(255, 255, 255, .5);
    }

    .instance-row {
        display: grid;
        grid-template-columns: 1fr max-content;
        column-gap: 8px;
    }

    .select {
        width: 100%;
        background-color: rgba(0, 0, 0, .26);
        border: none;
        border-bottom: solid 1px #4677FF;
        color: white;
        font-family: "Inter", sans-serif;
        padding: 7px;
        border-radius: 4px;
        cursor: pointer;
    }

    .select option {
        background-color: white;
        color: black;
    }

    .manage {
        background-color: rgba(0, 0, 0, .26);
        border: solid 1px #4677FF;
        border-radius: 4px;
        color: white;
        font-family: "Inter", sans-serif;
        font-size: 12px;
        padding: 0 12px;
        cursor: pointer;
    }

    .manage:hover {
        background-color: rgba(70, 119, 255, .2);
    }

    .version-selector {
        display: grid;
        grid-template-columns: 1fr 1fr;
        column-gap: 10px;
    }

    .running-button-wrapper {
        display: grid;
        grid-template-columns: 1fr max-content;
        column-gap: 10px;
    }
</style>
