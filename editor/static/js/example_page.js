let example_select = document.getElementById("examples");
let output_field = document.getElementById("output-field");
let copied_ack = document.getElementById("copied-ack");

// establish codemirror editor
var code_field = CodeMirror.fromTextArea(document.getElementById("code-field"), {
    lineNumbers: true,
    styleActiveLine: true,
    readOnly: true,
    mode: "GOOPEA",
});

code_field.setSize("100%", "100%");

//loading and unloading
document.addEventListener("DOMContentLoaded", () => {

    if ("example" in localStorage) {
        example_select.value = localStorage.getItem("example");
    }
    example_dropdown_changed();

    if ("theme" in localStorage) {
        if (localStorage.getItem("theme") === "dark") {
            document.documentElement.setAttribute("theme", "dark");
            change_theme(1);
        } else {
            document.documentElement.setAttribute("theme", "default");
        }
    }
});

window.onbeforeunload = function() {    
    if (document.getElementById("theme-button").classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }

    save_example(1);
};

//change example showed from dropdown selection
function example_dropdown_changed() {
    switch(example_select.value) {
        case "reverse":
            code_field.setValue( 
`enum List = Nil, Cons(List, Int);

Int: List
build x = match x == 0 {
    True: Nil,
    False: Cons(build(x - 1), x)
};
    
(List, List): List
reverseHelper(list, acc) =
    match list {
        Nil: acc,
        Cons(xs, x): reverseHelper(xs, Cons(acc, x))
    };

List: List
reverseList list = reverseHelper(list, Nil);

List: Int
sum list = match list {
    Nil: 0,
    Cons(rest, value): value + sum(rest)
};

(): Int
main = sum(reverseList(build(100)));

// (): List
//main = reverseList(Cons(Cons(Cons(Nil, 1), 2), 3));`);
            output_field.value = `5050

// [A: [A: [A: 0, 3], 2], 1]
// aka Cons(Cons(Cons(Nil, 3), 2), 1)
// note that the 0 is because Nil is the first definition of List and the 3, 2, 1 are just Ints`;
            break;
        case "treeflip":
            code_field.setValue(
`enum Tree = Empty, Node(Tree, Int, Tree);

(): Tree
build =
    Node(
        Node(
            Node(
                Empty,
                15,
                Empty
            ),
            10,
            Node(
                Empty,
                52,
                Empty
            )
        ),
        5,
        Node(
            Node(
                Empty,
                69,
                Node(
                    Node(
                        Empty,
                        7,
                        Empty
                    ),
                    100,
                    Node(
                        Empty,
                        4,
                        Empty
                    )
                ),
            ),
            23,
            Empty
        )
    );

Tree: Int
sum tree = match tree {
    Empty: 0,
    Node(left, value, right): sum(left) + value + sum(right)
};

(Tree, Tree): Tree
 

combine (a, b) = match a {
    Empty: match b {
        Empty: Empty,
        Node(left, value, right): Node(left, value, right)
    },
    Node(left, value, right): match b {
        Empty: Node(left, value, right),
        Node(left2, value2, right2): Node(combine(left, left2), value + value2, combine(right, right2))
    }
};

Tree: Tree
flip tree = match tree {
    Empty: Empty,
    Node(left, value, right): Node(flip(right), value, flip(left))
};

(): Int
main = sum(flip(combine(flip(build()), build())));`);
            output_field.value = "570";
            break;
        case "arithmetic":
            code_field.setValue( 
`(): Int
getMinusFive = -5;

(): Int
subtract = 2 - 1;

(): Int
main = 3 * (1 + 15/5) % (6/(2+1))*6;`);
            output_field.value = "0";
            break;
        case "complex-match":
            code_field.setValue( 
`enum Animal = Cat, Dog;

Animal: Animal
convert x = match x {
    Cat: Dog,
    Dog: Cat
};

(): (Int, Int)
coord = (7, 5);

(): Int
letUnbox = let x = 3 in x;

(): Animal
main = match convert Cat {
    Cat: Cat,
    Dog: Dog
};`);
            output_field.value = `1

Dog is the 2nd of the two values of enum Animal`;
            break;
        case "mrv":
            code_field.setValue( 
`enum List = Nil, Cons(List, Int);
enum Maybe = None, Some Int;

List: (List, Maybe)
next list = match list {
        Nil: (Nil, None),
        Cons(xs, x): (xs, Some x)
    };

List: (List, Maybe, Maybe)
next_twice list = 
    let (list, x1) = next list in //(Cons(Cons(Cons(Nil, 3), 4), 5), Some 6)
        let (list, x2) = next list in //(Cons(Cons(Nil, 3), 4), Some 5)
            (list, x1, x2); //(Cons(Cons(Nil, 3), 4), Some 6, Some 5)
            
(): (List, Maybe, Maybe)
main = next_twice(Cons(Cons(Cons(Cons(Nil, 3), 4), 5), 6));`);
            output_field.value = `([B: [B: 0, 3], 4], [B: 6], [B: 5])

aka (Cons(Cons(Nil, 3), 4), Some 6, Some 5)`;
            break;
        case "type-error":
            code_field.setValue( 
`enum Animal = Dog, Cat;

Int: Int
inc x = x + 1;

(): Int
main = inc Dog;`);
            output_field.value = `ERROR: Wrong argument type for function call of 'inc'. Expected (Int), but got (Animal)

Occured at 7:8-7:15

7. main = inc Dog;`;
            break;
        case "utuple":
            code_field.setValue( 
`enum List = Nil, Cons(Int, List);

(List, List, List) : (List, List, List)
g(a,b,c) = (a,b,c);

() : Int
main = let (a,b,c) = g(Cons(5, Nil), Cons(10, Nil), Cons(20, Nil)) in sum(a) + sum(b);


List : Int
sum xs = match xs {
    Nil: 0,
    Cons(x, xx): x + sum(xx)
};`);
            output_field.value = "15";
            break;
        case "zipper-tree":
            code_field.setValue( 
`enum Tree = 
    Bin(Tree, Tree),
    Tip Int;

enum TZipper = 
    Top,                 
    BinL(TZipper, Tree),
    BinR(Tree, TZipper);

fip (Tree, TZipper): Tree
down(t, ctx) =
    match t {
        Bin(l, r):
            down(l, BinL(ctx, r)),
        Tip x: app(Tip(x + 1), ctx)
    };

fip (Tree, TZipper): Tree
app(t, ctx) =
    match ctx {
        Top: t,
        BinR(l, up):
            app(Bin(l, t), up),
        BinL(up, r):
            down(r, BinR(t, up))
    };

fip Tree: Tree
tmap t = down(t, Top);

fip (): Tree
main = tmap(Bin(Tip 1, Bin(Tip 2, Tip 3)));`);
            output_field.value = `[A: [A: 2], [A: [A: 3], [A: 4]]]
            
which is the same as Bin(Tip 2, Bin(Tip 3, Tip 4))
            
walkthrough (note: numbered Tips to keep track of them):
main = tmap(Bin(Tip1 1, Bin(Tip2 2, Tip3 3)));

down(Bin(Tip1 1, Bin(Tip2 2, Tip3 3)), Top)
  Bin l,r
  	l = Tip1 1
  	r = Bin(Tip2 2, Tip3 3)
down(Tip1 1, BinL(Top, Bin(Tip2 2, Tip3 3)))
  Tip x
  	x = 1
app(Tip1 1+1, BinL(Top, Bin(Tip2 2, Tip3 3)))
  BinL up,r
  	up = Top
  	r = Bin(Tip2 2, Tip3 3)
down(Bin(Tip2 2, Tip3 3), BinR(Tip1 2, Top))
  Bin l,r
  	l = Tip2 2
  	r = Tip3 3
down(Tip2 2, BinL(BinR(Tip1 2, Top), Tip3 3))
  Tip x
  	x = 2
app(Tip2 2+1, BinL(BinR(Tip1 2, Top), Tip3 3))
  BinL up,r
  	up = BinR(Tip1 2, Top)
    r = Tip3 3
down(Tip3 3, BinR(Tip2 3, BinR(Tip1 2, Top)))
  Tip x
  	x = 3
app(Tip3 4, BinR(Tip2 3, BinR(Tip1 2, Top)))
  BinR l,up
  	l = Tip2 3
  	up = BinR(Tip1 2, Top)
app(Bin(Tip2 3, Tip3 4), BinR(Tip1 2, Top))
  BinR l,up
  	l = Tip1 2
  	up = Top
app(Bin(Tip1 2, Bin(Tip2 3, Tip3 4)), Top)
  Top t
  	t = Bin(Tip1 2, Bin(Tip2 3, Tip3 4))
      
==> main = Bin(Tip1 2, Bin(Tip2 3, Tip3 4))`;
            break;
        case "bools":
            code_field.setValue( 
`Int: Int
fib x = match (x <= 1) {
    True: 1,
    False: fib(x-1) + fib(x-2)
};

() : Int
main = fib(10);`);
            output_field.value = "89";
            break;
        case "dag":
            code_field.setValue( 
`enum Node = Data(Int), Child(Node), Children(Node, Node);

Node: Node
build x = Children(Child(x), Child(x));
  
(): (Node)
main = build (Data 5);`);
            output_field.value = `[B: [A: [A: 5]], [A: [A: 5]]]
            
aka Children(Child(Data 5), Child(Data 5))`;
            break;
        case "inorder":
            code_field.setValue( 
`enum Tree = Empty, Node(Tree, Int, Tree);
enum List = Nil, Cons(Int, List);


(List,List) : List
concat(xs, ys) = match xs {
    Nil: ys,
    Cons(x,xx): Cons(x, concat(xx, ys))
};

Tree : List
inorder tree = match tree {
    Empty: Nil,
    Node(left, value, right): concat(inorder(left), Cons(value, inorder(right)))
};
      
(): List
main = inorder(Node(Node(Node(Empty, 1, Empty), 2, Node(Empty, 3, Empty)), 4, Node(Empty, 5, Empty)));`);
            output_field.value = `[A: 1, [A: 2, [A: 3, [A: 4, [A: 5, 0]]]]]
            
aka Cons(1, Cons(2, Cons(3, Cons(4, Cons(5, Nil)))))`;
            break;
        case "rdt":
            code_field.setValue( 
`enum Pair = Pair(Int, Int);
enum List = Nil, Cons(Int, List);

List: Pair
from_List xs = match xs {
    Nil: Pair(0, 0),
    Cons(x, xx): Pair(x, x)
};

Pair: Int
toInt(p) = match p {
    Pair(x, y): x * y
};

() : Int
main = toInt(from_List(Cons(7, Cons(2, Cons(3, Nil)))));`);
            output_field.value = "49";
            break;
        case "square":
            code_field.setValue( 
`enum List = Nil, Cons(Int, List);

(): List
build = Cons(2, Cons(3, Cons(4, Nil)));

List: List
squareList list = match list{
	Nil: Nil,
    Cons(x, xs): Cons(x*x, squareList xs)
};

(): List
main = squareList build;`);
            output_field.value = "[4, 9, 16]";
            break;
        case "addone":
            code_field.setValue( 
`enum List = Nil, Cons(Int, List);

List: List
addOne list = match list{
	Nil: Nil,
    Cons(x, xs): Cons(x + 1, addOne xs)
};

(): List
main = addOne Cons(2, Cons(3, Cons(4, Nil)));`);
            output_field.value = "[3, 4, 5]";
            break;
        default:
            code_field.setValue( 
`enum List = Nil, Cons(List, Int);

fip (List, List): List
reverseHelper(list, acc) =
        match list {
            Nil: acc,
            Cons(xs, x): reverseHelper(xs, Cons(acc, x))
        };

fip List: List
reverseList list = reverseHelper(list, Nil);`);
            output_field.value = "Cons(3, Cons(2, Cons(1, Nil)))";
    }

    code_field.refresh();
}

function change_example_editor_theme(opt) {
    switch (opt) {
        case 0: //dark theme
            code_field.setOption("theme", "3024-night");
            break;
        case 1: //light theme
            code_field.setOption("theme", "default");
            break;
        default:
            code_field.setOption("theme", "default");   
    }
}

async function export_code() {
    localStorage.setItem("exported_code", code_field.getValue());
    localStorage.setItem("exported", "true");
    save_example(0); //switches to editor page immediately
}

function save_example(opt) {
    localStorage.setItem("example", example_select.value);

    change_page(opt);
}