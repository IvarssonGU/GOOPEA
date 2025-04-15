let visualization_containter = document.getElementById("visualization");

const width = 500;
const height = 500;

const box_height = 50;
const box_field_width = 50;

let nodes = []
let links = []

// Create the SVG container.
const svg = d3.create("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("style", "max-width: 100%; height: auto;");

const simulation = d3.forceSimulation()
    .force("link", d3.forceLink(links).id(d => d.id).distance(200))
    .force("charge", d3.forceManyBody().strength(-200))
    .force("x", d3.forceX(width / 2))
    .force("y", d3.forceY(height / 2))
    .on("tick", ticked);

let link = svg.append("g")
    .attr("stroke", "#999")
    .attr("stroke-opacity", 0.6)
    .selectAll("line");

let node = svg.append("g")
    .attr("stroke", "#fff")
    .attr("stroke-width", 1.5)
    .selectAll("circle");

function ticked() {
    node
        .attr("x", d => d.x - d.width / 2)
        .attr("y", d => d.y - box_height / 2);

    link
        .attr("x1", d => d.source.x)
        .attr("y1", d => d.source.y)
        .attr("x2", d => d.target.x)
        .attr("y2", d => d.target.y);
}

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

    while (nodes.length < mem.heap.length) {
        nodes.push({ x: 0, y: height / 2, id: nodes.length })
    }

    while (nodes.length > mem.heap.length) {
        nodes.pop()
    }

    for (let i = 0; i < mem.heap.length; i++) {
        nodes[i].fields = mem.heap[i]
        nodes[i].width = mem.heap[i].length * box_field_width
    }

    node = node.data(nodes);
    let was_changed = !node.enter().empty() || !node.exit().empty();

    node.exit()
        .transition()
            .attr("fill", "red")
        .transition()
            .attr("opacity", 0)
            .remove()

    node = node.enter().append("rect")
        .call(d3.drag()
            .on("start", dragstarted)
            .on("drag", dragged)
            .on("end", dragended))
        .attr("width", x => x.width)
        .attr("height", box_height)
        .attr("opacity", 0)
        .attr("fill", "green")
        .attr("stroke", "#333")
        .attr("class", "node")
        .transition()
            .attr("opacity", 1)
        .transition()
            .attr()
            .attr("fill", "#f0f0f0")
            .duration(2000)
            .selection()
        .merge(node);

    links = []
    for([i, fields] of mem.heap.entries()) {
        for(data of fields) {

            if (!data.is_ptr) { continue }

            links.push({
                source: i,
                target: data.val
            })
        }
    }

    link = link.data(links, d => d.source.id + "-" + d.target.id);
    was_changed = was_changed || !link.enter().empty() || !link.exit().empty();

    link.exit().remove();
    link = link.enter().append("line")
      .attr("class", "link")
      .merge(link);

    console.log(links)

    
    if (was_changed) {
        simulation.force("link").links(links);
        simulation.nodes(nodes)
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