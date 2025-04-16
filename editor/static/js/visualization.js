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
        for(const [i, field] of node.data.entries()) {
            if(field.is_ptr) {
                graph.link(node, graph_nodes[field.val], i)
            }
        }
    }

    const { width: graph_width, height: graph_height } = d3.sugiyama()
        .nodeSize(d => [10 + box_field_width * d.data.length, box_height])
        .gap([20, 50])
        .tweaks([/*d3.tweakFlip("diagonal"), */tweak_endpoints])(graph)


    const zoom_scale = Math.min(width / graph_width, height / graph_height);


    console.log(graph_width)
    console.log(graph_height)

    const line = d3.linkVertical().x(d => d[0]).y(d => d[1]);

    function transform_node(selection) {
        selection
        .attr("width", d => box_field_width * d.data.length)
        .attr("transform", d => `translate(${d.x - (box_field_width * d.data.length) / 2},${d.y - box_height / 2})`)
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
        .selectAll("g")
        .data(graph.nodes())
        .join(
            function(enter) {
                was_changed |= !enter.empty();

                return enter.append("g")
                    /*.call(d3.drag()
                        .on("start", dragstarted)
                        .on("drag", dragged)
                        .on("end", dragended))*/
                    .call(transform_node)
                    .attr("height", box_height)
                    .attr("stroke-width", 2)
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
                    .selection()
                    .each(function(p, _) {
                        d3.select(this)
                            .selectAll("rect")
                            .data(p.data)
                            .join(enter => 
                                enter
                                .append("g")
                                .attr("transform", (_,i) => `translate(${i * box_field_width},0)`)
                                .each(function(data, i) {
                                    let rect = d3.select(this).append("rect")
                                        .attr("width", box_field_width)
                                        .attr("height", box_height);

                                    if (i < 3) {
                                        rect.attr("stroke-dasharray", "5")
                                    }

                                    if (!data.is_ptr) {
                                        d3.select(this).append("text")
                                            .attr("x", box_field_width / 2)
                                            .attr("y", box_height / 2)
                                            .attr("text-anchor", "middle")
                                            .attr("alignment-baseline", "central")
                                            .text(d => d.val)
                                    }
                                }) 
                            )
                    })
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
                .attr("d", ({ points }) =>
                    d3.linkVertical()({
                      source: points[0],
                      target: points[1]
                    })
                )                
                .attr("stroke-width", 2.5)
                .attr("fill", "none")
                .attr("stroke", "black")
                .attr("opacity", 0)
                .transition()
                    .attr("opacity", 1)
                    .delay(500)
            },
            function (update) {

            },
            function (exit) {
                exit
                    .transition()
                    .delay(250)
                    .attr("opacity", 0)
                    .remove()
            }
        );

    if (was_changed) {
        svg.transition().call(zoom.scaleTo, zoom_scale)
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

function tweak_endpoints(graph, size) {
    for(let link of graph.links()) {
        let [sx, sy] = link.points[0]
        sx -= link.source.data.length * box_field_width / 2
        sx += (link.data + 0.5) * box_field_width
        sy += box_height / 2
        link.points[0] = [sx, sy]

        let [tx, ty] = link.points[1]
        ty -= box_height / 2
        link.points[1] = [tx, ty]
    }

    return size
}