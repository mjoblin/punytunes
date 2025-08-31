import type { FetchOptions, Response } from "@tauri-apps/api/http";
import { fetch } from "@tauri-apps/api/http";

/**
 * An HTTP/fetch pool.
 *
 * Limits concurrent requests to maxConcurrentRequests. Uses Tauri's "fetch" (which in turn uses
 * the Rust HTTP client) to perform the requests.
 *
 * Call fetch(url, options) to make a request. The returned promise will not resolve until the
 * request has been processed, which might need to wait until prior requests complete (depending on
 * the size of maxConcurrentRequests). The returned promise will resolve to the fetched Result.
 */
class RequestPool {
    private maxConcurrentRequests: number;
    private currentRequestsCount: number;
    private requestQueue: {
        url: string;
        options: FetchOptions;
        resolve: (value?: Response<unknown>) => void;
        reject: (reason?: any) => void;
    }[];

    constructor(maxConcurrentRequests: number) {
        this.maxConcurrentRequests = maxConcurrentRequests;
        this.currentRequestsCount = 0;
        this.requestQueue = [];
    }

    async fetch(url: string, options: FetchOptions): Promise<Response<unknown>> {
        if (this.currentRequestsCount < this.maxConcurrentRequests) {
            this.currentRequestsCount++;

            try {
                return await this.doFetch(url, options);
            } finally {
                this.currentRequestsCount--;
                this.processQueue();
            }
        } else {
            // maxConcurrentRequests reached; queue the request for later processing
            return new Promise((resolve, reject) => {
                this.requestQueue.push({ url, options, resolve, reject });
            });
        }
    }

    private processQueue(): void {
        if (
            this.currentRequestsCount < this.maxConcurrentRequests &&
            this.requestQueue.length > 0
        ) {
            const nextRequest = this.requestQueue.shift();

            if (!nextRequest) {
                // This is here mostly to keep TypeScript happy; the "length > 0" should be enough
                // to prevent undefined requests.
                return;
            }

            this.fetch(nextRequest.url, nextRequest.options)
                .then(nextRequest.resolve)
                .catch(nextRequest.reject);
        }
    }

    private async doFetch(url: string, options: FetchOptions): Promise<Response<unknown>> {
        return await fetch(url, options);
    }
}

export default RequestPool;
