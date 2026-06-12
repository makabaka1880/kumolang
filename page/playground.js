// ---- KumoLang WASM Playground ----

import initSync, { verify_kumolang, verify_kumolang_json } from '../pkg/kumolang.js';

const editor = document.getElementById('playground-editor');
const output = document.getElementById('playground-output');
const graphView = document.getElementById('graph-view');
const btnVerify = document.getElementById('btn-verify');
const btnReset = document.getElementById('btn-reset');
const sampleBtns = document.querySelectorAll('[data-sample]');

// Canonical samples
const samples = {
    basque: `(begin
    (new-mixture BaseMix)
    (new-mixture BakeMix)
    (prep c_cheese BaseMix)
    (prep c_sugar BaseMix)
    (pour c_bowl c_cheese c_sugar)
    (pour-out c_cake c_bowl BakeMix)
    (skip)
)`,

    simple: `(begin
    (new-mixture Batter)
    (prep flour Batter)
    (prep eggs Batter)
    (pour bowl flour eggs)
    (skip)
)`,

    error: `(begin
    (new-mixture Batter)
    (new-mixture Glaze)
    (prep flour Batter)
    (prep sugar Glaze)
    (pour bowl flour sugar)
    (skip)
)`,

    multi: `(begin
    (new-mixture Crust)
    (new-mixture Filling)
    (new-mixture Topping)
    (prep c_flour Crust)
    (prep c_butter Crust)
    (pour c_base c_flour c_butter)
    (prep c_cheese Filling)
    (prep c_cream Filling)
    (pour c_fill c_cheese c_cream)
    (pour-out c_cake c_base Crust)
    (pour-out c_cake c_fill Filling)
    (skip)
)`
};

// ---- UI helpers ----

function setOutput(state, text) {
    output.className = `playground-output ${state}`;
    output.textContent = text;
}

function setLoading(loading) {
    btnVerify.disabled = loading;
    btnVerify.textContent = loading ? 'Verifying…' : 'Verify';
}

function hideGraph() {
    graphView.classList.add('empty');
    graphView.innerHTML = '';
}

// ---- Graph Viewer ----

/** Parse node_type from JSON into a uniform shape. */
function parseNodeType(nt) {
    if (typeof nt === 'string') {
        return { kind: nt };            // "mixture"
    }
    if (nt && nt.ingredient !== undefined) {
        return { kind: 'ingredient', mixture: nt.ingredient.mixture };
    }
    if (nt && nt.container !== undefined) {
        return { kind: 'container', name: nt.container.name };
    }
    return { kind: 'unknown' };
}

/**
 * Layout the provenance DAG.
 *
 * Edge convention (from dag.rs): addEdge(from, to) means "from depends on to".
 * In the JSON: edges = { fromId: [toId, ...] }.
 *
 * We want arrows pointing FROM dependencies (toId) TO dependent (fromId),
 * matching the paper's tikz figure direction (source → derived).
 *
 * Returns { nodes: [{id, name, kind, x, y}], edges: [{sx, sy, ex, ey, sid, eid}] }.
 */
function layoutDag(graph) {
    const nodes = graph.nodes;
    const rawEdges = graph.edges;

    // Build node lookup
    const nodeById = {};
    for (const n of nodes) {
        const pt = parseNodeType(n.node_type);
        nodeById[n.id] = { id: n.id, name: n.name, kind: pt.kind };
    }

    // Build visual adjacency: visAdj[A] = [B, C] means arrows go A → B, A → C
    // (i.e. A is a dependency of B and C)
    const visAdj = {};
    const inDegree = {};
    for (const n of nodes) {
        visAdj[n.id] = [];
        inDegree[n.id] = 0;
    }

    for (const [fromId, toIds] of Object.entries(rawEdges)) {
        for (const toId of toIds) {
            // Arrow: toId (dependency) → fromId (dependent)
            if (visAdj[toId]) {
                visAdj[toId].push(fromId);
                inDegree[fromId] = (inDegree[fromId] || 0) + 1;
            }
        }
    }

    // Topological sort to assign layers (BFS)
    const layer = {};
    let queue = [];
    for (const n of nodes) {
        if (inDegree[n.id] === 0) {
            layer[n.id] = 0;
            queue.push(n.id);
        }
    }

    let maxLayer = 0;
    while (queue.length > 0) {
        const cur = queue.shift();
        for (const next of (visAdj[cur] || [])) {
            const candidate = layer[cur] + 1;
            if (layer[next] === undefined || candidate > layer[next]) {
                layer[next] = candidate;
                if (candidate > maxLayer) maxLayer = candidate;
            }
            inDegree[next]--;
            if (inDegree[next] === 0) {
                queue.push(next);
            }
        }
    }

    // Group nodes by layer
    const layerNodes = {};
    for (const n of nodes) {
        const l = layer[n.id] !== undefined ? layer[n.id] : 0;
        if (!layerNodes[l]) layerNodes[l] = [];
        layerNodes[l].push(nodeById[n.id]);
    }

    // Layout parameters
    const nodeW = 150, nodeH = 36;
    const layerGapX = 190, nodeGapY = 54;
    const paddingX = 30, paddingY = 24;

    // Position nodes
    for (let l = 0; l <= maxLayer; l++) {
        const group = layerNodes[l] || [];
        const totalH = group.length * nodeH + (group.length - 1) * nodeGapY;
        let startY = paddingY;
        // Center each layer vertically
        const canvasH = Math.max(totalH + paddingY * 2, 120);
        startY = (canvasH - totalH) / 2;

        for (let i = 0; i < group.length; i++) {
            group[i].x = paddingX + l * layerGapX;
            group[i].y = startY + i * (nodeH + nodeGapY);
        }
    }

    // Build positioned edges
    const posEdges = [];
    for (const [fromId, toIds] of Object.entries(rawEdges)) {
        for (const toId of toIds) {
            const src = nodeById[toId];  // dependency
            const dst = nodeById[fromId]; // dependent
            if (src && dst && src.x !== undefined && dst.x !== undefined) {
                posEdges.push({
                    sid: src.id, eid: dst.id,
                    sx: src.x + nodeW, sy: src.y + nodeH / 2,
                    ex: dst.x, ey: dst.y + nodeH / 2,
                });
            }
        }
    }

    // Compute SVG dimensions
    const svgW = paddingX + (maxLayer + 1) * layerGapX + nodeW + paddingX;
    // Find max layerCount to compute height
    let maxCount = 1;
    for (let l = 0; l <= maxLayer; l++) {
        maxCount = Math.max(maxCount, (layerNodes[l] || []).length);
    }
    const svgH = Math.max(maxCount, 1) * nodeH + (Math.max(maxCount - 1, 0)) * nodeGapY + paddingY * 2;

    return { positionedNodes: Object.values(nodeById), posEdges, svgW, svgH, nodeW, nodeH };
}

/** Render the provenance DAG as SVG inside #graph-view. */
function renderGraph(graph) {
    if (!graph || !graph.nodes || graph.nodes.length === 0) {
        hideGraph();
        return;
    }

    const { positionedNodes, posEdges, svgW, svgH, nodeW, nodeH } = layoutDag(graph);

    // Color map
    const fillMap = {
        mixture: '#dbe8fc',
        ingredient: '#d8f0d4',
        container: '#faf0dc',
    };
    const strokeMap = {
        mixture: '#8aabcf',
        ingredient: '#8bc28b',
        container: '#d4b87a',
    };
    const labelMap = {
        mixture: 'M',
        ingredient: 'I',
        container: 'C',
    };

    const arrowSize = 6;

    let svg = `<svg width="${svgW}" height="${svgH}" xmlns="http://www.w3.org/2000/svg">`;

    // Defs: arrowhead marker
    svg += `<defs>
    <marker id="arrowhead" markerWidth="${arrowSize}" markerHeight="${arrowSize}" refX="${arrowSize}" refY="${arrowSize / 2}" orient="auto-start-reverse">
      <polygon points="0 0, ${arrowSize} ${arrowSize / 2}, 0 ${arrowSize}" class="edge-arrow"/>
    </marker>
  </defs>`;

    // Edges
    for (const e of posEdges) {
        // Shorten line from end to not overlap arrowhead
        const dx = e.ex - e.sx;
        const dy = e.ey - e.sy;
        const len = Math.sqrt(dx * dx + dy * dy);
        if (len < 1) continue;
        const ux = dx / len, uy = dy / len;
        const endX = e.ex - ux * (arrowSize + 1);
        const endY = e.ey - uy * (arrowSize + 1);
        svg += `<line x1="${e.sx}" y1="${e.sy}" x2="${endX}" y2="${endY}" class="edge-line" marker-end="url(#arrowhead)"/>`;
    }

    // Nodes
    for (const n of positionedNodes) {
        if (n.x === undefined) continue;
        const kind = n.kind || 'unknown';
        const fill = fillMap[kind] || '#f0f0f0';
        const stroke = strokeMap[kind] || '#ccc';
        const prefix = labelMap[kind] || '?';
        // Truncate label if needed
        const label = n.name.length > 16 ? n.name.slice(0, 14) + '…' : n.name;
        svg += `<rect x="${n.x}" y="${n.y}" width="${nodeW}" height="${nodeH}" class="node-rect ${kind}" fill="${fill}" stroke="${stroke}"/>`;
        svg += `<text x="${n.x + nodeW / 2}" y="${n.y + nodeH / 2}" class="node-text">${prefix}: ${escapeXml(label)}</text>`;
    }

    // Legend
    const legendY = svgH - 14;
    const legendItems = [
        { kind: 'mixture', label: 'Mixture' },
        { kind: 'ingredient', label: 'Ingredient' },
        { kind: 'container', label: 'Container' },
    ];
    let legendX = 10;
    for (const item of legendItems) {
        const f = fillMap[item.kind];
        const s = strokeMap[item.kind];
        svg += `<rect x="${legendX}" y="${legendY - 8}" width="12" height="12" rx="2" class="node-rect ${item.kind}" fill="${f}" stroke="${s}"/>`;
        svg += `<text x="${legendX + 18}" y="${legendY}" class="graph-legend">${item.label}</text>`;
        legendX += 90;
    }

    svg += '</svg>';

    graphView.innerHTML = svg;
    graphView.classList.remove('empty');
}

function escapeXml(s) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// ---- Verification ----

async function doVerify() {
    const source = editor.value.trim();
    if (!source) {
        setOutput('error', '✗ No input provided. Write some KumoLang first!');
        hideGraph();
        return;
    }
    setOutput('loading', 'Verifying…');
    hideGraph();
    setLoading(true);

    // Yield so the loading state renders before the (synchronous) WASM call.
    await new Promise(r => setTimeout(r, 20));

    try {
        const resultText = verify_kumolang(source);
        const passed = resultText.includes('✓ Verification passed.');
        setOutput(passed ? 'success' : 'error', resultText.trimEnd());

        // Render provenance graph from structured output
        try {
            const jsonStr = verify_kumolang_json(source);
            const data = JSON.parse(jsonStr);
            if (data.passed && data.graph && data.graph.nodes && data.graph.nodes.length > 0) {
                renderGraph(data.graph);
            } else {
                hideGraph();
            }
        } catch (graphErr) {
            // Graph rendering is best-effort; don't break the text output
            console.warn('Graph rendering failed:', graphErr);
            hideGraph();
        }
    } catch (e) {
        setOutput('error', `✗ Internal error: ${e.message || e}`);
        hideGraph();
    } finally {
        setLoading(false);
    }
}

// ---- Event wiring ----

btnVerify.addEventListener('click', doVerify);

// Ctrl+Enter / Cmd+Enter to verify
editor.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        doVerify();
    }
});

btnReset.addEventListener('click', () => {
    editor.value = samples.basque;
    output.className = 'playground-output idle';
    output.innerHTML = 'Click <em>Verify</em> to check your recipe.';
    hideGraph();
    sampleBtns.forEach(b => b.classList.remove('active'));
    const basqueBtn = document.querySelector('[data-sample="basque"]');
    if (basqueBtn) basqueBtn.classList.add('active');
});

sampleBtns.forEach(btn => {
    btn.addEventListener('click', () => {
        const key = btn.dataset.sample;
        if (samples[key]) {
            editor.value = samples[key];
            sampleBtns.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            setOutput('idle', `Loaded "${btn.textContent}" sample.`);
            hideGraph();
        }
    });
});

// ---- Initialise WASM ----

async function initWasm() {
    try {
        const resp = await fetch('./pkg/kumolang_bg.wasm');
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        const buf = await resp.arrayBuffer();
        initSync(buf);
        setOutput('idle', 'WASM engine ready. Click <em>Verify</em> to check your recipe.');
        btnVerify.disabled = false;
        // Highlight the default Basque sample
        const basqueBtn = document.querySelector('[data-sample="basque"]');
        if (basqueBtn) basqueBtn.classList.add('active');
    } catch (e) {
        setOutput('error',
            `✗ Failed to load WASM engine: ${e.message || e}\n\n` +
            `Make sure you built the WASM package:\n` +
            `  wasm-pack build --target web\n\n` +
            `Then serve this directory over HTTP (opening index.html from disk\n` +
            `won't work because of ES module and fetch restrictions):\n` +
            `  python3 -m http.server 8080\n` +
            `  npx serve .\n` +
            `  # or any static file server`
        );
        btnVerify.disabled = true;
    }
}

btnVerify.disabled = true;
initWasm();
