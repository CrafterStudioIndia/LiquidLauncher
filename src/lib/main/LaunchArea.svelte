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
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.15);
    }

    .version-info .banner .title {
        font-weight: 800;
        color: white;
        font-size: 18px;
        margin: 10px 20px;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
    }

    .version-info .date {
        font-size: 12px;
        padding: 5px 12px;
        background-color: rgba(0, 0, 0, 0.65);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
        border-radius: 6px;
        position: absolute;
        top: 8px;
        right: 8px;
        font-weight: 500;
    }

    .instance-selector {
        display: flex;
        flex-direction: column;
        row-gap: 6px;
    }

    .label {
        font-size: 12px;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.65);
        letter-spacing: 0.02em;
    }

    .instance-row {
        display: grid;
        grid-template-columns: 1fr max-content;
        column-gap: 8px;
    }

    .select {
        width: 100%;
        background-color: rgba(15, 23, 42, 0.65);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
        font-family: "Inter", sans-serif;
        font-size: 13px;
        padding: 9px 12px;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.25);
    }

    .select:focus {
        border-color: #60a5fa;
        box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.25);
    }

    .select option {
        background-color: #0f172a;
        color: white;
    }

    .manage {
        background: rgba(30, 41, 59, 0.7);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 8px;
        color: white;
        font-family: "Inter", sans-serif;
        font-size: 12px;
        font-weight: 600;
        padding: 0 14px;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
    }

    .manage:hover {
        background-color: rgba(51, 65, 85, 0.9);
        border-color: rgba(99, 102, 241, 0.4);
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    .manage:active {
        transform: translateY(0);
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
