let visualization_container = document.getElementById("visualization");

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

const frame_width = 75;
const frame_height = 20;
const frame_padding = 2;
const call_stack_label_height = 10;
const call_stack_padding = 5;

const field_height = 25;
const field_width = 40;
const field_padding = 6;
const interior_field_padding = 4;
const box_padding = 8;
const label_height = 14;
const label_padding = 2;

const zoom_padding = 50;

let show_header = true;

function node_width(node) {
    if(!node.data.is_var) return 2 * box_padding + field_width * node.data.fields.length + field_padding * (node.data.fields.length - 1);
    else return field_width;
}

function node_height(node) {
    if(!node.data.is_var) return field_height + box_padding + (show_header ? (label_height + 2 * label_padding) : box_padding)
    else return field_height + (node.data.label.length > 0 ? label_height + label_padding : 0);
}

function field_dx(i) {
    return box_padding + (field_width + field_padding) * i
}

function field_dy(i) {
    return (show_header ? (label_height + 2 * label_padding) : box_padding)
}

function var_dx(d) {
    return -node_width(d) / 2
}

function var_dy(d) {
    return (d.data.label.length > 0 ? label_height + label_padding : 0) - node_height(d) / 2
}

// Create the SVG container.
const svg = d3.create("svg")
    .attr("class", "svg");

const zoom = d3.zoom().on("zoom", zoomed);
svg.call(zoom);

let prev_zoom_scale = 1;
let currently_shifting = false;

const zoom_layer = svg.append("g");

let call_stack = svg.append("g")
    .attr("transform", `translate(${call_stack_padding}, ${call_stack_padding + call_stack_label_height})`)
    .attr("class", "frames");

call_stack.append("text").attr("id", "call-stack-text")
    .attr("x", frame_width / 2)
    .attr("y", -call_stack_label_height / 2 - 2)
    .attr("font-size", call_stack_label_height + "px")
    .text("Call Stack")

function zoomed({transform}) {
    zoom_layer.attr("transform", transform);
}

visualization_container.append(svg.node())

d3.select("#showHeaderCheckbox").on("change", update_visualization);

function update_visualization() {
    const mem = wasm_bindgen.take_interpreter_memory_snapshot()

    const entrance_color = window.getComputedStyle(svg.node()).getPropertyValue('--entrance-color');
    const exit_color = window.getComputedStyle(svg.node()).getPropertyValue('--exit-color');
    const update_color = window.getComputedStyle(svg.node()).getPropertyValue('--update-color');

    const new_show_header = d3.select("#showHeaderCheckbox").property("checked");
    const now_showing_headers = new_show_header && !show_header;
    const now_hiding_headers = !new_show_header && show_header;
    show_header = new_show_header

    const graph = d3.graph()

    let mem_nodes = mem.heap.map((fields, i) => {
        let modified_fields = fields.map((field, i) => {
            let label = "";

            if(i == 0) label = "Tag";
            else if (i == 1) label = "Size";
            else if (i == 2) label = "Refs";

            field.label = label;
        
            return { data: field, index: i }
        })

        if(!show_header) modified_fields.splice(0, 3)

        return { fields: modified_fields, id: i, is_var: false }
    }).reduce((map, d) => {
        if(d.fields.length > 0) {
            map.set(d.id, graph.node(d))
        }

        return map;
    }, new Map());

    let var_nodes = [...mem.variables.entries().map(([label, data]) => {
        data.label = label;
        data.is_var = true;
        return graph.node(data)
    })];

    for(const node of var_nodes) {
        if(node.data.is_ptr) {
            let target = mem_nodes.get(node.data.val);
            graph.link(node, target, { id: node.data.label })
        }
    }
    
    for(const node of mem_nodes.values()) {
        for(const [i, d] of node.data.fields.entries()) {
            if(d.data.is_ptr) {
                let target = mem_nodes.get(d.data.val);
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

    const mem_selection = zoom_layer.selectAll(".node")
        .stable_data(mem_nodes.values(), d => d.data.id);

    const var_selection = zoom_layer.selectAll(".var")
        .stable_data(var_nodes, d => d.data.label);

    const link_selection = zoom_layer.selectAll(".link")
        .stable_data(graph.links(), d => d.data.id)

    let nothing_exiting = mem_selection.exit().size() == 0 && link_selection.exit().size() == 0 && var_selection.exit().size() == 0 && !now_hiding_headers;
    let nothing_entering = mem_selection.enter().size() == 0 && link_selection.enter().size() == 0 && var_selection.enter().size() == 0 && !now_showing_headers;

    const exit_time = nothing_exiting ? 0 : 500;
    const shift_time = 500;
    const enter_time = nothing_entering ? 0 : 500;

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

    const node_enter_color_transition = node_enter_transition.transition("color").duration(1000);

    const red_percent = 0.6;
    const fade_percent = 1 - 0.4;

    const red_time = red_percent * node_exit_transition.duration();
    const fade_time = fade_percent * node_exit_transition.duration();

    function fade_in(selection) {
        selection
            .style("opacity", 0)
            .transition(node_enter_transition)
                .style("opacity", null)
    }

    function green_in(selection) {
        selection.style("fill", entrance_color)
            .transition(node_enter_color_transition)
                .style("fill", null)
                .duration(2000)
    }

    function transform_node(selection) {
        selection
        .attr("transform", d => `translate(${d.x - node_width(d) / 2},${d.y - node_height(d) / 2})`)
    }

    function transform_bounding_box(selection) {
        selection
            .attr("width", d => node_width(d))
            .attr("height", d => node_height(d))
    }
    
    mem_selection.join(
        function(enter) {
            let group = enter.append("g").attr("class", "node")
                .call(transform_node)
                .attr("height", field_height)
            
            group.call(fade_in)

            group.append("rect").attr("class", "bounding_box box")
                .call(transform_bounding_box)
                .call(green_in)

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
                .style("fill", exit_color)

            exit.transition("color")
                .delay(red_time)
                .duration(fade_time)
                .style("opacity", 0)
                .remove()
        }
    );

    function join_field(selection, position_func, class_func, enter_func, exit_func) {
        function get_text(d) {
            if(d.data.is_ptr) {
                return ""
            } else {
                return d.data.val
            }
        }

        function update_text(selection) {
            selection.text(get_text)
            .call(center_and_size_text, interior_field_padding, interior_field_padding, field_width - interior_field_padding * 2, field_height - interior_field_padding * 2)
        }
    
        function transform_field(selection) {
            selection.attr("transform", function(d,i) {
                let pos = position_func(d, i);
                return `translate(${pos[0]},${pos[1]})`;
            })
        }
    
        selection.join(
            function(enter) { 
                let group = enter.append("g").attr("class", d => class_func(d))
                    .call(transform_field)
    
                group.call(fade_in)
    
                group.append("rect").attr("class", "box")
                    .attr("width", field_width)
                    .attr("height", field_height);
    
                group.append("text")
                    .call(update_text)
    
                group.append("text").attr("class", "field-name")
                    .text(d => d.data.label)
                    .call(center_and_size_text, 0, -label_padding - label_height, field_width, label_height)

                group.call(enter_func)
    
                return group;
            },
            function(update) {
                update
                    .transition(node_shift_transition)
                    .call(transform_field)
    
                update.each(function(d, i ) {
                    let text_obj = d3.select(this).select("text");

                    if(text_obj.text() != get_text(d)) {
                        text_obj.call(update_text)
    
                        d3.select(this).select("rect")
                            .style("fill", update_color)
                            .transition("data_edit")
                                .duration(1000)
                                .style("fill", null)
                    }
                })
            },
            function(exit) {
                exit.call(exit_func)
            }
        )
    }

    zoom_layer.selectAll(".node")
        .each(function(p, _) {
            d3.select(this)
                .selectAll(".field")
                .stable_data(p.data.fields, d => d.index)
                .call(join_field, (_, i) => [field_dx(i), field_dy(i)], d => "field" + (d.index < 3 ? " header-field" : ""), _ => {}, 
                    exit => {
                        exit
                            .transition(node_exit_transition)
                            .style("opacity", 0)
                            .remove()
                    }
                );
        });
        
    var_selection
        .call(
            join_field, 
            (d, _) => [d.x + var_dx(d), d.y + var_dy(d)], 
            _ => "var", 
            s => s.select(".box").call(green_in),
            exit => {
                exit.selectAll(".box")
                    .transition(node_exit_transition)
                    .duration(red_time)
                    .style("fill", exit_color)

                exit.transition("color")
                    .delay(red_time)
                    .duration(fade_time)
                    .style("opacity", 0)
                    .remove()
            }
        )

    link_selection.join(
        function (enter) {
            let link = enter.append("path").attr("class", "link")
                .attr("d", ({ points }) =>
                    d3.linkVertical()({
                        source: points[0],
                        target: points[points.length - 1]
                    })
                )    
                       
            link.call(fade_in)

            return link;
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
                .style("stroke", exit_color)
                .transition()
                .duration(fade_time)
                .style("opacity", 0)
                .remove()
        }
    );

    let stack = mem.call_stack.map((d, i) => ({ func: d, id: d + i }));
    stack.reverse();

    call_stack.selectAll("g")
        .stable_data(stack, (d, i) => d.id)
        .join(enter => {
            let group = enter.append("g").attr("class", "stack-frame")
                .attr("transform", (d, i) => `translate(0, ${i * frame_height})`);

            group.style("opacity", 0)
                .transition()
                .style("opacity", null);

            group.append("rect").attr("class", "box")
                .attr("width", frame_width)
                .attr("height", frame_height)

            group.append("text")
                .text(d => d.func)
                .call(center_and_size_text, frame_padding, frame_padding, frame_width - frame_padding * 2, frame_height - frame_padding * 2)
                

            return group
        },
        update => {
            update.transition()
                .attr("transform", (d, i) => `translate(0, ${i * frame_height})`)
        },
        exit => {
            exit.transition()
            .style("opacity", 0)
            .remove();
        })

    let zoom_scale;

    const size = svg.node().getBoundingClientRect();

    //Check if the graph has positive area
    if(graph_width * graph_height > 0) {
        const horizontal_scale = size.width / padded_graph_width;
        const vertical_scale = size.height / padded_graph_height;
        zoom_scale = Math.min(horizontal_scale, vertical_scale);
    } else {
        zoom_scale = 1;
    }

    let vertical_padding = (size.height - zoom_scale * graph_height) / 2;
    let horizontal_padding = (size.width - zoom_scale * graph_width) / 2;

    node_shift_transition
    .call(zoom.transform, d3.zoomIdentity.translate(horizontal_padding, vertical_padding).scale(zoom_scale), [horizontal_padding, vertical_padding])

    zoom.translateExtent([[-zoom_padding, -zoom_padding], [padded_graph_width, padded_graph_height]])
    zoom.scaleExtent([zoom_scale, 10])

    prev_zoom_scale = zoom_scale
}

function center_and_size_text(selection, x, y, width, height) {
    selection.each(function(d) {
        let label = d3.select(this);

        if(label.text().length == 0) return;
    
        let label_bbox = label.node().getBBox();
        let label_scale = Math.min(
            width / label_bbox.width,
            height / label_bbox.height
        );
    
        let new_label_font_size = Math.floor(height * label_scale);
    
        label
            .style("font-size", new_label_font_size + "px")
            .attr("x", x + width / 2)
            .attr("y", y + height / 2 + new_label_font_size / 3.5);
    })
}

function tweak_endpoints(graph, size) {
    for(let link of graph.links()) {
        let [sx, sy] = link.points[0]
        if(link.source.data.is_var) {
            sy += var_dy(link.source) + field_height;
        } else {
            sx -= node_width(link.source) / 2
            sx += field_dx(link.data.field_index) + field_width / 2

            sy -= node_height(link.source) / 2
            sy += field_dy(link.data.field_index) + field_height
        }
        link.points[0] = [sx, sy]

        let [tx, ty] = link.points[link.points.length - 1]
        ty -= node_height(link.target) / 2
        link.points[link.points.length - 1] = [tx, ty]
    }

    return size
}