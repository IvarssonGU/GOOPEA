from typing import Optional
import graphviz
    
import os
import glob
import re
from PIL import Image, ImageFont, ImageDraw

#files = glob.glob('/output/*')
#for f in files:
#    print(f)
#    os.remove(f)

SEP_W = 10
def add_code_to_side(base_img, line, fip):
    im1 = Image.new('RGB', (1000, 800), color = (255,255,255))
    fnt = ImageFont.truetype("CONSOLA.TTF", 37)
    draw = ImageDraw.Draw(im1)

    code = ""
    with open("reverse.goo", "r") as f:
        code = f.read()

    code = "\n".join(["{:02d}".format(i+1) + ". " +  ("--- " if i+1 == line else "    ") + ln for i, ln in enumerate(code.split("\n"))])
    if not fip: code = code.replace("fip ", "")

    selected_line_code = "\n".join([(ln if i+1 == line else "") for i, ln in enumerate(code.split("\n"))])
    not_selected_line_code = "\n".join([(ln if i+1 != line else "") for i, ln in enumerate(code.split("\n"))])

    draw.multiline_text((20, 400), not_selected_line_code, font=fnt, fill=(0,0,0), anchor="lm", spacing=25)
    draw.multiline_text((20, 400), selected_line_code, font=fnt, fill=(255,0,0), anchor="lm", spacing=25)
    
    im2 = Image.open(base_img)

    dst = Image.new('RGB', (im1.width + im2.width + SEP_W, im1.height))
    dst.paste(im1, (0, 0))
    dst.paste(im2, (im1.width + SEP_W, 0))

    dst.save(base_img)

scopes = []
def is_in_scope(name: str) -> bool:
    return name in scopes[-1][1]

deallocated = []
edited = []
new_nodes = []
old_vars = []
temp_vars = []
removed_vars = []
rendered_frames = 0
def render(fip: bool, line: int):
    global rendered_frames
    global deallocated
    global new_nodes

    dot = graphviz.Digraph(format='png')
    dot.attr(rankdir='TD')
    dot.attr(dpi='200')
    dot.attr(size='10,4!')

    node_parent = {}
    references = {}
    for name, (val, next) in nodes.items():
        node_parent[next] = name
        references[next] = 1

    roots = []
    for name in nodes.keys():
        if name not in node_parent:
            roots.append(name)
            references[name] = 0

    for var in vars.keys():
        if isinstance(vars[var], str):
            references[vars[var]] += 1

    node_big_subgraph = graphviz.Digraph(name="cluster_nodes")
    node_big_subgraph.attr(style="invis")
    for root in roots:
        curr = root

        node_subgraph = graphviz.Digraph(name="cluster_nodes" + curr)
        node_subgraph.attr(style="invis")

        # node_subgraph = graphviz.Digraph()
        while curr is not None:
            name, (val, next) = (curr, nodes[curr])

            label = f'{{ refs | {references[name]} }} | {{value | {val}}} | {{next | <ptr> {"Nil" if next is None else ""}}}'

            fillcolor = "lightblue"
            if name in deallocated: fillcolor = "red"
            if name in new_nodes: fillcolor = "lightgreen"
            if name in edited: fillcolor = "orange"

            node_subgraph.node(name, label, shape="record", style="filled", fillcolor=fillcolor)

            curr = next
        node_big_subgraph.subgraph(node_subgraph)

    dot.subgraph(node_big_subgraph)

    i, (label, scope_vars) = (0, scopes[-1])
    subgraph = graphviz.Digraph(f"cluster_{i}{label}")
    subgraph.attr(label=label)

    if len(scope_vars) == 0:
        subgraph.node("DUMMY", label="", shape="none")

    for var in scope_vars:
        fillcolor = "lightblue"
        color = None
        fontcolor = None

        #if var in old_vars: 
        #    fillcolor = "#64646430"
        #    color = fillcolor
        #    fontcolor = fillcolor

        if var in removed_vars: 
            fillcolor = "red"
    
        content_label = "Nil"
        if isinstance(vars[var], int): content_label = str(vars[var])
        if isinstance(vars[var], str): content_label = ""

        subgraph.node(var, f"{{{var_labels[var]} | <ptr> {content_label}}}", shape="Mrecord", style="filled", fillcolor=fillcolor, color=color, fontcolor=fontcolor)

    dot.subgraph(subgraph)

    for (a, b) in edges:
        color = "black"
        #if a in vars and a in old_vars:
        #    color = "#64646430"

        if a in vars: 
            if not is_in_scope(a): continue
            a = a + ":ptr"

        dot.edge(a, b, color=color)

    for var in scopes[-1][1]:
        if not var in old_vars:
            old_vars.append(var)

    path = dot.render("output/fip/img_{:02d}".format(rendered_frames) if fip else "output/non_fip/img_{:02d}".format(rendered_frames))
    add_code_to_side(path, line, fip)
    print(f"Rendered {rendered_frames}")
    rendered_frames += 1

    edited.clear()

    if not fip:
        did_deallocate = False
        for node in deallocated:
            did_deallocate = True

            next = nodes[node][1]
            if next is not None:
                edges.remove((node + ":ptr", next))

            del nodes[node]

        new_deallocated = []
        for root in roots:
            if root not in deallocated and references[root] == 0:
                new_deallocated.append(root)
                did_deallocate = True

        deallocated = new_deallocated

        if did_deallocate:
            render(fip, line)

    new_nodes.clear()

    for name in removed_vars:
        remove_var_immediate(name)

    removed_vars.clear()

    newly_removed = False
    for var in temp_vars:
        removed_vars.append(var)
        newly_removed = True
    temp_vars.clear()

    if newly_removed:
        render(fip, line)

added_identifiers = 0
def get_new_ident() -> str:
    global added_identifiers

    name = str(added_identifiers)
    added_identifiers += 1

    return name

edges = []
nodes = {}
def cons(val: int, next: Optional[str], unused_node: Optional[str] = None):
    if unused_node is not None:
        old_next = nodes[unused_node][1]
        if old_next is not None: edges.remove((unused_node + ":ptr", old_next))

        edited.append(unused_node)

    name = get_new_ident() if unused_node is None else unused_node

    nodes[name] = (val, next)

    if unused_node is None:
        new_nodes.append(name)

    if next is not None:
        edges.append((name + ":ptr", next))

    return name

vars = {}
var_labels = {}

def var(label: str, content: Optional[str], temporary: bool = False) -> str:
    name = get_new_ident()

    var_labels[name] = label
    scopes[-1][1].append(name)
    if temporary: temp_vars.append(name)

    vars[name] = None
    set_var_content(name, content)

    return name
        

def set_var_content(name: str, content: Optional[str | int]):
    if isinstance(vars[name], str):
        edges.remove((name, vars[name]))

    vars[name] = content

    if isinstance(content, str):
        edges.append((name, content))

def push_scope(label):
    scopes.append((label, []))

def remove_var_immediate(name):
    if isinstance(vars[name], str):
        edges.remove((name, vars[name]))

    scopes[-1][1].remove(name)

    del vars[name]
    del var_labels[name]

    if name in temp_vars:
        temp_vars.remove(name)

def remove_var(name, fip, line):
    removed_vars.append(name)
    render(fip, line)

def remove_vars(names, fip, line):
    removed_vars.extend(names)
    render(fip, line)

def pop_scope(fip, line):
    gone_vars = scopes[-1][1].copy()

    for name in gone_vars:
        remove_var_immediate(name)

    scopes.pop()

def reverse(list: Optional[str], acc: Optional[str], depth: int, fip: bool) -> str:
    push_scope(f"Reverse|{depth}")

    listvar = var("list", list)
    accvar = var("acc", acc, False)
    render(fip, 3)

    if not fip:
        remove_var(listvar, fip, 4)
    else:
        render(fip, 4)

    if list is None:
        remove_var(accvar, fip, 5)

        pop_scope(fip, 9)
        return acc
    else:

        x = nodes[list][0]
        xs = nodes[list][1]

        xvar = var("x", x)
        xsvar = var("xs", xs)
        render(fip, 6)


        remove_vars([accvar, xvar], fip, 7)
        c = cons(x, acc, list if fip else None)
        if not fip:
            listvar = var("list", c)
        else:
            set_var_content(listvar, c)

        render(fip, 7)

        remove_vars([xsvar, listvar], fip, 7)

        # if fip: temp_vars.append(accvar)
        res = reverse(xs, c, depth + 1, fip)
        retvar = var("return", res)
        render(fip, 8)

        remove_var(retvar, fip, 9)
        pop_scope(fip, 9)

        return res
    
#def reverse(list: str, fip: bool) -> str:
#    push_scope("Reverse")
#
#    var("list", list, True)
#    render(fip)
#
#    reversed = reverseHelper(list, None, 1, fip)
#    var("return", reversed)
#    render(fip)
#
#    pop_scope(fip)
#
#    return reversed
    
def main(fip: bool):
    push_scope("Main")
    render(fip, 11)

    render(fip, 12)
    c1 = cons(2, None)
    tmpvar = var("temp", c1)
    render(fip, 12)

    render(fip, 13)
    c2 = cons(1, c1)
    set_var_content(tmpvar, c2)
    render(fip, 13)

    render(fip, 14)
    remove_var(tmpvar, fip, 14)

    reversed = reverse(c2, None, 1, fip)

    var("return", reversed)
    render(fip, 11)

    pop_scope(fip, 11)

main(False)

vars.clear()
var_labels.clear()
edges.clear()
nodes.clear()

deallocated.clear()
edited.clear()
new_nodes.clear()
old_vars.clear()
temp_vars.clear()
removed_vars.clear()

rendered_frames = 0
added_identifiers = 0

main(True)