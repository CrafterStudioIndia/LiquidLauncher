<script>
    import {createEventDispatcher} from "svelte";
    import SettingsContainer from "../settings/SettingsContainer.svelte";
    import Description from "../settings/Description.svelte";
    import ButtonSetting from "../settings/ButtonSetting.svelte";

    const dispatch = createEventDispatcher();
</script>

<SettingsContainer
        title="Ready to play"
        on:hideSettings={() => dispatch('hide')}
>
    <Description
            description="The first launch downloads Minecraft 1.21.11, the Fabric loader and Fabric API from Mojang's, Fabric's and Modrinth's own servers. LiquidBounce itself is the jar bundled with this launcher; no update or build server is contacted."
    />

    <Description
            description="Everything is kept in the data folder you choose in the settings, so you can launch again without a connection once the files are downloaded."
    />

    <ButtonSetting
            text="Continue"
            on:click={() => dispatch('continue')}
            color="#4677FF"
    />
</SettingsContainer>
