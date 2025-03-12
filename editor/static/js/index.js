let output_textarea = document.getElementById("output");

var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
	autofocus: true,
    styleActiveLine: true,
    mode: "GOOPEA",
    autoCloseBrackets: true,
    matchBrackets: true,
});

editor.setSize("100%", "100%");
editor.setOption("extraKeys", key_binds);

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

commands to run first time or when changing rust code: (make sure to be in /GOOPEA/editor)
cargo clean
cargo build
wasm-pack build --target no-modules
basic-http-server

command to run: (make sure to be in /GOOPEA/editor)
basic-http-server

TODO
- keymap & extrakeys in configuration of codemirror
- documentation page
    - syntax guide?
- more examples
- memory visualization instead of diagrams
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
