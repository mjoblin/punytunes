<script lang="ts">
    export let size: number = 20;
    export let color: string = "#3498db";
</script>

<svg
    class="spinner"
    width={size}
    height={size}
    viewBox="0 0 100 100"
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    stroke={color}
    stroke-width="10"
>
    <!-- underlying complete circle -->
    <path
        d="M90 50 A 40 40 0 1 1 90 49.9999"
        fill="none"
        stroke="var(--background-bright)"
        stroke-width="10"
    />

    <!-- 45 degree arc -->
    <path d="M10 50 A 40 40 0 0 1 50 10" />
</svg>

<style>
    .spinner {
        animation: spin 1.5s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
</style>
