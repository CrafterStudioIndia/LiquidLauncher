import { writable } from "svelte/store";

export const activeSkin = writable(null);

export const DEFAULT_MINECRAFT_SKINS = [
    { id: "default_steve", name: "Steve", model: "classic", path: "/img/default_skins/wide/steve.png" },
    { id: "default_alex", name: "Alex", model: "slim", path: "/img/default_skins/slim/alex.png" },
    { id: "default_noor", name: "Noor", model: "slim", path: "/img/default_skins/slim/noor.png" },
    { id: "default_sunny", name: "Sunny", model: "slim", path: "/img/default_skins/slim/sunny.png" },
    { id: "default_ari", name: "Ari", model: "slim", path: "/img/default_skins/slim/ari.png" },
    { id: "default_zuri", name: "Zuri", model: "classic", path: "/img/default_skins/wide/zuri.png" },
    { id: "default_makena", name: "Makena", model: "slim", path: "/img/default_skins/slim/makena.png" },
    { id: "default_kai", name: "Kai", model: "slim", path: "/img/default_skins/slim/kai.png" },
    { id: "default_efe", name: "Efe", model: "slim", path: "/img/default_skins/slim/efe.png" },
];

/**
 * Extracts the 2D face avatar from a Minecraft skin image.
 * Combines base head and hat layer.
 * @param {string} skinDataUrl
 * @param {number} size
 * @returns {Promise<string>}
 */
export async function getSkinHeadDataUrl(skinDataUrl, size = 64) {
    if (!skinDataUrl) return "img/steve.png";

    return new Promise((resolve) => {
        const img = new Image();
        img.crossOrigin = "anonymous";
        img.onload = () => {
            const canvas = document.createElement("canvas");
            canvas.width = size;
            canvas.height = size;
            const ctx = canvas.getContext("2d");
            if (!ctx) {
                resolve("img/steve.png");
                return;
            }

            ctx.imageSmoothingEnabled = false;

            // Base head front: x=8, y=8, w=8, h=8
            ctx.drawImage(img, 8, 8, 8, 8, 0, 0, size, size);

            // Hat overlay front: x=40, y=8, w=8, h=8
            ctx.drawImage(img, 40, 8, 8, 8, 0, 0, size, size);

            resolve(canvas.toDataURL("image/png"));
        };
        img.onerror = () => resolve("img/steve.png");
        img.src = skinDataUrl;
    });
}

/**
 * Renders a crisp 2D full-body front preview of a Minecraft character,
 * matching Modrinth App's skin card display.
 * @param {string} skinDataUrl
 * @param {"classic" | "slim"} model
 * @param {number} width
 * @param {number} height
 * @returns {Promise<string>}
 */
export async function getSkinFullBodyDataUrl(skinDataUrl, model = "classic", width = 140, height = 180) {
    if (!skinDataUrl) return null;

    return new Promise((resolve) => {
        const img = new Image();
        img.crossOrigin = "anonymous";
        img.onload = () => {
            const canvas = document.createElement("canvas");
            canvas.width = width;
            canvas.height = height;
            const ctx = canvas.getContext("2d");
            if (!ctx) return resolve(null);

            ctx.imageSmoothingEnabled = false;

            const isSlim = model === "slim";
            const armW = isSlim ? 3 : 4;
            const is64x64 = img.height >= 64;

            // Character width in skin pixels: 16 (classic) or 14 (slim)
            // Character height in skin pixels: 32 (head: 8, torso: 12, legs: 12)
            const s = Math.min(Math.floor((width - 16) / 16), Math.floor((height - 16) / 32));
            const totalW = 16 * s;
            const totalH = 32 * s;
            const offX = Math.floor((width - totalW) / 2);
            const offY = Math.floor((height - totalH) / 2);

            const rArmOffX = isSlim ? offX + s : offX;

            // 1. Base Body Parts
            // Head: (8, 8, 8, 8) -> (offX + 4*s, offY, 8*s, 8*s)
            ctx.drawImage(img, 8, 8, 8, 8, offX + 4 * s, offY, 8 * s, 8 * s);

            // Torso: (20, 20, 8, 12) -> (offX + 4*s, offY + 8*s, 8*s, 12*s)
            ctx.drawImage(img, 20, 20, 8, 12, offX + 4 * s, offY + 8 * s, 8 * s, 12 * s);

            // Right Arm (viewer's left): (44, 20, armW, 12)
            ctx.drawImage(img, 44, 20, armW, 12, rArmOffX, offY + 8 * s, armW * s, 12 * s);

            // Left Arm (viewer's right): (36, 52, armW, 12) or mirrored right arm
            if (is64x64) {
                ctx.drawImage(img, 36, 52, armW, 12, offX + 12 * s, offY + 8 * s, armW * s, 12 * s);
            } else {
                ctx.save();
                ctx.translate(offX + 12 * s + armW * s, offY + 8 * s);
                ctx.scale(-1, 1);
                ctx.drawImage(img, 44, 20, armW, 12, 0, 0, armW * s, 12 * s);
                ctx.restore();
            }

            // Right Leg: (4, 20, 4, 12) -> (offX + 4*s, offY + 20*s, 4*s, 12*s)
            ctx.drawImage(img, 4, 20, 4, 12, offX + 4 * s, offY + 20 * s, 4 * s, 12 * s);

            // Left Leg: (20, 52, 4, 12) or mirrored right leg
            if (is64x64) {
                ctx.drawImage(img, 20, 52, 4, 12, offX + 8 * s, offY + 20 * s, 4 * s, 12 * s);
            } else {
                ctx.save();
                ctx.translate(offX + 8 * s + 4 * s, offY + 20 * s);
                ctx.scale(-1, 1);
                ctx.drawImage(img, 4, 20, 4, 12, 0, 0, 4 * s, 12 * s);
                ctx.restore();
            }

            // 2. Outer Layers (Hat, Jacket, Sleeves, Pants)
            // Hat: (40, 8, 8, 8)
            ctx.drawImage(img, 40, 8, 8, 8, offX + 4 * s, offY, 8 * s, 8 * s);

            if (is64x64) {
                // Jacket: (20, 36, 8, 12)
                ctx.drawImage(img, 20, 36, 8, 12, offX + 4 * s, offY + 8 * s, 8 * s, 12 * s);
                // Right Sleeve: (44, 36, armW, 12)
                ctx.drawImage(img, 44, 36, armW, 12, rArmOffX, offY + 8 * s, armW * s, 12 * s);
                // Left Sleeve: (52, 52, armW, 12)
                ctx.drawImage(img, 52, 52, armW, 12, offX + 12 * s, offY + 8 * s, armW * s, 12 * s);
                // Right Pant: (4, 36, 4, 12)
                ctx.drawImage(img, 4, 36, 4, 12, offX + 4 * s, offY + 20 * s, 4 * s, 12 * s);
                // Left Pant: (4, 52, 4, 12)
                ctx.drawImage(img, 4, 52, 4, 12, offX + 8 * s, offY + 20 * s, 4 * s, 12 * s);
            }

            resolve(canvas.toDataURL("image/png"));
        };
        img.onerror = () => resolve(null);
        img.src = skinDataUrl;
    });
}
