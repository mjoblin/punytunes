<script lang="ts">
    import { alwaysUseArtRustClient, osType } from "../../state.ts";

    import ArtBrowserClient from "./artClients/ArtBrowserClient.svelte";
    import ArtRustClient from "./artClients/ArtRustClient.svelte";

    export let url: string | null | undefined;
</script>

{#if $alwaysUseArtRustClient || $osType === "Windows_NT"}
    <ArtRustClient {url} {...$$restProps} />
{:else}
    <ArtBrowserClient {url} {...$$restProps} />
{/if}
