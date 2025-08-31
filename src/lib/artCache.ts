import { LRUCache } from "lru-cache";
import { ResponseType } from "@tauri-apps/api/http";

import { emitAppLog } from "./commands.ts";
import RequestPool from "./requestPool.ts";

// NOTE: This should match or exceed the largest rendered image size (probably in <TrackInfo>)
const MAX_IMAGE_SIZE = 80;

const requestPool = new RequestPool(5);

/**
 * Scale the provided imageData to a maximum single dimension (width or height) of MAX_IMAGE_SIZE.
 *
 * @param imageData String of base64'd image:data
 */
const resizeImage = async (imageData: string) => {
    // Load the provided imageData into an Image to provide high-level image information
    let scalerImg = new Image();
    scalerImg.src = imageData;
    await scalerImg.decode();

    // Determine scaled width & height. Do not exceed MAX_IMAGE_SIZE. Allow for width > height and
    // height > width, while maintaining aspect ratio.
    let scaledWidth;
    let scaledHeight;

    if (scalerImg.naturalWidth > scalerImg.naturalHeight) {
        scaledWidth = Math.min(MAX_IMAGE_SIZE, scalerImg.naturalWidth);
        scaledHeight = Math.min(MAX_IMAGE_SIZE, scalerImg.naturalHeight) * (scalerImg.naturalHeight / scalerImg.naturalWidth);
    } else {
        scaledWidth = Math.min(MAX_IMAGE_SIZE, scalerImg.naturalWidth) * (scalerImg.naturalWidth / scalerImg.naturalHeight);
        scaledHeight = Math.min(MAX_IMAGE_SIZE, scalerImg.naturalHeight);
    }

    // Created a canvas with the scaled dimensions.
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");

    if (!ctx) {
        // This should not happen, but if we don't have a valid context then just return the
        // provided unscaled image data.
        return imageData;
    }

    canvas.width = scaledWidth;
    canvas.height = scaledHeight;

    // Draw source image into the scaled canvas. This will produce a scaled image.
    ctx.drawImage(scalerImg, 0, 0, scaledWidth, scaledHeight);

    // Convert the scaled image back into an image:data url. Force PNG.
    return canvas.toDataURL("image/png");
}

/**
 * Retrieve the provided key (an art URL, presumably pointing to an image), and return the key's
 * value (which is a base64-encoded data:image string).
 */
const artFetcher = async (key: string) => {
    // Wait up to 500ms before making the request. This is a *very* naive approach to
    // limiting the load on the art server if/when a bunch of art is all being displayed
    // for the first time. Ultimately this might need to become some sort of worker-limited
    // (say 10 max) queueing system.
    await new Promise(r => setTimeout(r, Math.random() * 500));

    let response;

    try {
        response = await requestPool.fetch(key, { method: "GET",  responseType: ResponseType.Binary });
    } catch (e) {
        // TODO: Consider how to deal with fetch errors
        await emitAppLog("warn", `Could not fetch art: ${e}`);

        return "";
    }

    // TODO: Improve
    //  https://stackoverflow.com/questions/9267899/arraybuffer-to-base64-encoded-string/42334410#42334410
    var base64ImageData = btoa(
        new Uint8Array(response.data)
            .reduce((data, byte) => data + String.fromCharCode(byte), '')
    );

    // Determine extension from url, fall back on jpeg
    let extension = "jpeg";

    const parsedUrl = new URL(key);
    const pathname = parsedUrl.pathname;
    const potentialExtension = pathname.split(".").pop();

    if (potentialExtension && potentialExtension !== pathname) {
        extension = potentialExtension;
    }

    // Build the data:image string
    const imgDataUnscaled = `data:image/${extension};base64,${base64ImageData}`;

    return await resizeImage(imgDataUnscaled);
}

const options = {
    max: 500, // 500 art items
    maxSize: 50_000_000, // 50MB
    sizeCalculation: (value: string, key: string) => key.length + value.length,
    allowStale: true,
    updateAgeOnGet: true,
    updateAgeOnHas: true,
    fetchMethod: artFetcher,
};

// Key is the URL to the art image, value is a "data:image" string of base64 image data
const artCache = new LRUCache<string, string>(options);

export default artCache;
