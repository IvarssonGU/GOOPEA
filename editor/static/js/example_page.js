let example_select = document.getElementById("examples");
let output_field = document.getElementById("output-field");
let copied_ack = document.getElementById("copied-ack");

var code_field = CodeMirror.fromTextArea(document.getElementById("code-field"), {
    lineNumbers: true,
    styleActiveLine: true,
    readOnly: true,
    mode: "GOOPEA",
});

//change codemirror editor to fit code heightwise
// code_field.setSize("100%", "100%");

// window.onload = function () {
document.addEventListener("DOMContentLoaded", () => {

    if ("example" in localStorage) {
        example_select.value = localStorage.getItem("example");
    }
    example_dropdown_changed();

    if ("theme" in localStorage) {
        if (localStorage.getItem("theme") === "dark") {
            // change_theme(0);
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
};

//change example showed from dropdown
// example_select.addEventListener(onChange, (event) => {
function example_dropdown_changed() {
    switch(example_select.value) {
        case "reverse":
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
            output_field.value = "reverseList(Cons(1, Cons(2, Cons(3, Nil))))) = Cons(3, Cons(2, Cons(1, Nil)))";
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
                Empty
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

Tree: Tree
flip tree = match tree {
    Empty: Empty,
    Node(left, value, right): Node(flip(right), value, flip(left))
};

(): Int
main = sum(flip(build()));`);
            output_field.value = "174";
            break;
        case "arithmetic":
            code_field.setValue( 
`(): Int
getMinusFive = -5;

(): Int
subtract = 2 - 1;

(): Int
main = 3 * (1 + 15/5) - (6/(2+1))*6;`);
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

(): Int
matchUnbox = match (1, 2) {
    (x, y): x + y
};

(): (Int, Int)
coord = (7, 5);

(): Int
letUnbox = let x = 3 in x;

(): Int
matchUnbox2 = match coord {
    (x, y): x - y
};

Int: Int
fib i = match i {
    0: 1,
    1: 1,
    n: fib (i-1) + fib(i - 2)
};

(): Animal
main = match convert Cat {
    Cat: Cat,
    Dog: Dog
};`);
            output_field.value = "Dog";
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
    let (list, x1) = next list in 
        let (list, x2) = next list in 
            (list, x1, x2);`);
            output_field.value = "output here";
            break;
        case "type-error":
            code_field.setValue( 
`enum Animal = Dog, Cat;

Int: Int
inc x = x + 1;

(): Int
main = print (inc Dog);

Int: ()
print x = ();`);
            output_field.value = "output here";
            break;
        case "utuple":
            code_field.setValue( 
`enum Maybe = Nothing, Just Int;

(): (Int, Int)
nums = (Nothing, 5 * 2 + 9 * 20);

(): Int
main = let (a, b) = nums() in match a {
    Nothing: b,
    Just x: x * b + b
};`);
            output_field.value = "";
            break;
        case "zipper-tree":
            code_field.setValue( 
`// this is a test file
enum Tree = 
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
            down(l, BinL(ctx, r)), //Down comment
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
tmap t = down(t, Top);`);
            output_field.value = "output here";
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

async function copy_code() {
    // if (copied_ack.classList.contains("appearing")) copied_ack.classList.toggle("appearing");
    
    copied_ack.classList.toggle("appearing");
    // console.log("dfs");
    navigator.clipboard.writeText(code_field.getValue());
    // copied_ack.classList.toggle("appearing");
}

function save_example(opt) {
    localStorage.setItem("example", example_select.value);

    change_page(opt);
}


//slideshow
let slide_index = 0;
show_slide(slide_index);

function change_slide(n) {
    show_slide(slide_index += n);
}

function show_slide(i) {
    let slides = document.getElementsByClassName("slide");

    //make it circular
    if (i >= slides.length) {
        slide_index = 0;
    }
    if (i < 0) {
        slide_index = slides.length - 1;
    }

    for (x = 0; x < slides.length; x++) {
        slides[x].style.display = 'none';
    }

    slides[slide_index].style.display = "block";
}

document.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key === 's') {
        event.preventDefault();
    }
});