<script lang="ts">
    import { afterUpdate, onDestroy, onMount } from "svelte";
    import { inview } from "svelte-inview";

    import type { QueueListItem } from "../../../types/generated/streammagic_payloads/QueueListItem.ts";
    import { playQueueId } from "../../commands.ts";
    import {
        activeSourceId,
        followCurrentQueueItem,
        isConnected,
        isEstablishingConnectionState,
        queueList,
        showDetailedQueue,
        webSocketClientStatus,
    } from "../../state.ts";
    import { prettyDuration } from "../../utils.ts";
    import pubSub from "../../pubSub.ts";
    import Art from "../dataDisplay/Art.svelte";
    import QueueSummary from "../features/queue/QueueSummary.svelte";
    import WarningMessage from "../dataDisplay/WarningMessage.svelte";

    // Unsubscribe functions to be invoked on destroy.
    let unsubscribeFns: (() => void)[] = [];

    let itemIds: (number | null)[] = []; // The current queue item ids (order matters)
    let scrollToCurrentItemAfterUpdate = false;
    let scrollToCurrentItemAfterAppResizeComplete = false;
    let hasShuffleStateChanged = false;

    // The current queue position.
    $: currentItemId = $queueList?.play_id;

    let imageSize = 25;
    let cssVarStyles = `--image-size:${imageSize}px;`;

    // Every queue item in the table gets a unique DOM id of "item_id_<item_id>". This is used to
    // enable auto-scroll.
    const DOM_ID_PREFIX = "item_id_";
    const domIdPrefix = new RegExp(`^${DOM_ID_PREFIX}`);

    // --------------------------------------------------------------------------------------------
    // Lifecycle functions

    // Whenever the Queue screen is activated, ensure the current item is at the top when follow
    // mode is enabled.
    onMount(() => {
        if ($followCurrentQueueItem) {
            scrollToCurrentItemAfterAppResizeComplete = true;
        }
    });

    afterUpdate(async () => {
        // The new view mode has finished rendering, so scroll to the item that was at the top
        // before the view mode was changed
        if (scrollToFirstVisibleItemAfterUpdate && firstItemInView) {
            scrollToQueueItem(firstItemInView);
        }

        if (scrollToCurrentItemAfterUpdate) {
            // The setTimeout call handles the case where we're here after coming back to the queue
            // screen from another screen. If we don't call setTimeout, then it seems that the table
            // rows are not quite ready to be scrolled to. Svelte's tick() also works, but for some
            // reason that was causing occasional flashing when the app was migrated to a system
            // tray application and the user clicked 'queue' while following was enabled.
            setTimeout(() => {
                scrollToCurrentItem();
            }, 0);

            scrollToCurrentItemAfterUpdate = false;
        }

        scrollToFirstVisibleItemAfterUpdate = false;
        pubSub.publish("DetailViewUpdated");
    });

    onDestroy(() => {
        unsubscribeFns.forEach((unsubscribeFn) => {
            unsubscribeFn();
        });
    });

    unsubscribeFns.push(
        pubSub.subscribe("AppResizeComplete", () => {
            if (scrollToCurrentItemAfterAppResizeComplete) {
                scrollToCurrentItem();
                scrollToCurrentItemAfterAppResizeComplete = false;
            }
        })
    );

    // When the tray app is opened, scroll to the current queue item if in follow mode
    // TODO: Would be preferred to have a reliable "app displayed and scroll will not work"
    //  announcement, rather than relying on a dodgy setTimeout of 100ms.
    unsubscribeFns.push(
        pubSub.subscribe("TrayAppOpened", () => {
            $followCurrentQueueItem && setTimeout(() => {
                scrollToCurrentItem();
            }, 100);
        })
    );

    // When a connection is made to the streamer, ensure that the queue is scrolled to the
    // current item. There's a bit of a race condition here where this assumes that the
    // current queue information (items and active id) is up to date.
    unsubscribeFns.push(
        webSocketClientStatus.subscribe((clientStatus) => {
            clientStatus.state === "Connected" && $followCurrentQueueItem && setTimeout(() => {
                scrollToCurrentItem();
            }, 100);
        })
    );

    // --------------------------------------------------------------------------------------------
    // Helper functions

    $: scrollOffset = $showDetailedQueue ? -1 : -1;

    const itemIdToDomId = (id: number | null | undefined) => `${DOM_ID_PREFIX}${id}`;

    // Take the given domId (for a queue item) and find the domId of the previous queue item.
    const domIdToPreviousDomId = (domId: string): string | undefined => {
        const items = $queueList?.items;

        if (!items || items.length <= 0) {
            return undefined;
        }

        const itemId = parseInt(domId.replace(domIdPrefix, ""), 10);
        const itemIndex = $queueList?.items?.findIndex((item) => item.id === itemId);

        if (typeof itemIndex === "undefined") {
            return undefined;
        }

        const prevItemIndex = Math.max(itemIndex - 1, 0);

        return `${DOM_ID_PREFIX}${items[prevItemIndex].id}`;
    };

    // Scroll the queue table so that the given item is shown at the top. This is used to ensure
    // that the item at the top of the view remains at the top when switching view modes.
    const scrollToQueueItem = ((item: QueueListItem | undefined) => {
        const targetDomId = item && itemIdToDomId(item.id);

        targetDomId && document.getElementById(targetDomId)?.scrollIntoView(
            { behavior: "smooth", block: "start" }
        );
    });

    // Scroll the queue table so that the current item is shown at the top. This is mostly used
    // when follow mode is enabled and the track changes, or the user returns to the Queue screen.
    //
    // Scrolling to the current item actually scrolls to the *previous* item, to ensure that the
    // previous entry in the table is shown. This is done to maintain some context for the current
    // item.
    //
    // Note: scrollIntoView() also works, but we use scrollTo() instead because scrollIntoView()
    //  was causing occasional 2 or 3-pixel vertical shifts of the entire app for reasons I
    //  couldn't track down. (Setting all parent elements to "overflow: hidden" didn't seem to
    //  help).
    const scrollToCurrentItem = () => {
        const targetDomId = currentItemId && domIdToPreviousDomId(itemIdToDomId(currentItemId));
        const targetElem = targetDomId && document.getElementById(targetDomId);

        if (targetElem) {
            const detailsContent = targetElem?.parentElement?.parentElement?.parentElement;

            if (targetElem && detailsContent) {
                detailsContent.scrollTo({ top: targetElem.offsetTop - scrollOffset, behavior: "smooth" });
            }
        }
    };

    // --------------------------------------------------------------------------------------------
    // Whenever the queue changes, check to see if the queue list has changed. If it has, *and*
    // we've already detected a change to the shuffle state, then we want to ensure that we scroll
    // to the current item after the DOM is updated.
    //
    // Sequence:
    //
    //  -> user clicks shuffle;
    //      -> streamer shuffles the queue items and announces a new item array
    //          -> this component renders the newly ordered queue
    //              -> the table is scrolled to the new position of the currently-playing item
    //
    // Goal: Minimize jarring "I lost my spot" behavior in the queue table when shuffle is clicked.
    $: {
        const newItemIdOrdering = $queueList?.items?.map((item) => item.id) || [];
        const newIdOrderingStringified = JSON.stringify(newItemIdOrdering);
        const priorIdOrderingStringified = JSON.stringify(itemIds);

        if (newIdOrderingStringified !== priorIdOrderingStringified) {
            itemIds = newItemIdOrdering;

            if (hasShuffleStateChanged) {
                // User has changed the shuffle mode, so ensure we scroll to the current item once
                // the DOM has rendered the new queue ordering.
                scrollToCurrentItemAfterUpdate = true;
                hasShuffleStateChanged = false;
            }
        }
    }

    unsubscribeFns.push(
        pubSub.subscribe("ShuffleModeSet", () => {
            hasShuffleStateChanged = true;
        })
    );

    // --------------------------------------------------------------------------------------------
    // Handle automatically scrolling the top currently-visible item into view when the user
    // switches between normal and detailed views.

    let itemDomIdsInView: Record<string, Record<string, boolean>> = { data: {} }; // Key: dom item id; Value: is it in view
    let firstItemInView: QueueListItem | undefined;
    let scrollToFirstVisibleItemAfterUpdate = false;

    unsubscribeFns.push(
        showDetailedQueue.subscribe(() => {
            // The user has switched view mode, so find what item is currently at the top
            firstItemInView = $queueList?.items?.find((item) => !!itemDomIdsInView.data[itemIdToDomId(item.id)]);
            scrollToFirstVisibleItemAfterUpdate = true;
        })
    );

    // --------------------------------------------------------------------------------------------
    // Handle automatically scrolling to the current item if we're in follow mode, or if the
    // "scroll to current" action was triggered.

    $: if (typeof currentItemId === "number" && $followCurrentQueueItem) {
        scrollToCurrentItem();
    }

    unsubscribeFns.push(
        pubSub.subscribe("ScrollToCurrentQueueItem", () => {
            scrollToCurrentItem();
        })
    );

    // --------------------------------------------------------------------------------------------

    $: activeId = $queueList?.play_id; // The id of the currently-active queue item
    $: items = $queueList?.items ?? [];
    $: isQueueActive = $activeSourceId === "MEDIA_PLAYER";

    // Determine what (if any) "this entry is active" class should be applied.
    $: activeClass = (entry: QueueListItem): string =>
        entry.id === activeId ?
            isQueueActive ?
                "active"
                : "active-not-playing"
            : "";
</script>

<!-- ------------------------------------------------------------------------------------------ -->

<div class="Queue" style={cssVarStyles}>
    {#if !$isConnected && !$isEstablishingConnectionState}
        <div class="detail-data-empty">Not connected to a StreamMagic streamer</div>
    {:else if items.length <= 0}
        <div class="detail-data-empty">Queue is empty</div>
    {:else}
        <table class="table">
            {#each items as item (item.id)}
                <tr
                    id={itemIdToDomId(item.id)}
                    class:detailed-view={showDetailedQueue}
                    on:click={() => item.id && playQueueId(item.id)}
                    use:inview={{ threshold: 0.5 }}
                    on:inview_change={(event) => {
                        const { inView, node } = event.detail;

                        // The "data" level of indirection is to prevent every set done here
                        // resulting in a svelte update (which in turn calls afterUpdate()).
                        const data = itemDomIdsInView.data;
                        data[node.id] = inView;
                    }}
                >
                    <td class={`position right-align ${activeClass(item)}`}>
                        {typeof item.position === "number" ? item.position + 1 : ""}
                    </td>

                    {#if $showDetailedQueue}
                        <td class={`art ${activeClass(item)}`}>
                            <Art url={item.metadata?.art_url} size={imageSize} />
                        </td>
                    {/if}

                    <td class={activeClass(item)}>
                        {#if $showDetailedQueue}
                            <div style="display: flex; flex-direction: column">
                                <div>{item.metadata?.title}</div>
                                <div class="details-sub">{item.metadata?.artist}</div>
                            </div>
                        {:else}
                            {item.metadata?.title}
                        {/if}
                    </td>

                    <td class={`duration right-align ${activeClass(item)}`}>
                        {prettyDuration(item.metadata?.duration ?? undefined, true)}
                    </td>
                </tr>
            {/each}
        </table>

        <QueueSummary />

        {#if !isQueueActive}
            <WarningMessage>
                queue is not currently being used for playback
            </WarningMessage>
        {/if}
    {/if}
</div>

<!-- ------------------------------------------------------------------------------------------ -->

<!-- NOTE: Some table style are defined in main styles.css -->

<style>
    .Queue {
        max-width: 100%;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .queue-inactive {
        display: flex;
        align-items: center;
        gap: 5px;
        padding: 5px;
        font-weight: bold;
    }

    .position {
        width: 15px;
        color: var(--text-dim);
    }

    .art {
        width: var(--image-size);
    }

    .duration {
        font-size: 0.9em;
    }

    tr.detailed-view {
        vertical-align: middle;
    }

    .right-align {
        text-align: right;
    }

    .details-sub {
        color: var(--text-dim);
        font-size: 0.8em;
    }

    .active {
        color: var(--text-min);
        font-weight: 500;

        & .details-sub {
            color: var(--background-dim);
        }
    }

    .active-not-playing {
        background-color: var(--background-bright);
    }
</style>
