let visualization_containter = document.getElementById("visualization");

/*
=== How I want transitions to function ===
When update is called many times in quick succession

Element starting to dissappear should logically already removed
Elements start existing under Exit

Exit should interrupt entrance

Three phases:
Exit - Things that should be removed are removed
Shift - Things move to their correct place, camera shift
Enter - Things should start existing enter

Interruption during exit:
    - Things continue to dissapear
    - New shift is applied during shift
    - Enter is deferred until new enter

Interruption during shift:
    - Shift still happens
    - Enter is deferred until new enter

Interruption during enter:
    - Entrance animation still happens

*/

// This makes it so that elements that are being removed are logically removed immediately, even though they may transition away
d3.selection.prototype.stable_data = d3.transition.prototype.stable_data = function(data, key) {
    let new_selection = this
        .filter(function(_, _) { return !this.__being_removed })
        .data(data, key);
    
    new_selection
        .exit()
        .each(function(_, _) { this.__being_removed = true });

    return new_selection;
};

const width = 500;
const height = 500;

const frame_width = 75;
const frame_height = 20;
const frame_padding = 4;
const call_stack_label_height = 10;
const call_stack_padding = 5;

const field_height = 25;
const field_width = 40;
const field_padding = 6;
const box_padding = 8;
const label_height = 14;
const label_padding = 2;

const zoom_padding = 50;

let show_header = true;

function node_width(node) {
    return 2 * box_padding + field_width * node.data.fields.length + field_padding * (node.data.fields.length - 1);
}

function node_height(node) {
    return field_height + box_padding + (show_header ? (label_height + 2 * label_padding) : box_padding)
}

function field_dx(i) {
    return box_padding + (field_width + field_padding) * i
}

function field_dy(i) {
    return (show_header ? (label_height + 2 * label_padding) : box_padding)
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
let currently_shifting = false;

const zoom_layer = svg.append("g").attr("class", "zoom-layer");

zoom_layer.append("g")
    .attr("class", "nodes");

zoom_layer.append("g")
    .attr("class", "links");

let call_stack = svg.append("g")
    .attr("stroke", "#333")
    .attr("stroke-width", 1.5)
    .attr("fill", data_color)
    .attr("transform", `translate(${call_stack_padding}, ${call_stack_padding + call_stack_label_height})`)
    .attr("class", "frames");

call_stack.append("text")
    .attr("stroke-width", 0.6)
    .attr("x", frame_width / 2)
    .attr("y", -call_stack_label_height / 2 - 2)
    .attr("font-size", call_stack_label_height + "px")
    .attr("text-anchor", "middle")
    .attr("alignment-baseline", "central")
    .text("Call Stack")

function zoomed({transform}) {
    zoom_layer.attr("transform", transform);
}

visualization_containter.append(svg.node())

d3.select("#showHeaderCheckbox").on("change", update_visualization);

function update_visualization() {
    const mem = wasm_bindgen.take_interpreter_memory_snapshot()

    const new_show_header = d3.select("#showHeaderCheckbox").property("checked");
    const now_showing_headers = new_show_header && !show_header;
    const now_hiding_headers = !new_show_header && show_header;
    show_header = new_show_header

    const graph = d3.graph()

    let graph_nodes = mem.heap.map((fields, i) => {
        let modified_fields = fields.map((field, i) => {
            let label = "";

            if(i == 0) label = "Tag";
            else if (i == 1) label = "Size";
            else if (i == 2) label = "Refs";

            field.label = label;
            field.index = i;
        
            return field
        })

        if(!show_header) modified_fields.splice(0, 3)

        return { fields: modified_fields, id: i }
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
                graph.link(node, target, { field_index: i, id: `${node.data.id}#${i + (show_header ? 0 : 3)}` })
            }
        }
    }

    const { width: graph_width, height: graph_height } = d3.sugiyama()
        .nodeSize(d => [node_width(d), node_height(d)])
        .gap([20, 50])
        .tweaks([/*d3.tweakFlip("diagonal"), */tweak_endpoints])(graph)

    const padded_graph_height = graph_height + zoom_padding;
    const padded_graph_width = graph_width + zoom_padding;

    const node_selection = d3.select(".nodes")
        .selectAll(".node")
        .stable_data(graph.nodes(), d => d.data.id);

    const link_selection = svg
        .select(".links")
        .selectAll("path")
        .stable_data(graph.links(), d => d.data.id)

    const exit_time = node_selection.exit().size() == 0 && link_selection.exit().size() == 0 && !now_hiding_headers ? 0 : 500;
    const shift_time = 500;
    const enter_time = node_selection.enter().size() == 0 && link_selection.enter().size() == 0 && !now_showing_headers ? 0 : 500;

    const node_exit_transition = d3.transition("node").duration(exit_time)
    const node_shift_transition = d3.transition("shift").delay(exit_time).duration(shift_time).ease(d3.easeCubicInOut);
    const node_enter_transition = d3.transition("node").delay(exit_time + shift_time).duration(enter_time)

    node_shift_transition.on("end", function() { currently_shifting = false; })
    node_shift_transition.on("start", function() {
        if(currently_shifting) {
            node_shift_transition.ease(d3.easeCubicOut)
        }

        currently_shifting = true;
    })

    const node_enter_color_transition = node_enter_transition.transition("color").duration(2000);

    const red_percent = 0.6;
    const fade_percent = 1 - 0.4;

    const red_time = red_percent * node_exit_transition.duration();
    const fade_time = fade_percent * node_exit_transition.duration();

    function transform_node(selection) {
        selection
        .attr("transform", d => `translate(${d.x - node_width(d) / 2},${d.y - node_height(d) / 2})`)
    }

    function transform_bounding_box(selection) {
        selection
            .attr("width", d => node_width(d))
            .attr("height", d => node_height(d))
    }
    
    node_selection.join(
        function(enter) {
            let group = enter.append("g")
                .call(transform_node)
                .attr("height", field_height)
                .attr("stroke-width", 2)
                .attr("stroke", "#333")
                .attr("class", "node")
            
            group
                .attr("opacity", 0)
                .transition(node_enter_transition)
                    .attr("opacity", 1)

            group.append("rect")
                .attr("class", "bounding_box")
                .call(transform_bounding_box)
                .attr("fill", "lightgreen")
                .transition(node_enter_color_transition)
                    .attr("fill", data_color)
                    .duration(2000)

            return group;
        },
        function(update) {
            update
                .transition(node_shift_transition)
                .call(transform_node)
            
            update.selectAll(".bounding_box")
                .data(d => [d])
                .join()
                .transition(node_shift_transition)
                .call(transform_bounding_box)

            return update;
        },
        function(exit) {
            exit.selectAll(".bounding_box")
                .transition(node_exit_transition)
                .duration(red_time)
                .attr("fill", "red")

            exit.transition("color")
                .delay(red_time)
                .duration(fade_time)
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

    function transform_field(selection) {
        selection.attr("transform", (_,i) => `translate(${field_dx(i)},${field_dy(i)})`)
    }

    d3.select(".nodes")
        .selectAll(".node")
        .each(function(p, _) {
            d3.select(this)
                .selectAll(".field")
                .stable_data(p.data.fields, d => d.index)
                .join(
                    function(enter) { 
                        let group = enter.append("g")
                            .attr("class", "field")
                            .call(transform_field)

                        group
                            .attr("opacity", 0)
                            .transition(node_enter_transition)
                            .attr("opacity", 1)

                        group.each(function(data) {
                            this.__prev_val = data.val;

                            let rect = d3.select(this).append("rect")
                                .attr("width", field_width)
                                .attr("height", field_height)
                                .attr("rx", 3)
                                .attr("ry", 3)
                                .attr("fill", field_color(data.index));

                            if (data.index < 3) {
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

                        return group;
                    },
                    function(update) {
                        update
                            .transition(node_shift_transition)
                            .call(transform_field)

                        update.each(function(data, i ) {
                            if(this.__prev_val != data.val) {
                                d3.select(this).select("text").call(update_text)

                                d3.select(this).select("rect")
                                    .attr("fill", "orange")
                                    .transition("data_edit")
                                        .duration(1000)
                                        .attr("fill", field_color(data.index))

                                this.__prev_val = data.val
                            }
                        })
                    },
                    function(exit) {
                        exit
                            .transition(node_exit_transition)
                            .attr("opacity", 0)
                            .remove()
                    }
                )
        })
    

    link_selection.join(
        function (enter) {
            return enter.append("path")
            .attr("d", ({ points }) =>
                d3.linkVertical()({
                    source: points[0],
                    target: points[points.length - 1]
                })
            )                
            .attr("stroke-width", 2.5)
            .attr("fill", "none")
            .attr("stroke", "black")
            .attr("opacity", 0)
            .transition(node_enter_transition)
                .attr("opacity", 1)
        },
        function (update) {
            update
                .transition(node_shift_transition)
                .attr("d", ({ points }) =>
                    d3.linkVertical()({
                        source: points[0],
                        target: points[points.length - 1]
                    })
                )
                
        },
        function (exit) {
            exit
                .transition(node_exit_transition)
                .duration(red_time)
                .attr("stroke", "red")
                .transition()
                .duration(fade_time)
                .attr("opacity", 0)
                .remove()
        }
    );

    let stack = mem.call_stack.map((d, i) => ({ func: d, id: d + i }));
    stack.reverse();

    call_stack.selectAll("g")
        .stable_data(stack, (d, i) => d.id)
        .join(enter => {
            let group = enter.append("g")
                .attr("transform", (d, i) => `translate(0, ${i * frame_height})`);

            group.attr("opacity", 0)
                .transition()
                .attr("opacity", 1);

            group.append("rect")
                .attr("width", frame_width)
                .attr("height", frame_height)

            group.append("text")
                .attr("x", frame_width / 2)
                .attr("y", frame_height / 2)
                .attr("text-anchor", "middle")
                .attr("alignment-baseline", "central")
                .attr("stroke-width", 0.8)
                .style("font-size", (frame_height - 2 * frame_padding) + "px")
                .text(d => d.func)

            return group
        },
        update => {
            update.transition()
                .attr("transform", (d, i) => `translate(0, ${i * frame_height})`)
        },
        exit => {
            exit.transition()
            .attr("opacity", 0)
            .remove();
        })

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

    node_shift_transition
    .call(zoom.transform, d3.zoomIdentity.translate(horizontal_padding, vertical_padding).scale(zoom_scale), [horizontal_padding, vertical_padding])

    zoom.translateExtent([[-zoom_padding, -zoom_padding], [padded_graph_width, padded_graph_height]])
    zoom.scaleExtent([zoom_scale, 10])
    console.log(zoom_scale)

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

        let [tx, ty] = link.points[link.points.length - 1]
        ty -= node_height(link.target) / 2
        link.points[link.points.length - 1] = [tx, ty]
    }

    return size
}