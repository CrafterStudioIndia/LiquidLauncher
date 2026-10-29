<script>
    import {createEventDispatcher} from "svelte";
    import ButtonSetting from "../settings/ButtonSetting.svelte";
    import Message from "../settings/Message.svelte";
    import RippleLoader from "../common/RippleLoader.svelte";

    export let info = null;
    export let infoError = null;
    export let loadingInfo = false;
    /** The mods the selected instance has, beside LiquidBounce and Fabric API. */
    export let customMods = [];

    const dispatch = createEventDispatcher();

    function date(value) {
        if (!value) return "Never";
        const parsed = new Date(value);
        return Number.isNaN(parsed.getTime()) ? "Never" : parsed.toLocaleString();
    }

    function name(value) {
        const trimmed = (value ?? "").replace(/[\\/]+$/, "");
        const index = Math.max(trimmed.lastIndexOf("/"), trimmed.lastIndexOf("\\"));
        return index === -1 ? trimmed : trimmed.slice(index + 1);
    }
</script>

<div class="build-info">
    <div class="header">
        <div class="title">Local build</div>
        <button class="refresh" type="button" title="Read again" on:click={() => dispatch("reload")}>
            ⟳
        </button>
    </div>

    {#if infoError}
        <Message title="The local build could not be read" note={infoError}>
            <ButtonSetting small text="Try again" on:click={() => dispatch("reload")}/>
        </Message>
    {:else if !info || loadingInfo}
        <Message>
            <RippleLoader size={80}/>
        </Message>
    {:else}
        <div class="rows">
            <div class="row"><span>Instance</span><span>{info.selected.name}</span></div>
            <div class="row"><span>Minecraft</span><span>{info.version.minecraft}</span></div>
            <div class="row"><span>Fabric loader</span><span>{info.version.loader}</span></div>
            <div class="row"><span>Fabric API</span><span>{info.version.fabricApi}</span></div>
            <div class="row"><span>LiquidBounce</span><span>{info.version.liquidbounce}</span></div>
            <div class="row"><span>Java</span><span>{info.version.java}</span></div>
            <div class="row"><span>Mods</span><span>{customMods.length} added</span></div>
            <div class="row"><span>Last played</span><span>{date(info.selected.lastPlayed)}</span></div>
        </div>

        <div class="jar">
            <div class="label">Client jar</div>
            <div class="path" title={info.clientJar}>{name(info.clientJar)}</div>
        </div>
    {/if}
</div>

<style>
    .build-info {
        background-color: rgba(0, 0, 0, .26);
        border-radius: 12px;
        padding: 18px;
        display: flex;
        flex-direction: column;
        row-gap: 12px;
        overflow: hidden;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .title {
        font-size: 16px;
        color: white;
        font-weight: 600;
    }

    .refresh {
        background-color: transparent;
        border: none;
        color: rgba(255, 255, 255, .5);
        font-size: 18px;
        cursor: pointer;
        line-height: 1;
    }

    .refresh:hover {
        color: white;
    }

    .rows {
        display: flex;
        flex-direction: column;
        row-gap: 8px;
        overflow: auto;
    }

    .row {
        display: grid;
        grid-template-columns: 110px 1fr;
        font-size: 12px;
        column-gap: 10px;
    }

    .row > span:first-child {
        color: rgba(255, 255, 255, .5);
    }

    .row > span:last-child {
        color: white;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .jar {
        display: flex;
        flex-direction: column;
        row-gap: 4px;
        border-top: solid 1px rgba(255, 255, 255, .1);
        padding-top: 12px;
    }

    .label {
        font-size: 12px;
        color: rgba(255, 255, 255, .5);
    }

    .path {
        font-size: 12px;
        color: white;
        word-break: break-all;
    }
</style>
