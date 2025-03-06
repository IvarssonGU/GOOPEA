from typing import Optional
import graphviz
    
import os
import glob
import re

#files = glob.glob('/output/*')
#for f in files:
#    print(f)
#    os.remove(f)

scopes = []
def is_in_scope(name: str) -> bool:
    return name in scopes[-1][1]

deallocated = []
new_nodes = []
old_vars = []
rendered_frames = 0
def render():
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

            label = f'''{{value | {val}}} | {{next | <ptr> {"Nil" if next is None else "Cons"}}} | {{ refs | {references[name]} }}'''

            fillcolor = "lightblue"
            if name in deallocated: fillcolor = "red"
            if name in new_nodes: fillcolor = "lightgreen"

            node_subgraph.node(name, label, shape="record", style="filled", fillcolor=fillcolor)

            curr = next
        node_big_subgraph.subgraph(node_subgraph)

    dot.subgraph(node_big_subgraph)

    i, (label, scope_vars) = (0, scopes[-1])
    subgraph = graphviz.Digraph(f"cluster_{i}{label}")
    subgraph.attr(label=label)

    for var in scope_vars:
        fillcolor = "lightblue"
        color = None
        fontcolor = None

        if var in old_vars: 
            fillcolor = "#64646430"
            color = fillcolor
            fontcolor = fillcolor
    
        content_label = "Nil"
        if isinstance(vars[var], int): content_label = str(vars[var])
        if isinstance(vars[var], str): content_label = "Cons"

        subgraph.node(var, f"{{{var_labels[var]} | <ptr> {content_label}}}", shape="Mrecord", style="filled", fillcolor=fillcolor, color=color, fontcolor=fontcolor)
    
    dot.subgraph(subgraph)

    for (a, b) in edges:
        color = "black"
        if a in vars and a in old_vars:
            color = "#64646430"

        if a in vars: 
            if not is_in_scope(a): continue
            a = a + ":ptr"

        dot.edge(a, b, color=color)

    for var in scopes[-1][1]:
        if not var in old_vars:
            old_vars.append(var)

    dot.render(f"output/memory_step_{rendered_frames}")
    print(f"Rendered {rendered_frames}")
    rendered_frames += 1

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
    new_nodes.clear()

    if did_deallocate:
        render()

added_identifiers = 0
def get_new_ident() -> str:
    global added_identifiers

    name = str(added_identifiers)
    added_identifiers += 1

    return name

edges = []
nodes = {}
def cons(val: int, next: Optional[str] = None):
    name = get_new_ident()

    nodes[name] = (val, next)
    new_nodes.append(name)

    if next is not None:
        edges.append((name + ":ptr", next))

    var("temp", name)

    render()
    return name

vars = {}
var_labels = {}

def var(label: str, content: Optional[str]) -> str:
    name = get_new_ident()

    var_labels[name] = label
    scopes[-1][1].append(name)

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

def remove_var(name):
    if isinstance(vars[name], str):
        edges.remove((name, vars[name]))

    del vars[name]
    del var_labels[name]

def pop_scope():
    gone_vars = scopes.pop()[1]

    for name in gone_vars:
        remove_var(name)

def reverseHelper(list: Optional[str], acc: Optional[str], depth: int, fip: bool) -> str:
    push_scope(f"ReverseHelper|{depth}")

    var("list", list)
    var("acc", acc)
    render()

    if list is None:
        pop_scope()
        return acc
    else:
        x = nodes[list][0]
        xs = nodes[list][1]

        var("x", x)
        var("xs", xs)
        render()

        res = reverseHelper(xs, cons(x, acc), depth + 1, fip)
        var("return", res)
        render()
        pop_scope()

        return res
    
def reverse(list: str, fip: bool) -> str:
    push_scope("Reverse")

    var("list", list)
    render()

    reversed = reverseHelper(list, None, 0, fip)
    var("return", reversed)
    render()

    pop_scope()

    return reversed
    
def do_reverse(fip: bool):
    push_scope("Main")
    push_scope("Example")

    lst = cons(1, cons(2, None))

    reversed = reverse(lst, fip)
    var("return", reversed)
    render()

    pop_scope()

    var("temp", reversed)
    render()

do_reverse(True)