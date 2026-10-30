<script>
    import { onMount, onDestroy, createEventDispatcher } from "svelte";

    export let skinUrl = "";
    export let model = "classic"; // "classic" | "slim"
    export let showOuterLayer = true;
    export let autoRotate = true;

    const dispatch = createEventDispatcher();

    let canvas;
    let gl;
    let animationFrameId;
    let texture = null;

    let yaw = 0.45;
    let pitch = 0.05;
    let distance = 56.0;
    let isDragging = false;
    let lastMouseX = 0;
    let lastMouseY = 0;
    let needsRender = true;

    let skinImg = null;
    let is64x32 = false;

    const vsSource = `
        attribute vec3 aPosition;
        attribute vec3 aNormal;
        attribute vec2 aTexCoord;

        uniform mat4 uMVP;
        uniform mat3 uNormalMatrix;

        varying vec2 vTexCoord;
        varying vec3 vNormal;

        void main() {
            gl_Position = uMVP * vec4(aPosition, 1.0);
            vTexCoord = aTexCoord;
            vNormal = uNormalMatrix * aNormal;
        }
    `;

    const fsSource = `
        precision mediump float;
        varying vec2 vTexCoord;
        varying vec3 vNormal;

        uniform sampler2D uSampler;

        void main() {
            vec4 texColor = texture2D(uSampler, vTexCoord);
            if (texColor.a < 0.1) {
                discard;
            }

            vec3 norm = normalize(vNormal);
            vec3 lightDir1 = normalize(vec3(0.5, 0.8, 1.0));
            vec3 lightDir2 = normalize(vec3(-0.5, 0.3, -0.6));
            float diff1 = max(dot(norm, lightDir1), 0.0);
            float diff2 = max(dot(norm, lightDir2), 0.0);
            float light = 0.70 + 0.22 * diff1 + 0.08 * diff2;

            gl_FragColor = vec4(texColor.rgb * light, texColor.a);
        }
    `;

    let programInfo = null;
    let buffers = null;

    function initShaders() {
        const vs = createShader(gl, gl.VERTEX_SHADER, vsSource);
        const fs = createShader(gl, gl.FRAGMENT_SHADER, fsSource);
        const prog = gl.createProgram();
        gl.attachShader(prog, vs);
        gl.attachShader(prog, fs);
        gl.linkProgram(prog);

        if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
            console.error("Shader link error:", gl.getProgramInfoLog(prog));
            return null;
        }

        return {
            program: prog,
            attribLocations: {
                position: gl.getAttribLocation(prog, "aPosition"),
                normal: gl.getAttribLocation(prog, "aNormal"),
                texCoord: gl.getAttribLocation(prog, "aTexCoord"),
            },
            uniformLocations: {
                mvp: gl.getUniformLocation(prog, "uMVP"),
                normalMatrix: gl.getUniformLocation(prog, "uNormalMatrix"),
                sampler: gl.getUniformLocation(prog, "uSampler"),
            },
        };
    }

    function createShader(gl, type, source) {
        const s = gl.createShader(type);
        gl.shaderSource(s, source);
        gl.compileShader(s);
        if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
            console.error("Shader compile error:", gl.getShaderInfoLog(s));
            gl.deleteShader(s);
            return null;
        }
        return s;
    }

    function addBox(vertices, normals, uvs, indices, box) {
        const { x0, x1, y0, y1, z0, z1, u, v, w, h, d, isLegacy } = box;
        const tw = 64.0;
        const th = isLegacy ? 32.0 : 64.0;

        function addQuad(p0, p1, p2, p3, norm, uv0, uv1, uv2, uv3) {
            const baseIdx = vertices.length / 3;

            vertices.push(
                p0[0], p0[1], p0[2],
                p1[0], p1[1], p1[2],
                p2[0], p2[1], p2[2],
                p3[0], p3[1], p3[2]
            );

            for (let i = 0; i < 4; i++) {
                normals.push(norm[0], norm[1], norm[2]);
            }

            uvs.push(
                uv0[0] / tw, uv0[1] / th,
                uv1[0] / tw, uv1[1] / th,
                uv2[0] / tw, uv2[1] / th,
                uv3[0] / tw, uv3[1] / th
            );

            indices.push(
                baseIdx, baseIdx + 1, baseIdx + 2,
                baseIdx, baseIdx + 2, baseIdx + 3
            );
        }

        // Top (+Y)
        addQuad(
            [x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0],
            [0, 1, 0],
            [u + d, v + d], [u + d + w, v + d], [u + d + w, v], [u + d, v]
        );

        // Bottom (-Y)
        addQuad(
            [x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1],
            [0, -1, 0],
            [u + d + w, v], [u + d + w + w, v], [u + d + w + w, v + d], [u + d + w, v + d]
        );

        // Right (+X)
        addQuad(
            [x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1],
            [1, 0, 0],
            [u, v + d + h], [u + d, v + d + h], [u + d, v + d], [u, v + d]
        );

        // Front (+Z)
        addQuad(
            [x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1],
            [0, 0, 1],
            [u + d, v + d + h], [u + d + w, v + d + h], [u + d + w, v + d], [u + d, v + d]
        );

        // Left (-X)
        addQuad(
            [x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0],
            [-1, 0, 0],
            [u + d + w, v + d + h], [u + d + w + d, v + d + h], [u + d + w + d, v + d], [u + d + w, v + d]
        );

        // Back (-Z)
        addQuad(
            [x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0],
            [0, 0, -1],
            [u + d + w + d, v + d + h], [u + d + w + d + w, v + d + h], [u + d + w + d + w, v + d], [u + d + w + d, v + d]
        );
    }

    function buildGeometry(isSlim, includeOuter) {
        const vertices = [];
        const normals = [];
        const uvs = [];
        const indices = [];

        const armW = isSlim ? 3.0 : 4.0;
        const armTexW = isSlim ? 3 : 4;
        const legacy = is64x32;

        // --- BASE LAYER ---
        // Head (8x8x8)
        addBox(vertices, normals, uvs, indices, {
            x0: -4, x1: 4, y0: 8, y1: 16, z0: -4, z1: 4,
            u: 0, v: 0, w: 8, h: 8, d: 8, isLegacy: legacy
        });

        // Torso (8x12x4)
        addBox(vertices, normals, uvs, indices, {
            x0: -4, x1: 4, y0: -4, y1: 8, z0: -2, z1: 2,
            u: 16, v: 16, w: 8, h: 12, d: 4, isLegacy: legacy
        });

        // Right Arm (armW x 12 x 4)
        addBox(vertices, normals, uvs, indices, {
            x0: -4 - armW, x1: -4, y0: -4, y1: 8, z0: -2, z1: 2,
            u: 40, v: 16, w: armTexW, h: 12, d: 4, isLegacy: legacy
        });

        // Left Arm (armW x 12 x 4)
        if (legacy) {
            // Mirror right arm
            addBox(vertices, normals, uvs, indices, {
                x0: 4, x1: 4 + armW, y0: -4, y1: 8, z0: -2, z1: 2,
                u: 40, v: 16, w: armTexW, h: 12, d: 4, isLegacy: true
            });
        } else {
            addBox(vertices, normals, uvs, indices, {
                x0: 4, x1: 4 + armW, y0: -4, y1: 8, z0: -2, z1: 2,
                u: 32, v: 48, w: armTexW, h: 12, d: 4, isLegacy: false
            });
        }

        // Right Leg (4x12x4)
        addBox(vertices, normals, uvs, indices, {
            x0: -4, x1: 0, y0: -16, y1: -4, z0: -2, z1: 2,
            u: 0, v: 16, w: 4, h: 12, d: 4, isLegacy: legacy
        });

        // Left Leg (4x12x4)
        if (legacy) {
            // Mirror right leg
            addBox(vertices, normals, uvs, indices, {
                x0: 0, x1: 4, y0: -16, y1: -4, z0: -2, z1: 2,
                u: 0, v: 16, w: 4, h: 12, d: 4, isLegacy: true
            });
        } else {
            addBox(vertices, normals, uvs, indices, {
                x0: 0, x1: 4, y0: -16, y1: -4, z0: -2, z1: 2,
                u: 16, v: 48, w: 4, h: 12, d: 4, isLegacy: false
            });
        }

        // --- OUTER LAYER (Hat, Jacket, Sleeves, Pants) ---
        if (includeOuter) {
            const o = 0.45; // layer offset

            // Hat
            addBox(vertices, normals, uvs, indices, {
                x0: -4 - o, x1: 4 + o, y0: 8 - o, y1: 16 + o, z0: -4 - o, z1: 4 + o,
                u: 32, v: 0, w: 8, h: 8, d: 8, isLegacy: legacy
            });

            if (!legacy) {
                // Jacket
                addBox(vertices, normals, uvs, indices, {
                    x0: -4 - o, x1: 4 + o, y0: -4 - o, y1: 8 + o, z0: -2 - o, z1: 2 + o,
                    u: 16, v: 32, w: 8, h: 12, d: 4, isLegacy: false
                });

                // Right Sleeve
                addBox(vertices, normals, uvs, indices, {
                    x0: -4 - armW - o, x1: -4 + o, y0: -4 - o, y1: 8 + o, z0: -2 - o, z1: 2 + o,
                    u: 40, v: 32, w: armTexW, h: 12, d: 4, isLegacy: false
                });

                // Left Sleeve
                addBox(vertices, normals, uvs, indices, {
                    x0: 4 - o, x1: 4 + armW + o, y0: -4 - o, y1: 8 + o, z0: -2 - o, z1: 2 + o,
                    u: 48, v: 48, w: armTexW, h: 12, d: 4, isLegacy: false
                });

                // Right Pant
                addBox(vertices, normals, uvs, indices, {
                    x0: -4 - o, x1: 0 + o, y0: -16 - o, y1: -4 + o, z0: -2 - o, z1: 2 + o,
                    u: 0, v: 32, w: 4, h: 12, d: 4, isLegacy: false
                });

                // Left Pant
                addBox(vertices, normals, uvs, indices, {
                    x0: 0 - o, x1: 4 + o, y0: -16 - o, y1: -4 + o, z0: -2 - o, z1: 2 + o,
                    u: 0, v: 48, w: 4, h: 12, d: 4, isLegacy: false
                });
            }
        }

        return {
            pos: new Float32Array(vertices),
            norm: new Float32Array(normals),
            uv: new Float32Array(uvs),
            idx: new Uint16Array(indices),
            count: indices.length,
        };
    }

    function updateBuffers() {
        if (!gl || !programInfo) return;

        const geom = buildGeometry(model === "slim", showOuterLayer);

        if (!buffers) {
            buffers = {
                pos: gl.createBuffer(),
                norm: gl.createBuffer(),
                uv: gl.createBuffer(),
                idx: gl.createBuffer(),
                count: geom.count,
            };
        }

        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.pos);
        gl.bufferData(gl.ARRAY_BUFFER, geom.pos, gl.STATIC_DRAW);

        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.norm);
        gl.bufferData(gl.ARRAY_BUFFER, geom.norm, gl.STATIC_DRAW);

        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.uv);
        gl.bufferData(gl.ARRAY_BUFFER, geom.uv, gl.STATIC_DRAW);

        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buffers.idx);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, geom.idx, gl.STATIC_DRAW);

        buffers.count = geom.count;
    }

    function loadTexture(url) {
        if (!gl) return;
        const img = new Image();
        img.crossOrigin = "anonymous";
        img.onload = () => {
            skinImg = img;
            is64x32 = img.height === 32;

            if (!texture) {
                texture = gl.createTexture();
            }
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, img);

            // NEAREST filtering for crisp Minecraft pixel art
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

            updateBuffers();
        };
        img.src = url;
    }

    // Matrix math helpers
    function mat4Perspective(out, fovy, aspect, near, far) {
        const f = 1.0 / Math.tan(fovy / 2);
        out[0] = f / aspect; out[1] = 0; out[2] = 0; out[3] = 0;
        out[4] = 0; out[5] = f; out[6] = 0; out[7] = 0;
        out[8] = 0; out[9] = 0; out[10] = (far + near) / (near - far); out[11] = -1;
        out[12] = 0; out[13] = 0; out[14] = (2 * far * near) / (near - far); out[15] = 0;
        return out;
    }

    function mat4Multiply(out, a, b) {
        const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
        const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
        const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
        const a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];

        let b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
        out[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = b[4]; b1 = b[5]; b2 = b[6]; b3 = b[7];
        out[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = b[8]; b1 = b[9]; b2 = b[10]; b3 = b[11];
        out[8] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out[9] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = b[12]; b1 = b[13]; b2 = b[14]; b3 = b[15];
        out[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
        return out;
    }

    function render() {
        if (!gl || !programInfo || !buffers || !texture) {
            animationFrameId = requestAnimationFrame(render);
            return;
        }

        if (autoRotate && !isDragging) {
            yaw += 0.008;
            needsRender = true;
        }

        const width = canvas.clientWidth;
        const height = canvas.clientHeight;
        if (canvas.width !== width || canvas.height !== height) {
            canvas.width = width;
            canvas.height = height;
            gl.viewport(0, 0, width, height);
            needsRender = true;
        }

        if (!needsRender && !isDragging && !autoRotate) {
            animationFrameId = requestAnimationFrame(render);
            return;
        }
        needsRender = false;

        gl.clearColor(0.04, 0.06, 0.09, 0.0);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        gl.enable(gl.DEPTH_TEST);
        gl.depthFunc(gl.LEQUAL);
        gl.disable(gl.CULL_FACE);

        gl.useProgram(programInfo.program);

        const proj = new Float32Array(16);
        const fovy = 35 * Math.PI / 180;
        const aspect = (width && height) ? width / height : 1.0;
        mat4Perspective(proj, fovy, aspect, 0.5, 500.0);

        // Orbit camera looking at target (0, 0, 0)
        const cosY = Math.cos(yaw);
        const sinY = Math.sin(yaw);
        const cosP = Math.cos(pitch);
        const sinP = Math.sin(pitch);

        const targetY = 0.0;

        const view = new Float32Array([
            cosY,         -sinP * sinY,                 sinY * cosP,                0,
            0,             cosP,                        sinP,                       0,
            -sinY,        -sinP * cosY,                 cosP * cosY,                0,
            0,            -cosP * targetY,             -sinP * targetY - distance,  1
        ]);

        const mvp = new Float32Array(16);
        mat4Multiply(mvp, proj, view);
        gl.uniformMatrix4fv(programInfo.uniformLocations.mvp, false, mvp);

        const normMat = new Float32Array([
            view[0], view[1], view[2],
            view[4], view[5], view[6],
            view[8], view[9], view[10],
        ]);
        gl.uniformMatrix3fv(programInfo.uniformLocations.normalMatrix, false, normMat);

        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.uniform1i(programInfo.uniformLocations.sampler, 0);

        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.pos);
        gl.enableVertexAttribArray(programInfo.attribLocations.position);
        gl.vertexAttribPointer(programInfo.attribLocations.position, 3, gl.FLOAT, false, 0, 0);

        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.norm);
        gl.enableVertexAttribArray(programInfo.attribLocations.normal);
        gl.vertexAttribPointer(programInfo.attribLocations.normal, 3, gl.FLOAT, false, 0, 0);

        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.uv);
        gl.enableVertexAttribArray(programInfo.attribLocations.texCoord);
        gl.vertexAttribPointer(programInfo.attribLocations.texCoord, 2, gl.FLOAT, false, 0, 0);

        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buffers.idx);
        gl.drawElements(gl.TRIANGLES, buffers.count, gl.UNSIGNED_SHORT, 0);

        animationFrameId = requestAnimationFrame(render);
    }

    function onMouseDown(e) {
        isDragging = true;
        lastMouseX = e.clientX;
        lastMouseY = e.clientY;
        needsRender = true;
    }

    function onMouseMove(e) {
        if (!isDragging) return;
        const dx = e.clientX - lastMouseX;
        const dy = e.clientY - lastMouseY;
        lastMouseX = e.clientX;
        lastMouseY = e.clientY;

        yaw += dx * 0.01;
        pitch = Math.max(-1.4, Math.min(1.4, pitch + dy * 0.01));
        needsRender = true;
    }

    function onMouseUp() {
        isDragging = false;
        needsRender = true;
    }

    function onWheel(e) {
        e.preventDefault();
        distance = Math.max(35.0, Math.min(100.0, distance + e.deltaY * 0.05));
        needsRender = true;
    }

    export function resetCamera() {
        yaw = 0.45;
        pitch = 0.05;
        distance = 56.0;
        needsRender = true;
    }

    $: if (gl && model) {
        updateBuffers();
        needsRender = true;
    }

    $: if (gl && showOuterLayer !== undefined) {
        updateBuffers();
        needsRender = true;
    }

    $: if (skinUrl) {
        loadTexture(skinUrl);
    }

    onMount(() => {
        gl = canvas.getContext("webgl", { alpha: true, antialias: true, depth: true });
        if (!gl) {
            console.error("WebGL not supported");
            return;
        }

        programInfo = initShaders();
        updateBuffers();

        // Initialize placeholder texture so render() immediately functions
        if (!texture) {
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array([80, 85, 95, 255]));
        }

        if (skinUrl) {
            loadTexture(skinUrl);
        }

        render();

        window.addEventListener("mouseup", onMouseUp);
        window.addEventListener("mousemove", onMouseMove);
    });

    onDestroy(() => {
        if (animationFrameId) {
            cancelAnimationFrame(animationFrameId);
        }
        window.removeEventListener("mouseup", onMouseUp);
        window.removeEventListener("mousemove", onMouseMove);
    });
</script>

<div class="skin-3d-container">
    <canvas
        bind:this={canvas}
        on:mousedown={onMouseDown}
        on:wheel={onWheel}
    ></canvas>

    <div class="controls-overlay">
        <button
            type="button"
            class="control-btn"
            class:active={autoRotate}
            title={autoRotate ? "Pause rotation" : "Auto-rotate"}
            on:click={() => autoRotate = !autoRotate}
        >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/>
            </svg>
            <span>{autoRotate ? "Rotate: On" : "Rotate: Off"}</span>
        </button>

        <button
            type="button"
            class="control-btn"
            title="Reset camera view"
            on:click={resetCamera}
        >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <path d="m15 9-6 6M9 9l6 6"/>
            </svg>
            <span>Reset</span>
        </button>

        <button
            type="button"
            class="control-btn"
            class:active={showOuterLayer}
            title={showOuterLayer ? "Hide outer jacket/hat layer" : "Show outer jacket/hat layer"}
            on:click={() => showOuterLayer = !showOuterLayer}
        >
            <span>{showOuterLayer ? "Layers: All" : "Layers: Base"}</span>
        </button>

        <button
            type="button"
            class="control-btn"
            title="Toggle arm model: Classic (4px) or Slim (3px)"
            on:click={() => {
                model = (model === "slim") ? "classic" : "slim";
                dispatch("modelChange", model);
                needsRender = true;
            }}
        >
            <span>{model === "slim" ? "Slim (3px)" : "Classic (4px)"}</span>
        </button>
    </div>
</div>

<style>
    .skin-3d-container {
        position: relative;
        width: 100%;
        height: 100%;
        min-height: 260px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: radial-gradient(circle at center, rgba(30, 41, 59, 0.4) 0%, rgba(15, 23, 42, 0.7) 100%);
        border-radius: 8px;
        overflow: hidden;
        border: 1px solid rgba(255, 255, 255, 0.08);
    }

    canvas {
        width: 100%;
        height: 100%;
        cursor: grab;
        display: block;
    }

    canvas:active {
        cursor: grabbing;
    }

    .controls-overlay {
        position: absolute;
        bottom: 12px;
        left: 50%;
        transform: translateX(-50%);
        display: flex;
        gap: 8px;
        background: rgba(10, 15, 25, 0.75);
        backdrop-filter: blur(8px);
        padding: 5px 8px;
        border-radius: 20px;
        border: 1px solid rgba(255, 255, 255, 0.12);
        z-index: 5;
    }

    .control-btn {
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.7);
        font-size: 11px;
        font-weight: 500;
        padding: 4px 10px;
        border-radius: 14px;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 5px;
        transition: all 0.15s ease;
    }

    .control-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: white;
    }

    .control-btn.active {
        background: rgba(59, 130, 246, 0.25);
        color: #60a5fa;
    }
</style>

