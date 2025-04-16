let visualization_containter = document.getElementById("visualization");

const width = 500;
const height = 500;

const box_height = 50;
const box_field_width = 50;

let nodes = []

// Create the SVG container.
const svg = d3.create("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("style", "max-width: 100%; height: auto;");

const zoom = d3.zoom().on("zoom", zoomed);
svg.call(zoom);

/*const simulation = d3.forceSimulation()
    .force("charge", d3.forceManyBody().strength(-200))
    .force("x", d3.forceX(width / 2))
    .force("y", d3.forceY(height / 2))
    .on("tick", ticked);*/

const zoom_layer = svg.append("g").attr("class", "zoom-layer");

zoom_layer.append("g")
    .attr("stroke", "#fff")
    .attr("stroke-width", 1.5)
    .attr("class", "nodes");

zoom_layer.append("g")
    .attr("stroke", "#fff")
    .attr("stroke-width", 1.5)
    .attr("class", "links");

function zoomed({transform}) {
    zoom_layer.attr("transform", transform);
}

/*function ticked() {
    d3.select(".nodes").selectAll("rect")
        .attr("x", d => d.x - d.width / 2)
        .attr("y", d => d.y - box_height / 2);
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
}*/

visualization_containter.append(svg.node())

function update_visualization() {
    const mem = wasm_bindgen.take_interpreter_memory_snapshot()

    const graph = d3.graph()

    let graph_nodes = mem.heap.map(fields => graph.node(fields))
    
    for(const node of graph_nodes) {
        for(const field of node.data) {
            if(field.is_ptr) {
                graph.link(node, graph_nodes[field.val])
            }
        }
    }

    const { width: graph_width, height: graph_height } = d3.sugiyama()
        .nodeSize(d => [10 + box_field_width * d.data.length, box_height])
        .gap([10, 10])
        .tweaks([/*d3.tweakFlip("diagonal"), */])(graph)

    const scale = Math.min(width / graph_width, height / graph_height);
    console.log(graph_width)
    console.log(graph_height)

    const line = d3.line(d => d[0] * scale, d => d[1] * scale).curve(d3.curveBasis);

    function transform_node(selection) {
        selection
        .attr("width", d => (box_field_width * d.data.length + 10) * scale)
        .attr("height", box_height * scale)
        .attr("stroke-width", 2 * scale)
        .attr("x", d => (d.x - (box_field_width * d.data.length + 10) / 2) * scale)
        .attr("y", d => (d.y - box_height / 2) * scale)
    }

    function transform_line(selection) {
        selection
            .attr("d", ({ points }) => line(points))
            .attr("stroke-width", 5 * scale)
    }

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

    let was_changed = false;

    d3.select(".nodes")
        .selectAll("rect")
        .data(graph.nodes())
        .join(
            function(enter) {
                was_changed |= !enter.empty();

                return enter.append("rect")
                    /*.call(d3.drag()
                        .on("start", dragstarted)
                        .on("drag", dragged)
                        .on("end", dragended))*/
                    .call(transform_node)
                    .attr("opacity", 0)
                    .attr("fill", "green")
                    .attr("stroke", "#333")
                    .attr("class", "node")
                    .transition()
                        .attr("opacity", 1)
                        .delay(500)
                    .transition()
                        .attr()
                        .attr("fill", "#f0f0f0")
                        .duration(2000)
            },
            function(update) { 
                update
                    .transition(1)
                    .delay(250)
                    .call(transform_node)
            },
            function(exit) {
                was_changed |= !exit.empty();

                exit.transition()
                    .attr("fill", "red")
                .transition()
                    .attr("opacity", 0)
                    .remove()
            }
        );
    
    svg
        .select(".links")
        .selectAll("path")
        .data(graph.links())
        .join(
            function (enter) {
                return enter.append("path")
                .call(transform_line)
                .attr("fill", "none")
                .attr("stroke", "black")
                .attr("opacity", 0)
                .transition()
                    .attr("opacity", 1)
                    .delay(500)
            },
            function (update) {
                update
                    .transition(1)
                    .delay(250)
                    .call(transform_line)
            },
            function (exit) {
                exit
                    .transition()
                    .delay(250)
                    .attr("opacity", 0)
                    .remove()
            }
        );

    /*if (was_changed) {
        simulation.nodes(nodes)
        simulation.alpha(1).restart();
    }*/

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