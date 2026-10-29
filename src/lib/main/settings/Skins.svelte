<script>
    import { onMount, createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open as dialogOpen, save as dialogSave } from "@tauri-apps/plugin-dialog";
    import SkinPreview3D from "../../skins/SkinPreview3D.svelte";
    import {
        activeSkin,
        getSkinHeadDataUrl,
        getSkinFullBodyDataUrl,
        DEFAULT_MINECRAFT_SKINS
    } from "../../skins/skin_utils.js";
    import { Circle } from "svelte-loading-spinners";

    export let options;

    const dispatch = createEventDispatcher();

    let skins = [];
    let selectedSkin = null;
    let loading = true;
    let actionLoading = false;
    let errorMessage = null;

    // Body renders cache: skin.id -> dataUrl
    let bodyRenderMap = {};
    let defaultRenderMap = {};

    // Section collapse states
    let savedSkinsOpen = true;
    let defaultSkinsOpen = true;

    // Fetch from player input
    let fetchPlayerUsername = "";
    let fetchLoading = false;

    // Drag-and-drop state
    let isDraggingOver = false;

    // Edit Skin Modal state
    let showEditModal = false;
    let editSkinTarget = null;
    let editSkinName = "";
    let editSkinModel = "classic";
    let editModalLoading = false;
    let editModalError = null;

    // Derive player display name
    $: playerName = options?.start?.account?.name || options?.auth?.name || "Crafter_Studio";

    async function loadSkins() {
        loading = true;
        errorMessage = null;
        try {
            skins = await invoke("list_skins", { options });
            if (skins.length > 0) {
                const active = skins.find((s) => s.active) || skins[0];
                selectedSkin = active;
                activeSkin.set(active);
            }

            // Generate full-body renders for all saved skins
            for (const s of skins) {
                if (s.dataUrl) {
                    bodyRenderMap[s.id] = await getSkinFullBodyDataUrl(s.dataUrl, s.model, 140, 180);
                }
            }

            // Generate full-body renders for default skins
            for (const def of DEFAULT_MINECRAFT_SKINS) {
                if (!defaultRenderMap[def.id]) {
                    defaultRenderMap[def.id] = await getSkinFullBodyDataUrl(def.path, def.model, 140, 180);
                }
            }
        } catch (err) {
            console.error("Failed to load skins:", err);
            errorMessage = err.toString();
        } finally {
            loading = false;
        }
    }

    async function handleEquipSkin(skin) {
        if (!skin) return;
        actionLoading = true;
        errorMessage = null;
        try {
            const updated = await invoke("set_active_skin", { options, id: skin.id });
            await loadSkins();
            selectedSkin = skins.find((s) => s.id === updated.id) || updated;
            activeSkin.set(selectedSkin);
            dispatch("skinChanged", selectedSkin);
        } catch (err) {
            console.error("Failed to set active skin:", err);
            errorMessage = err.toString();
        } finally {
            actionLoading = false;
        }
    }

    async function handleEquipDefaultSkin(defSkin) {
        actionLoading = true;
        errorMessage = null;
        try {
            // Check if this default skin is already imported in saved skins
            const existing = skins.find((s) => s.name.toLowerCase() === defSkin.name.toLowerCase());
            if (existing) {
                await handleEquipSkin(existing);
                return;
            }

            // Fetch texture image and convert to base64
            const res = await fetch(defSkin.path);
            const blob = await res.blob();
            const reader = new FileReader();

            reader.onloadend = async () => {
                try {
                    const base64Data = reader.result;
                    const created = await invoke("add_skin", {
                        options,
                        name: defSkin.name,
                        model: defSkin.model,
                        sourceType: "base64",
                        source: base64Data,
                        setActive: true,
                    });
                    await loadSkins();
                    selectedSkin = skins.find((s) => s.id === created.id) || created;
                    activeSkin.set(selectedSkin);
                    dispatch("skinChanged", selectedSkin);
                } catch (err) {
                    console.error("Error adding default skin:", err);
                    errorMessage = err.toString();
                } finally {
                    actionLoading = false;
                }
            };
            reader.readAsDataURL(blob);
        } catch (err) {
            console.error("Failed to equip default skin:", err);
            errorMessage = err.toString();
            actionLoading = false;
        }
    }

    async function handleFetchPlayerSkin() {
        const username = fetchPlayerUsername.trim();
        if (!username) return;

        fetchLoading = true;
        errorMessage = null;
        try {
            const created = await invoke("add_skin", {
                options,
                name: username,
                model: "classic",
                sourceType: "mojang",
                source: username,
                setActive: true,
            });

            fetchPlayerUsername = "";
            await loadSkins();
            selectedSkin = skins.find((s) => s.id === created.id) || created;
            activeSkin.set(selectedSkin);
            dispatch("skinChanged", selectedSkin);
        } catch (err) {
            console.error("Failed to fetch player skin:", err);
            errorMessage = "Failed to fetch skin: " + err.toString();
        } finally {
            fetchLoading = false;
        }
    }

    async function handleBrowseFile() {
        try {
            const selected = await dialogOpen({
                multiple: false,
                filters: [{ name: "Minecraft Skin (.png)", extensions: ["png"] }],
            });
            if (selected) {
                const path = typeof selected === "string" ? selected : selected[0];
                const filename = path.split(/[\/\\]/).pop().replace(/\.png$/i, "");
                const skinName = filename.charAt(0).toUpperCase() + filename.slice(1);

                actionLoading = true;
                const created = await invoke("add_skin", {
                    options,
                    name: skinName,
                    model: "classic",
                    sourceType: "file",
                    source: path,
                    setActive: true,
                });
                await loadSkins();
                selectedSkin = skins.find((s) => s.id === created.id) || created;
                activeSkin.set(selectedSkin);
                dispatch("skinChanged", selectedSkin);
            }
        } catch (err) {
            console.error("File selection error:", err);
            errorMessage = err.toString();
        } finally {
            actionLoading = false;
        }
    }

    function handleDragOver(e) {
        e.preventDefault();
        isDraggingOver = true;
    }

    function handleDragLeave(e) {
        e.preventDefault();
        isDraggingOver = false;
    }

    async function handleDrop(e) {
        e.preventDefault();
        isDraggingOver = false;

        const files = e.dataTransfer?.files;
        if (!files || files.length === 0) return;

        const file = files[0];
        if (!file.name.toLowerCase().endsWith(".png")) {
            errorMessage = "Please drop a valid .png Minecraft skin image.";
            return;
        }

        const skinName = file.name.replace(/\.png$/i, "");
        const formattedName = skinName.charAt(0).toUpperCase() + skinName.slice(1);

        const reader = new FileReader();
        actionLoading = true;
        reader.onloadend = async () => {
            try {
                const base64Data = reader.result;
                const created = await invoke("add_skin", {
                    options,
                    name: formattedName,
                    model: "classic",
                    sourceType: "base64",
                    source: base64Data,
                    setActive: true,
                });
                await loadSkins();
                selectedSkin = skins.find((s) => s.id === created.id) || created;
                activeSkin.set(selectedSkin);
                dispatch("skinChanged", selectedSkin);
            } catch (err) {
                console.error("Failed to add dropped skin:", err);
                errorMessage = err.toString();
            } finally {
                actionLoading = false;
            }
        };
        reader.readAsDataURL(file);
    }

    function openEditModal(skin) {
        if (!skin) return;
        editSkinTarget = skin;
        editSkinName = skin.name;
        editSkinModel = skin.model || "classic";
        editModalError = null;
        showEditModal = true;
    }

    function closeEditModal() {
        showEditModal = false;
        editSkinTarget = null;
        editModalError = null;
    }

    async function handleSaveEdit() {
        if (!editSkinTarget) return;
        editModalLoading = true;
        editModalError = null;
        try {
            const updated = await invoke("update_skin", {
                options,
                id: editSkinTarget.id,
                name: editSkinName.trim(),
                model: editSkinModel,
            });
            await loadSkins();
            selectedSkin = skins.find((s) => s.id === updated.id) || updated;
            if (selectedSkin.active) {
                activeSkin.set(selectedSkin);
                dispatch("skinChanged", selectedSkin);
            }
            closeEditModal();
        } catch (err) {
            console.error("Failed to update skin:", err);
            editModalError = err.toString();
        } finally {
            editModalLoading = false;
        }
    }

    async function handleDeleteFromEdit() {
        if (!editSkinTarget || skins.length <= 1) return;
        if (!confirm(`Are you sure you want to delete "${editSkinTarget.name}"?`)) return;

        editModalLoading = true;
        try {
            skins = await invoke("delete_skin", { options, id: editSkinTarget.id });
            selectedSkin = skins.find((s) => s.active) || skins[0];
            activeSkin.set(selectedSkin);
            dispatch("skinChanged", selectedSkin);
            closeEditModal();
        } catch (err) {
            console.error("Failed to delete skin:", err);
            editModalError = err.toString();
        } finally {
            editModalLoading = false;
        }
    }

    async function handleExportFromEdit() {
        if (!editSkinTarget) return;
        try {
            const savePath = await dialogSave({
                filters: [{ name: "PNG Image", extensions: ["png"] }],
                defaultPath: `${editSkinTarget.name.toLowerCase().replace(/[^a-z0-9]/g, "_")}.png`,
            });
            if (savePath) {
                await invoke("export_skin", { id: editSkinTarget.id, path: savePath });
            }
        } catch (err) {
            console.error("Failed to export skin:", err);
            alert("Export failed: " + err.toString());
        }
    }

    onMount(() => {
        loadSkins();
    });
</script>

<div class="skin-selector-root">
    {#if loading}
        <div class="loading-state">
            <Circle size="42" color="#10b981" unit="px" />
            <p>Loading skins...</p>
        </div>
    {:else}
        {#if errorMessage}
            <div class="error-banner">
                <span>{errorMessage}</span>
                <button type="button" class="dismiss-btn" on:click={() => (errorMessage = null)}>✕</button>
            </div>
        {/if}

        <div class="modrinth-skin-layout">
            <!-- Left Column: 3D Character Viewport & Controls -->
            <div class="left-pane">
                <div class="left-header">
                    <h2 class="skin-selector-title">Skin selector</h2>
                    <div class="player-tag">{playerName}</div>
                </div>

                <!-- 3D WebGL Viewport -->
                <div class="model-viewport">
                    {#if selectedSkin?.dataUrl}
                        <SkinPreview3D
                            skinUrl={selectedSkin.dataUrl}
                            model={selectedSkin.model}
                        />
                    {:else}
                        <div class="no-preview">
                            <p>No skin selected</p>
                        </div>
                    {/if}
                </div>

                <div class="drag-hint">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="8 3 3 8 8 13"/>
                        <polyline points="16 3 21 8 16 13"/>
                        <line x1="3" y1="8" x2="21" y2="8"/>
                        <polyline points="8 21 3 16 8 11"/>
                        <polyline points="16 21 21 16 16 11"/>
                        <line x1="3" y1="16" x2="21" y2="16"/>
                    </svg>
                    <span>Drag to rotate</span>
                </div>

                <!-- Edit Skin Button -->
                <button
                    type="button"
                    class="edit-skin-btn"
                    disabled={!selectedSkin}
                    on:click={() => openEditModal(selectedSkin)}
                >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 20h9"/>
                        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>
                    </svg>
                    <span>Edit skin</span>
                </button>

                <!-- Bottom Fetch Skin from Player -->
                <form class="fetch-row" on:submit|preventDefault={handleFetchPlayerSkin}>
                    <input
                        type="text"
                        class="fetch-input"
                        placeholder="Fetch skin from player"
                        bind:value={fetchPlayerUsername}
                        disabled={fetchLoading}
                    />
                    <button
                        type="submit"
                        class="fetch-btn"
                        disabled={fetchLoading || !fetchPlayerUsername.trim()}
                    >
                        {#if fetchLoading}
                            <Circle size="14" color="#ffffff" unit="px" />
                        {:else}
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                <polyline points="7 10 12 15 17 10"/>
                                <line x1="12" y1="15" x2="12" y2="3"/>
                            </svg>
                            <span>Fetch</span>
                        {/if}
                    </button>
                </form>
            </div>

            <!-- Right Column: Saved Skins & Default Skins Grids -->
            <div class="right-pane">
                <!-- Section 1: Saved Skins -->
                <div class="skins-section">
                    <button
                        type="button"
                        class="section-header-btn"
                        on:click={() => (savedSkinsOpen = !savedSkinsOpen)}
                    >
                        <span class="chevron" class:collapsed={!savedSkinsOpen}>⌃</span>
                        <span class="section-title">Saved skins</span>
                    </button>

                    {#if savedSkinsOpen}
                        <div class="cards-grid">
                            <!-- Add Skin Card with Drag & Drop -->
                            <div
                                class="card add-card"
                                class:drag-over={isDraggingOver}
                                role="button"
                                tabindex="0"
                                on:click={handleBrowseFile}
                                on:keydown={(e) => (e.key === "Enter" || e.key === " ") && handleBrowseFile()}
                                on:dragover={handleDragOver}
                                on:dragleave={handleDragLeave}
                                on:drop={handleDrop}
                            >
                                <div class="add-icon">+</div>
                                <div class="add-label">Add skin</div>
                                <div class="add-subtext">Drag and drop</div>
                            </div>

                            <!-- Saved Skins Cards -->
                            {#each skins as skin (skin.id)}
                                <div
                                    class="card skin-card"
                                    class:active={skin.active}
                                    class:selected={selectedSkin?.id === skin.id}
                                    role="button"
                                    tabindex="0"
                                    on:click={() => {
                                        selectedSkin = skin;
                                        handleEquipSkin(skin);
                                    }}
                                    on:keydown={(e) => {
                                        if (e.key === "Enter" || e.key === " ") {
                                            selectedSkin = skin;
                                            handleEquipSkin(skin);
                                        }
                                    }}
                                >
                                    <!-- Active Checkmark Badge -->
                                    {#if skin.active}
                                        <div class="active-checkmark-badge" title="Active skin">
                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#18191c" stroke-width="3.5">
                                                <polyline points="20 6 9 17 4 12"/>
                                            </svg>
                                        </div>
                                    {/if}

                                    <!-- Rendered Character Body -->
                                    <div class="skin-canvas-container">
                                        {#if bodyRenderMap[skin.id]}
                                            <img
                                                src={bodyRenderMap[skin.id]}
                                                alt={skin.name}
                                                class="skin-body-render"
                                            />
                                        {:else}
                                            <div class="rendering-placeholder">
                                                <Circle size="20" color="#6b7280" unit="px" />
                                            </div>
                                        {/if}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>

                <!-- Section 2: Default Skins -->
                <div class="skins-section">
                    <button
                        type="button"
                        class="section-header-btn"
                        on:click={() => (defaultSkinsOpen = !defaultSkinsOpen)}
                    >
                        <span class="chevron" class:collapsed={!defaultSkinsOpen}>⌃</span>
                        <span class="section-title">Default skins</span>
                    </button>

                    {#if defaultSkinsOpen}
                        <div class="cards-grid default-grid">
                            {#each DEFAULT_MINECRAFT_SKINS as def (def.id)}
                                {@const isDefActive = selectedSkin?.name?.toLowerCase() === def.name.toLowerCase() && selectedSkin?.active}
                                <div
                                    class="card skin-card"
                                    class:active={isDefActive}
                                    class:selected={selectedSkin?.name?.toLowerCase() === def.name.toLowerCase()}
                                    role="button"
                                    tabindex="0"
                                    on:click={() => handleEquipDefaultSkin(def)}
                                    on:keydown={(e) => (e.key === "Enter" || e.key === " ") && handleEquipDefaultSkin(def)}
                                >
                                    <!-- Active Checkmark Badge -->
                                    {#if isDefActive}
                                        <div class="active-checkmark-badge" title="Active skin">
                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#18191c" stroke-width="3.5">
                                                <polyline points="20 6 9 17 4 12"/>
                                            </svg>
                                        </div>
                                    {/if}

                                    <!-- Rendered Character Body -->
                                    <div class="skin-canvas-container">
                                        {#if defaultRenderMap[def.id]}
                                            <img
                                                src={defaultRenderMap[def.id]}
                                                alt={def.name}
                                                class="skin-body-render"
                                            />
                                        {:else}
                                            <div class="rendering-placeholder">
                                                <Circle size="20" color="#6b7280" unit="px" />
                                            </div>
                                        {/if}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    {/if}
</div>

<!-- Edit Skin Modal -->
{#if showEditModal && editSkinTarget}
    <div
        class="modal-backdrop"
        on:click={(e) => e.target === e.currentTarget && closeEditModal()}
        on:keydown={(e) => e.key === "Escape" && closeEditModal()}
        role="presentation"
    >
        <div class="modal-window" role="dialog" aria-modal="true">
            <div class="modal-header">
                <h3>Edit skin</h3>
                <button type="button" class="modal-close" on:click={closeEditModal}>✕</button>
            </div>

            {#if editModalError}
                <div class="modal-error">{editModalError}</div>
            {/if}

            <div class="modal-body">
                <div class="form-group">
                    <label for="edit-name">Skin Name</label>
                    <input
                        id="edit-name"
                        type="text"
                        class="modal-input"
                        bind:value={editSkinName}
                        placeholder="Skin Name"
                    />
                </div>

                <div class="form-group">
                    <span class="label-text">Model Type</span>
                    <div class="model-toggle-row">
                        <button
                            type="button"
                            class="model-choice"
                            class:active={editSkinModel === "classic"}
                            on:click={() => (editSkinModel = "classic")}
                        >
                            Classic (4px)
                        </button>
                        <button
                            type="button"
                            class="model-choice"
                            class:active={editSkinModel === "slim"}
                            on:click={() => (editSkinModel = "slim")}
                        >
                            Slim (3px)
                        </button>
                    </div>
                </div>

                <div class="modal-action-row">
                    <button
                        type="button"
                        class="btn-secondary"
                        on:click={handleExportFromEdit}
                    >
                        Export PNG
                    </button>
                    {#if skins.length > 1}
                        <button
                            type="button"
                            class="btn-danger"
                            on:click={handleDeleteFromEdit}
                        >
                            Delete Skin
                        </button>
                    {/if}
                </div>
            </div>

            <div class="modal-footer">
                <button type="button" class="btn-cancel" on:click={closeEditModal}>Cancel</button>
                <button
                    type="button"
                    class="btn-save"
                    disabled={editModalLoading || !editSkinName.trim()}
                    on:click={handleSaveEdit}
                >
                    {editModalLoading ? "Saving..." : "Save changes"}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    .skin-selector-root {
        width: 100%;
        height: 100%;
        box-sizing: border-box;
        color: #e5e7eb;
        user-select: none;
    }

    .loading-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 400px;
        gap: 14px;
        color: #9ca3af;
    }

    .error-banner {
        display: flex;
        align-items: center;
        justify-content: space-between;
        background: rgba(239, 68, 68, 0.15);
        border: 1px solid rgba(239, 68, 68, 0.35);
        border-radius: 8px;
        padding: 8px 14px;
        margin-bottom: 12px;
        font-size: 13px;
        color: #fca5a5;
    }

    .dismiss-btn {
        background: transparent;
        border: none;
        color: #fca5a5;
        cursor: pointer;
        font-size: 14px;
    }

    /* Modrinth 2-Column Layout */
    .modrinth-skin-layout {
        display: grid;
        grid-template-columns: 290px 1fr;
        gap: 24px;
        height: 100%;
        min-height: 520px;
    }

    /* Left Pane */
    .left-pane {
        display: flex;
        flex-direction: column;
        align-items: center;
        background: #17181c;
        border-radius: 10px;
        padding: 16px 14px 14px 14px;
        border: 1px solid #23252a;
    }

    .left-header {
        text-align: center;
        margin-bottom: 10px;
        width: 100%;
    }

    .skin-selector-title {
        font-size: 20px;
        font-weight: 700;
        margin: 0 0 6px 0;
        color: #ffffff;
        letter-spacing: -0.2px;
    }

    .player-tag {
        font-size: 14px;
        font-weight: 600;
        color: #9ba1ad;
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        letter-spacing: 0.2px;
    }

    /* 3D Model Viewport */
    .model-viewport {
        width: 100%;
        height: 310px;
        position: relative;
        border-radius: 8px;
        overflow: hidden;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .no-preview {
        color: #6b7280;
        font-size: 13px;
    }

    .drag-hint {
        display: flex;
        align-items: center;
        gap: 6px;
        color: #8b929e;
        font-size: 12px;
        margin: 6px 0 12px 0;
    }

    .edit-skin-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 7px;
        width: 100%;
        background: #25272c;
        color: #e5e7eb;
        border: 1px solid #363940;
        border-radius: 6px;
        padding: 9px 12px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .edit-skin-btn:hover:not(:disabled) {
        background: #2d3036;
        border-color: #484c56;
        color: #ffffff;
    }

    .edit-skin-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    /* Bottom Fetch Bar */
    .fetch-row {
        display: flex;
        gap: 8px;
        width: 100%;
        margin-top: auto;
        padding-top: 14px;
    }

    .fetch-input {
        flex: 1;
        background: #1e2025;
        border: 1px solid #2e3138;
        border-radius: 6px;
        padding: 7px 10px;
        font-size: 12px;
        color: #f3f4f6;
        outline: none;
        transition: border-color 0.15s;
    }

    .fetch-input:focus {
        border-color: #3b82f6;
    }

    .fetch-input::placeholder {
        color: #646a77;
    }

    .fetch-btn {
        display: flex;
        align-items: center;
        gap: 5px;
        background: #23252a;
        border: 1px solid #34373e;
        border-radius: 6px;
        color: #e5e7eb;
        padding: 7px 12px;
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s;
    }

    .fetch-btn:hover:not(:disabled) {
        background: #2d3036;
        border-color: #474b55;
    }

    .fetch-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    /* Right Pane */
    .right-pane {
        display: flex;
        flex-direction: column;
        gap: 20px;
        overflow-y: auto;
        padding-right: 6px;
    }

    .skins-section {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .section-header-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        background: transparent;
        border: none;
        color: #9ba1ad;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        padding: 0;
        text-align: left;
    }

    .section-header-btn:hover {
        color: #ffffff;
    }

    .chevron {
        font-size: 14px;
        transition: transform 0.2s ease;
        display: inline-block;
    }

    .chevron.collapsed {
        transform: rotate(180deg);
    }

    /* Cards Grid */
    .cards-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
        gap: 12px;
    }

    .default-grid {
        grid-template-columns: repeat(3, 1fr);
    }

    /* Card Item */
    .card {
        aspect-ratio: 1 / 1.25;
        background: #24262b;
        border-radius: 8px;
        position: relative;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1.5px solid transparent;
        transition: all 0.18s ease;
        overflow: hidden;
    }

    .card:hover {
        background: #2b2e34;
        transform: translateY(-2px);
    }

    .card.selected {
        border-color: #3b82f6;
    }

    .card.active {
        border-color: rgba(255, 255, 255, 0.4);
    }

    /* Add Card */
    .add-card {
        border: 1.5px dashed #3a3e47;
        background: #1c1e22;
        flex-direction: column;
        gap: 4px;
        color: #e5e7eb;
    }

    .add-card:hover, .add-card.drag-over {
        border-color: #10b981;
        background: rgba(16, 185, 129, 0.06);
    }

    .add-icon {
        font-size: 26px;
        font-weight: 300;
        line-height: 1;
        margin-bottom: 2px;
    }

    .add-label {
        font-size: 13px;
        font-weight: 600;
        color: #f3f4f6;
    }

    .add-subtext {
        font-size: 11px;
        color: #838a97;
    }

    /* Active Checkmark Badge */
    .active-checkmark-badge {
        position: absolute;
        top: 8px;
        right: 8px;
        width: 18px;
        height: 18px;
        background: #ffffff;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 5;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
    }

    .skin-canvas-container {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 8px;
        box-sizing: border-box;
    }

    .skin-body-render {
        max-width: 90%;
        max-height: 90%;
        object-fit: contain;
        image-rendering: pixelated;
        image-rendering: -moz-crisp-edges;
        image-rendering: crisp-edges;
        filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.35));
    }

    .rendering-placeholder {
        display: flex;
        align-items: center;
        justify-content: center;
    }

    /* Modal Styling */
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.65);
        backdrop-filter: blur(4px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
    }

    .modal-window {
        background: #1e2025;
        border: 1px solid #2f323a;
        border-radius: 10px;
        width: 380px;
        padding: 20px;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
    }

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 16px;
    }

    .modal-header h3 {
        margin: 0;
        font-size: 17px;
        color: #ffffff;
    }

    .modal-close {
        background: transparent;
        border: none;
        color: #8b929e;
        font-size: 16px;
        cursor: pointer;
    }

    .modal-close:hover {
        color: #ffffff;
    }

    .modal-error {
        background: rgba(239, 68, 68, 0.15);
        border: 1px solid rgba(239, 68, 68, 0.3);
        color: #fca5a5;
        padding: 8px 12px;
        border-radius: 6px;
        font-size: 12px;
        margin-bottom: 14px;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 14px;
    }

    .form-group label, .form-group .label-text {
        font-size: 12px;
        font-weight: 500;
        color: #9ba1ad;
    }

    .modal-input {
        background: #17181c;
        border: 1px solid #2e3138;
        border-radius: 6px;
        padding: 8px 12px;
        font-size: 13px;
        color: #f3f4f6;
        outline: none;
    }

    .modal-input:focus {
        border-color: #3b82f6;
    }

    .model-toggle-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px;
    }

    .model-choice {
        background: #17181c;
        border: 1px solid #2e3138;
        border-radius: 6px;
        padding: 8px 10px;
        color: #9ba1ad;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.15s;
    }

    .model-choice:hover {
        background: #202227;
        color: #ffffff;
    }

    .model-choice.active {
        background: #252830;
        border-color: #3b82f6;
        color: #3b82f6;
        font-weight: 600;
    }

    .modal-action-row {
        display: flex;
        gap: 8px;
        margin-top: 16px;
    }

    .btn-secondary {
        flex: 1;
        background: #252830;
        border: 1px solid #363a45;
        color: #e5e7eb;
        border-radius: 6px;
        padding: 7px 10px;
        font-size: 12px;
        cursor: pointer;
    }

    .btn-secondary:hover {
        background: #2f333e;
    }

    .btn-danger {
        flex: 1;
        background: rgba(239, 68, 68, 0.12);
        border: 1px solid rgba(239, 68, 68, 0.3);
        color: #fca5a5;
        border-radius: 6px;
        padding: 7px 10px;
        font-size: 12px;
        cursor: pointer;
    }

    .btn-danger:hover {
        background: rgba(239, 68, 68, 0.22);
    }

    .modal-footer {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 8px;
        margin-top: 20px;
        padding-top: 14px;
        border-top: 1px solid #282a30;
    }

    .btn-cancel {
        background: transparent;
        border: 1px solid #32353d;
        color: #9ba1ad;
        border-radius: 6px;
        padding: 7px 14px;
        font-size: 12px;
        cursor: pointer;
    }

    .btn-cancel:hover {
        color: #ffffff;
        border-color: #454954;
    }

    .btn-save {
        background: #10b981;
        border: none;
        color: #ffffff;
        border-radius: 6px;
        padding: 7px 16px;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s;
    }

    .btn-save:hover:not(:disabled) {
        background: #059669;
    }

    .btn-save:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
