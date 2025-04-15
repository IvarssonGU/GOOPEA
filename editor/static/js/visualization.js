let visualization_containter = document.getElementById("visualization");

const width = 500;
const height = 500;

let graphical_nodes = []

// Create the SVG container.
const svg = d3.create("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("style", "max-width: 100%; height: auto;");

const simulation = d3.forceSimulation()
    .force("charge", d3.forceManyBody())
    .force("x", d3.forceX(width / 2))
    .force("y", d3.forceY(height / 2))
    .on("tick", ticked);

function ticked() {
    node
        .attr("cx", d => d.x)
        .attr("cy", d => d.y);
}

let node = svg.append("g")
    .attr("stroke", "#fff")
    .attr("stroke-width", 1.5)
    .selectAll("circle");

function dragstarted(event) {
    if (!event.active) simulation.alphaTarget(0.3).restart();
    event.subject.fx = event.subject.x;
    event.subject.fy = event.subject.y;
}

    // Update the subject (dragged node) position during drag.
function dragged(event) {
    event.subject.fx = event.x;
    event.subject.fy = event.y;
}

// Restore the target alpha so the simulation cools after dragging ends.
// Unfix the subject position now that it’s no longer being dragged.
function dragended(event) {
    if (!event.active) simulation.alphaTarget(0);
    event.subject.fx = null;
    event.subject.fy = null;
}

visualization_containter.append(svg.node())

function update_visualization() {
    const mem = wasm_bindgen.take_interpreter_memory_snapshot()
    console.log(mem)

    while (graphical_nodes.length < mem.heap.length) {
        graphical_nodes.push({ x: 0, y: height / 2 })
    }

    while (graphical_nodes.length > mem.heap.length) {
        graphical_nodes.pop()
    }

    node = node.data(graphical_nodes);
    let was_changed = !node.enter().empty();

    node.exit()
        .transition()
            .attr("fill", "red")
        .transition()
            .attr("r", 0)
            .remove()

    node = node.enter().append("circle")
        .call(d3.drag()
            .on("start", dragstarted)
            .on("drag", dragged)
            .on("end", dragended))
        .attr("r", 0)
        .attr("fill", "green")
        .attr("class", "node")
        .transition()
            .attr("r", 10)
        .transition()
            .attr()
            .attr("fill", "black")
            .duration(2000)
            .selection()
        .merge(node);

    if (was_changed) {
        simulation.nodes(graphical_nodes)
        simulation.alpha(1).restart();
    }

    /*
    const fieldHeight = 30;
    const fieldWidth = 200;
    const padding = 10;
    
    // Draw the outer box for the struct
    svg.append("rect")
        .attr("x", padding)
        .attr("y", padding)
        .attr("width", fieldWidth)
        .attr("height", struct.fields.length * fieldHeight)
        .attr("fill", "#f0f0f0")
        .attr("stroke", "#333");

    // Struct name label
    svg.append("text")
        .attr("x", padding)
        .attr("y", padding - 2)
        .attr("class", "struct-name")
        .text(`struct ${struct.name}`);

    // Draw fields
    const fieldGroups = svg.selectAll(".field")
        .data(struct.fields)
        .enter()
        .append("g")
        .attr("transform", (d, i) => `translate(${padding}, ${padding + i * fieldHeight})`);

    fieldGroups.append("rect")
        .attr("width", fieldWidth)
        .attr("height", fieldHeight)
        .attr("class", "field");

    fieldGroups.append("text")
        .attr("x", 5)
        .attr("y", fieldHeight / 2)
        .attr("class", "field-label")
        .text(d => `${d.offset} — ${d.name} (${d.size} bytes)`);*/
}