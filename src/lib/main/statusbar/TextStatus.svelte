<script>
    export let text;
</script>

<div class="text-status">{text}</div>

<style>
    .text-status {
        color: white;
        display: flex;
        align-items: center;
        border-bottom: 2px solid transparent;
        border-image: linear-gradient(90deg, #3b82f6, #8b5cf6) 1;
        width: max-content;
        margin: 0 auto;
        font-weight: 500;
        font-size: 13px;
        letter-spacing: 0.01em;
    }
</style>