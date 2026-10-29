<script>
    import ButtonClose from "../common/ButtonClose.svelte";
    import Logo from "../common/Logo.svelte";
    import TitleBar from "../common/TitleBar.svelte";
    import VerticalFlexWrapper from "../common/VerticalFlexWrapper.svelte";
    import LoginModal from "./loginmodal/LoginModal.svelte";

    export let options;
</script>

<VerticalFlexWrapper blur={false}>
    <TitleBar>
        <Logo />
        <ButtonClose />
    </TitleBar>

    <div class="main-section-wrapper">
        <div class="intro">
            <div class="title">Welcome to LiquidBounce.</div>
            <div class="description">
                A local build for Minecraft 1.21.11. Choose a username to play with an
                offline account; the launcher supplies Minecraft, Fabric and LiquidBounce
                without asking a remote service for a build.
            </div>
        </div>

        <LoginModal bind:options />
    </div>
</VerticalFlexWrapper>

<style>
    .main-section-wrapper {
        display: grid;
        grid-template-columns: 1fr max-content;
        flex: 1;
        padding: 0 50px;
        margin-bottom: 20px;
    }

    .intro {
        display: flex;
        flex-direction: column;
        justify-content: center;
        row-gap: 15px;
        max-width: 450px;
    }

    .title {
        font-size: 40px;
        color: white;
        font-weight: 600;
    }

    .description {
        font-size: 18px;
        color: rgba(255, 255, 255, 0.5);
    }
</style>
