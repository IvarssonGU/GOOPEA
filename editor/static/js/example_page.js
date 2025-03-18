let example_select = document.getElementById("examples");
let output_field = document.getElementById("output-field");

var code_field = CodeMirror.fromTextArea(document.getElementById("code-field"), {
    lineNumbers: true,
    styleActiveLine: true,
    readOnly: true,
    mode: "GOOPEA",
});

//change codemirror editor to fit code heightwise
// code_field.setSize("100%", "100%");

window.onload = example_dropdown_changed();


//change example showed from dropdown
// example_select.addEventListener(onChange, (event) => {
function example_dropdown_changed() {
    switch(example_select.value) {
        case "reverse":
            code_field.setValue( 
`enum List = Nil, Cons(Int, List);

fip (List, List): List
ReverseHelper(list, acc) =
        match list {
            Nil: acc,
            Cons(x, xs): ReverseHelper(xs, Cons(x, acc))
        };

fip List: List
ReverseList list = ReverseHelper(list, Nil);

fip (): ()
Main = Print(ReverseList(Cons(1, Cons(2, Cons(3, Nil)))));`);
            output_field.value = "Cons(3, Cons(2, Cons(1, Nil)))";
            break;
        case "treeflip":
            code_field.setValue(
`enum Tree = Empty, Node(Tree, Int, Tree);

(): Tree
build = Node(Node(Node(Empty, 15, Empty), 10, Node(Empty, 52, Empty)), 5, Node(Node(Empty, 69, Empty), 23, Empty));

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
GetMinusFive = -5;

(): Int
Subtract = 2 - 1;

(): Int
main = 3 * (1 + 15/5) - (6/(2+1))*6;`);
            output_field.value = "0";
            break;
        case "complex-match":
            code_field.setValue( 
`enum Animal = Cat, Dog;

Animal: Animal
Convert x = match x {
    Cat: Dog,
    Dog: Cat
};

(): Int
MatchUnbox = match (1, 2) {
    (x, y): x + y
};

(): (Int, Int)
Coord = (7, 5);

(): Int
LetUnbox = let x = 3 in x;

(): Int
MatchUnbox2 = match Coord {
    (x, y): x - y
};

Int: Int
Fib i = match i {
    0: 1,
    1: 1,
    n: Fib (i-1) + Fib(i - 2)
};

(): Animal
Main = match Convert Cat {
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
            output_field.value = "output here";
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
Down(t, ctx) =
    match t {
        Bin(l, r):
            Down(l, BinL(ctx, r)),
        Tip x: App(Tip(x + 1), ctx)
    };

fip (Tree, TZipper): Tree
App(t, ctx) =
    match ctx {
        Top: t,
        BinR(l, up):
            App(Bin(l, t), up),
        BinL(up, r):
            Down(r, BinR(t, up))
    };

fip Tree: Tree
TMap t = Down(t, Top);`);
            output_field.value = "output here";
            break;
        default:
            code_field.setValue( 
`enum List = Nil, Cons(Int, List);

fip (List, List): List
ReverseHelper(list, acc) =
        match list {
            Nil: acc,
            Cons(x, xs): ReverseHelper(xs, Cons(x, acc))
        };

fip List: List
ReverseList list = ReverseHelper(list, Nil);

fip (): ()
Main = Print(ReverseList(Cons(1, Cons(2, Cons(3, Nil)))));`);
            output_field.value = "Cons(3, Cons(2, Cons(1, Nil)))";
    }

    code_field.refresh();
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