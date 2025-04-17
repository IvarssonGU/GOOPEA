let visualization_containter = document.getElementById("visualization");

const width = 500;
const height = 500;

const field_height = 25;
const field_width = 40;
const field_padding = 6;
const box_padding = 8;
const label_height = 14;
const label_padding = 2;

const zoom_padding = 50;

function node_width(node) {
    return 2 * box_padding + field_width * node.data.fields.length + field_padding * (node.data.fields.length - 1);
}

function node_height(node) {
    return field_height + box_padding + label_height + 2 * label_padding
}

function field_dx(i) {
    return box_padding + (field_width + field_padding) * i
}

function field_dy(i) {
    return label_height + 2 * label_padding
}

const data_color = "#f0f0f0";
const header_color = "#c0c0c0";

function field_color(i) {
    if(i < 3) return header_color;
    else return data_color;
}

// Create the SVG container.
const svg = d3.create("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("style", "max-width: 100%; height: auto;");

const zoom = d3.zoom().on("zoom", zoomed);
svg.call(zoom);

let prev_zoom_scale = 1;

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

    let graph_nodes = mem.heap.map((fields, i) => {
        let labeled_fields = fields.map((field, i) => {
            let label = "";

            if(i == 0) label = "Tag";
            else if (i == 1) label = "Size";
            else if (i == 2) label = "Refs";

            field.label = label;
        
            return field
        })

        return { fields: labeled_fields, id: i }
    }).reduce((map, d) => {
        if(d.fields.length > 0) {
            map.set(d.id, graph.node(d))
        } 

        return map;
    }, new Map());
    
    for(const node of graph_nodes.values()) {
        for(const [i, field] of node.data.fields.entries()) {
            if(field.is_ptr) {
                let target = graph_nodes.get(field.val);
                graph.link(node, target, { field_index: i, id: `${node.data.id}-${target.data.id}` })
            }
        }
    }

    const { width: graph_width, height: graph_height } = d3.sugiyama()
        .nodeSize(d => [node_width(d), node_height(d)])
        .gap([20, 50])
        .tweaks([/*d3.tweakFlip("diagonal"), */tweak_endpoints])(graph)

    const padded_graph_height = graph_height + zoom_padding;
    const padded_graph_width = graph_width + zoom_padding;

    function transform_node(selection) {
        selection
        .attr("transform", d => `translate(${d.x - node_width(d) / 2},${d.y - node_height(d) / 2})`)
    }

    d3.select(".nodes")
        .selectAll(".node")
        .data(graph.nodes(), d => d.data.id)
        .join(
            function(enter) {
                return enter.append("g")
                    .call(transform_node)
                    .attr("height", field_height)
                    .attr("stroke-width", 2)
                    .attr("stroke", "#333")
                    .attr("class", "node")
                    .attr("opacity", 0)
                    .transition()
                        .attr("opacity", 1)
                        .delay(500)
                    .selection()
                        .append("rect")
                        .attr("class", "bounding_box")
                        .attr("width", d => node_width(d))
                        .attr("height", d => node_height(d))
                        .attr("fill", "lightgreen")
                        .transition()
                            .delay(500)
                            .attr("fill", data_color)
                            .duration(2000)
            },
            function(update) { 
                update
                    .transition(1)
                    .delay(250)
                    .call(transform_node)
            },
            function(exit) {
                exit.selectAll(".bounding_box")
                    .transition()
                    .attr("fill", "red")

                exit.transition()
                    .delay(500)
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
                        .attr("transform", (_,i) => `translate(${field_dx(i)},${field_dy(i)})`)
                        .each(function(data, i) {
                            this.__prev_val = data.val;

                            let rect = d3.select(this).append("rect")
                                .attr("width", field_width)
                                .attr("height", field_height)
                                .attr("fill", field_color(i));

                            if (i < 3) {
                                rect.attr("stroke-dasharray", "2.5")
                            }

                            d3.select(this).append("text")
                                .attr("x", field_width / 2)
                                .attr("y", field_height / 2)
                                .attr("text-anchor", "middle")
                                .attr("alignment-baseline", "central")
                                .call(update_text)

                            let label = d3.select(this).append("text")
                                .attr("x", field_width / 2)
                                .attr("y", -label_padding - label_height/2)
                                .attr("text-anchor", "middle")
                                .attr("alignment-baseline", "central")
                                .attr("stroke-width", 0.6)
                                .attr("font-style", "italic")
                                .text(data.label)
                                .style("font-size", label_height + "px");

                            let label_bbox = label.node().getBBox();
                            let label_scale = Math.min(
                                field_width / label_bbox.width,
                                label_height / label_bbox.height
                            );
                            
                            let new_label_font_size = Math.floor(label_height * label_scale);
                            
                            label.style("font-size", new_label_font_size + "px");
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
                                        .attr("fill", field_color(i))

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
                    .attr("stroke", "red")
                    .transition()
                    .delay(250)
                    .attr("opacity", 0)
                    .remove()
            }
        );

    let zoom_scale;

    //Check if the graph has positive area
    if(graph_width * graph_height > 0) {
        const horizontal_scale = width / padded_graph_width;
        const vertical_scale = height / padded_graph_height;
        zoom_scale = Math.min(horizontal_scale, vertical_scale);
    } else {
        zoom_scale = 1;
    }

    let vertical_padding = (height - zoom_scale * graph_height) / 2;
    let horizontal_padding = (width - zoom_scale * graph_width) / 2;

    let transition = svg.transition().duration(500);
    if(prev_zoom_scale < zoom_scale) {
        transition = transition.delay(1000);
    }
    transition.call(zoom.transform, d3.zoomIdentity.translate(horizontal_padding, vertical_padding).scale(zoom_scale))

    prev_zoom_scale = zoom_scale
}

function tweak_endpoints(graph, size) {
    for(let link of graph.links()) {
        let [sx, sy] = link.points[0]
        sx -= node_width(link.source) / 2
        sx += field_dx(link.data.field_index) + field_width / 2

        sy -= node_height(link.source) / 2
        sy += field_dy(link.data.field_index) + field_height
        link.points[0] = [sx, sy]

        let [tx, ty] = link.points[1]
        ty -= node_height(link.target) / 2
        tx -= node_width(link.target) / 2
        tx += field_dx(3 + Math.floor((link.target.data.fields.length - 3) / 2)) + field_width / 2
        link.points[1] = [tx, ty]
    }

    return size
}