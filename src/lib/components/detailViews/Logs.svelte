<script lang="ts">
    import { afterUpdate } from "svelte";

    import type { AppLog } from "../../../types/generated/AppLog.ts";
    import pubSub from "../../pubSub.ts";
    import { appLogs, logLevelDisplay } from "../../state.ts";

    afterUpdate(() => {
        pubSub.publish("DetailViewUpdated");
    });

    const timestamp = (appLog: AppLog): string => {
        let when = new Date(appLog.when);
        return `${
            when.toLocaleTimeString([], { hour12: false, hour: "2-digit", minute: "2-digit", fractionalSecondDigits: 3 })
        }`;
    }

    $: appLogsToShow = $appLogs.filter((log) => {
        const visibleLevels = Object.keys($logLevelDisplay).filter(level => $logLevelDisplay[level]);
        return visibleLevels.includes(log.level);
    });
</script>

<div class="Logs">
    <div>
        {#if $appLogs.length <= 0}
            <div class="detail-data-empty">No logs received</div>
        {:else if appLogsToShow.length <= 0}
            <div class="detail-data-empty">No matching logs</div>
        {:else}
            <table>
                {#each appLogsToShow as appLog}
                    <tr
                        class:info={appLog.level === "info"}
                        class:warn={appLog.level === "warn"}
                        class:error={appLog.level === "error"}
                    >
                        <td class="when">{timestamp(appLog)}</td>
                        <td class="level">{appLog.level}</td>
                        <td>{appLog.message}</td>
                    </tr>
                {/each}
            </table>
        {/if}
    </div>
</div>

<style>
    .Logs {
        display: flex;
        flex-direction: column;
        font-size: 0.9em;

        & tr:hover {
            cursor: default;
        }
    }

    .when {
        width: 70px;
    }

    .level {
        width: 25px;
    }

    .info {
        color: var(--text-normal);
    }

    .warn {
        color: var(--attention-color-bright);
    }

    .error {
        color: var(--alert-color);
        font-weight: 500;
    }
</style>