<div class="status-bar">
    <slot />
</div>

<style>
    .status-bar {
        background: rgba(15, 23, 42, 0.65);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.08);
        height: 52px;
        display: flex;
        flex: 1;
        border-radius: 10px;
        margin: 0 24px;
        overflow: hidden;
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05);
    }
</style>
