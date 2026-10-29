<script>
    import {invoke} from "@tauri-apps/api/core";
    import ModalButton from "./ModalButton.svelte";
    import ModalInput from "./ModalInput.svelte";
    import RippleLoader from "../../common/RippleLoader.svelte";

    export let options;

    let username = "";
    let busy = false;

    $: canLogin = username.trim().length > 0 && username.trim().length <= 16 && !busy;

    async function login() {
        if (!canLogin) return;
        busy = true;
        try {
            options.start.account = await invoke("login_offline", {username: username.trim()});
            await options.store();
        } catch (e) {
            console.error("Failed to sign in:", e);
            alert(`Failed to sign in: ${e}`);
        } finally {
            busy = false;
        }
    }
</script>

<div class="login-modal">
    <div class="title">Offline account</div>

    <ModalInput
            icon="person"
            placeholder="Username"
            characterLimit={16}
            bind:value={username}
    />

    <div class="note">
        Only offline accounts are supported. Use a name that is not registered on the
        server you join, or the server will refuse it.
    </div>

    {#if busy}
        <RippleLoader size={80} />
    {:else}
        <ModalButton text="Play" primary disabled={!canLogin} on:click={login}/>
    {/if}
</div>

<style>
    .login-modal {
        background-color: rgba(0, 0, 0, 0.36);
        border-radius: 12px;
        padding: 25px;
        width: 350px;
        display: flex;
        flex-direction: column;
        row-gap: 20px;
        justify-content: center;
        align-self: center;
    }

    .title {
        font-size: 18px;
        color: white;
        font-weight: 600;
    }

    .note {
        font-size: 12px;
        line-height: 16px;
        color: rgba(255, 255, 255, 0.5);
    }
</style>
