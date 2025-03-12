CodeMirror.defineSimpleMode("GOOPEA", {
    start: [
        {regex: /(?:fip|match|enum|let|in)\b/, token: "keyword"},
        {regex: /Nil|Cons/, token: "def"},
        {regex: /true|false/, token: "atom"},
        {regex: /[\{\[\()]/, token: "bracket", indent: true},
        {regex: /[\}\]\)]/, token: "bracket", dedent: true},
        // {regex: /[A-Z][a-z]*([A-Z][a-z]*)*(?=\()/, token: "def"},
        {regex: /0x[a-f\d]+|[-+]?(?:\.\d+|\d+\.?\d*)(?:e[-+]?\d+)?/i, token: "number"},
        {regex: /[A-Z][a-z]*([A-Z][a-z]*)*\b/, token: "variable-2"},
        {regex: /\/\/.*/, token: "comment"},
        {regex: /\/\*/, token: "comment", next: "comment"},
        {regex: /[-+\/*]+/, token: "operator"},
        {regex: /[a-z$][\w$]*/, token: "variable"},
        {regex: /:|=/, token: "punctuation"},
        // {regex: /:|=/, token: "variable-3"},
    ],
    comment: [
        {regex: /.*?\*\//, token: "comment", next: "start"},
        {regex: /.*/, token: "comment"}
    ],
    meta: {
        dontIndentStates: ["comment"],
        lineComment: "//"
    }
});

let output_textarea = document.getElementById("output");

var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
	autofocus: true,
    styleActiveLine: true,
    mode: "GOOPEA",
    autoCloseBrackets: true,
});

editor.setSize("100%", "100%");

window.onload = function() {
    if ("code" in sessionStorage) {
        editor.setValue(sessionStorage.getItem("code"));
    }
};

window.onbeforeunload = function() {
    sessionStorage.setItem("code", editor.getValue());
};

async function run_button_clicked() {
	const startTime = performance.now();

	await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();
    sessionStorage.setItem("code", code);
    const output = wasm_bindgen.rust_function(code);
    output_textarea.value = output;

	const endTime = performance.now();

	update_runtime(endTime - startTime);
}

function update_runtime(runtime) {
	let runtime_div = document.getElementById("runtime");

	runtime_div.innerHTML = "Runtime: " + runtime + " ms";
}

function clear_button_clicked() {
    editor.setValue("");
}

function save_code(opt) {
    sessionStorage.setItem("code", editor.getValue());

    switch(opt) {
        case 0:
            window.location.href = "example_page.html";
            break;
        case 1:
            window.location.href = "documentation_page.html";
            break;
        default:
            window.location.href = "index.html";
    }
}



/*
install if necessary:
cargo install wasm-pack
cargo install basic-http-server

commands to run:
cargo clean
cargo build
wasm-pack build --target no-modules
basic-http-server

to look into later
keymap & extrakeys in configuration of codemirror
*/


/* syntax highlighting showcase

//this is a line comment

0987834582755654565 //numbers

fip match enum in let //"keywords"

Cons Nil //"def" constructors

true false //"atom"

EverythingThatStartsWithCapitalAndOnlyContainsLetters 	//"variable-2"

() {} [] //"bracket"

= : //"punctuation"

+ - / * //"operator"

the rest //variable (black)

enum List = Nil, Cons(Int, List);

fip (List, List): List
ReverseHelper(list, acc) =
        match list {
            Nil: acc,
            Cons(x, xs): ReverseHelper(xs, Cons(x, acc))
        };

fip List: List
ReverseList list = ReverseHelper(list, Nil);

fip (): ()
Main = Print(ReverseList(Cons(1, Cons(2, Cons(3, Nil))))); 

*/
