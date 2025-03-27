let output_textarea = document.getElementById("output");
let memory_textarea = document.getElementById("memory");

var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
	autofocus: true,
    styleActiveLine: true,
    mode: "GOOPEA",
    autoCloseBrackets: true,
    matchBrackets: true,
    extraKeys: key_binds,
});

editor.setSize("100%", "100%");

window.onload = function() {
    memory_textarea.style.display = 'none';
    if ("code" in localStorage) {
        editor.setValue(localStorage.getItem("code"));
    }
    if ("theme" in localStorage) {
        let theme = localStorage.getItem("theme");
        if (theme === "dark") {
            change_theme(0);
        }
    }
};
// document.addEventListener("DOMContentLoaded", function() {
// document.onreadystatechange = function() {
//     if ("theme" in localStorage) {
//         let theme = localStorage.getItem("theme");
//         if (theme === "dark") {
//             change_theme(0);
//         }
//     }
// }

window.onbeforeunload = function() {
    localStorage.setItem("code", editor.getValue());
    
    if (document.getElementById("editor-body").classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }
};

async function run_button_clicked() {
	const startTime = performance.now();

	await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();
    localStorage.setItem("code", code);
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
    localStorage.setItem("code", editor.getValue());
    change_page(-1);

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

function switch_to_output() {
    document.getElementById("output-button").classList.toggle("current-tab");
    document.getElementById("memory-button").classList.toggle("current-tab");
    
    memory_textarea.style.display = 'none';
    output_textarea.style.display = 'block';
}
function switch_to_memory() {
    document.getElementById("output-button").classList.toggle("current-tab");
    document.getElementById("memory-button").classList.toggle("current-tab");

    output_textarea.style.display = 'none';
    memory_textarea.style.display = 'block';
}

function change_editor_theme(opt) {
    switch (opt) {
        case 0: //dark theme
            editor.setOption("theme", "3024-night");
            break;
        case 1: //light theme
            editor.setOption("theme", "default");
            break;
        default:
            editor.setOption("theme", "default");   
    }
}

document.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key === 's') {
        event.preventDefault();

        //copy editor text to clipboard
        navigator.clipboard.writeText(editor.getValue()); 
    }
});


/*
(make sure to be in /GOOPEA/editor)
install if necessary:
    cargo install wasm-pack
    cargo install basic-http-server
commands to run first time or when changing rust code:
    cargo clean
    cargo build
    wasm-pack build --target no-modules
    basic-http-server
command to run otherwise: 
    basic-http-server

or: (linux)
    GOOPEA.sh
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

the rest //"variable" (black)

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
