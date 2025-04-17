let visualization_containter = document.getElementById("visualization");

const width = 500;
const height = 500;

const box_height = 50;
const box_field_width = 50;

const box_color = "#f0f0f0"

// Create the SVG container.
const svg = d3.create("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("style", "max-width: 100%; height: auto;");

const zoom = d3.zoom().on("zoom", zoomed);
svg.call(zoom);

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

visualization_containter.append(svg.node())

function update_visualization() {
    const mem = wasm_bindgen.take_interpreter_memory_snapshot()
    console.log(mem.heap)

    const graph = d3.graph()

    let graph_nodes = mem.heap.map((fields, i) => graph.node({ fields: fields, id: i }))
    
    for(const node of graph_nodes) {
        for(const [i, field] of node.data.fields.entries()) {
            if(field.is_ptr) {
                let target = graph_nodes[field.val];
                graph.link(node, target, { field_index: i, id: `${node.data.id}-${target.data.id}` })
            }
        }
    }

    const { width: graph_width, height: graph_height } = d3.sugiyama()
        .nodeSize(d => [10 + box_field_width * d.data.fields.length, box_height])
        .gap([20, 50])
        .tweaks([/*d3.tweakFlip("diagonal"), */tweak_endpoints])(graph)

    function transform_node(selection) {
        selection
        .attr("width", d => box_field_width * d.data.fields.length)
        .attr("transform", d => `translate(${d.x - (box_field_width * d.data.fields.length) / 2},${d.y - box_height / 2})`)
    }

    let was_changed = false;

    console.log(graph.nnodes())

    d3.select(".nodes")
        .selectAll(".node")
        .data(graph.nodes(), d => d.data.id)
        .join(
            function(enter) {
                console.log("Enter: " + enter.size())

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
                        .attr("fill", box_color)
                        .duration(2000)
                    .selection()
            },
            function(update) { 
                console.log("Update: " + update.size())
                
                update
                    .transition(1)
                    .delay(250)
                    .call(transform_node)
            },
            function(exit) {
                console.log("Exit: " + exit.size())

                was_changed |= !exit.empty();

                exit.transition()
                    .attr("fill", "red")
                .transition()
                    .attr("opacity", 0)
                    .remove()
            }
        );

    function update_text(selection) {
        selection.text(d => {
            if(d.is_ptr) {
                return ""
            } else {
                return d.val
            }
        })
    }

    d3.select(".nodes")
        .selectAll(".node")
        .each(function(p, _) {
            d3.select(this)
                .selectAll("g")
                .data(p.data.fields)
                .join(function(enter) { 
                    return enter.append("g")
                        .attr("transform", (_,i) => `translate(${i * box_field_width},0)`)
                        .each(function(data, i) {
                            this.__prev_val = data.val;

                            let rect = d3.select(this).append("rect")
                                .attr("width", box_field_width)
                                .attr("height", box_height);

                            if (i < 3) {
                                rect.attr("stroke-dasharray", "5")
                            }

                            d3.select(this).append("text")
                                .attr("x", box_field_width / 2)
                                .attr("y", box_height / 2)
                                .attr("text-anchor", "middle")
                                .attr("alignment-baseline", "central")
                                .call(update_text)
                        })
                    },
                    function(update) {
                        update.each(function(data, i ) {
                            if(this.__prev_val != data.val) {
                                d3.select(this).select("text").call(update_text)

                                d3.select(this).select("rect")
                                    .attr("fill", "orange")
                                    .transition()
                                        .delay(500)
                                        .duration(1000)
                                        .attr("fill", box_color)

                                this.__prev_val = data.val
                            }


                        })
                    }
                )
        })
    
    svg
        .select(".links")
        .selectAll("path")
        .data(graph.links(), d => d.data.id)
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
                update
                    .transition(1)
                    .delay(250)
                    .attr("d", ({ points }) =>
                        d3.linkVertical()({
                          source: points[0],
                          target: points[1]
                        })
                    )
                    
            },
            function (exit) {
                exit
                    .transition()
                    .attr("opacity", 0)
                    .remove()
            }
        );

    if (was_changed) {
        const horizontal_scale = width / graph_width;
        const vertical_scale = height / graph_height;
        const zoom_scale = Math.min(horizontal_scale, vertical_scale);

        let vertical_padding = (height - zoom_scale * graph_height) / 2;
        let horizontal_padding = (width - zoom_scale * graph_width) / 2;

        console.log({hor: horizontal_padding, vert: vertical_padding})
        svg.transition().call(zoom.transform, d3.zoomIdentity.translate(horizontal_padding, vertical_padding).scale(zoom_scale))
    }
}

function tweak_endpoints(graph, size) {
    for(let link of graph.links()) {
        let [sx, sy] = link.points[0]
        sx -= link.source.data.fields.length * box_field_width / 2
        sx += (link.data.field_index + 0.5) * box_field_width
        sy += box_height / 2
        link.points[0] = [sx, sy]

        let [tx, ty] = link.points[1]
        ty -= box_height / 2
        link.points[1] = [tx, ty]
    }

    return size
}