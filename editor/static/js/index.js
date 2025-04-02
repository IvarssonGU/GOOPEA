let output_textarea = document.getElementById("output");
let debug_textarea = document.getElementById("debug");
let c_textarea = document.getElementById("c-code");

let output_button = document.getElementById("output-button");
let c_code_button = document.getElementById("c-button");
let debug_button = document.getElementById("debug-button");

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

// window.onload = function() {
document.addEventListener("DOMContentLoaded", () => {
    debug_textarea.style.display = 'none';
    c_textarea.style.display = 'none';
    if ("tab" in localStorage) {
        let current_tab = localStorage.getItem("tab");
        switch (current_tab) {
            case "output": switch_tab(0); break;
            case "c_code": switch_tab(1); break;
            case "debug": switch_tab(2); break;
            default: switch_tab(0);
        }
    }

    debug_textarea.value = "";

    if ("code" in localStorage) {
        editor.setValue(localStorage.getItem("code"));
    }
    if ("theme" in localStorage) {
        if (localStorage.getItem("theme") === "dark") {
            // change_theme(0);
            document.documentElement.setAttribute("theme", "dark");
            change_theme(0);
        } else {
            document.documentElement.setAttribute("theme", "default");
        }
    }

});
// window.onload = function() {
//     document.getElementById("editor-body").classList.toggle("hidden");
//     // output_textarea.classList.toggle("hidden");
//     // c_textarea.classList.toggle("hidden");
//     // debug_textarea.classList.toggle("hidden");
    
// }
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
   
    if (output_button.classList.contains("current-tab")) localStorage.setItem("tab", "output");
    if (c_code_button.classList.contains("current-tab")) localStorage.setItem("tab", "c_code");
    if (debug_button.classList.contains("current-tab")) localStorage.setItem("tab", "debug");
    
    if (document.getElementById("theme-button").classList.contains("dark")) {
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

    const c_code = wasm_bindgen.get_c_code(code);
    c_textarea.value = c_code;

    wasm_bindgen.start_interpreter(code);

    debug_textarea.value = wasm_bindgen.get_state();

	const endTime = performance.now();

	update_runtime(endTime - startTime);
}

function update_runtime(runtime) {
	let runtime_div = document.getElementById("runtime");

	runtime_div.innerHTML = "Runtime: " + runtime + " ms";
}

function clear_button_clicked() {
    editor.setValue("");
    output_textarea.value = "";
    c_textarea.value = "";
    debug_textarea.value = "";
}

function one_step_clicked() {
    debug_textarea.value = wasm_bindgen.get_one_step();
}
function one_run_clicked() {
    debug_textarea.value = wasm_bindgen.get_run();
}

function save_state(opt) {
    localStorage.setItem("code", editor.getValue());

    if (output_button.classList.contains("current-tab")) localStorage.setItem("tab", "output");
    if (c_code_button.classList.contains("current-tab")) localStorage.setItem("tab", "c_code");
    if (debug_button.classList.contains("current-tab")) localStorage.setItem("tab", "debug");

    // console.log("ihweorfu");
    change_page(opt);
}

function switch_tab(opt) {    
    
    let step_button = document.getElementById("step-button")
    let rud_button = document.getElementById("rud-button")

    if (output_button.classList.contains("current-tab")) output_button.classList.toggle("current-tab");
    if (c_code_button.classList.contains("current-tab")) c_code_button.classList.toggle("current-tab");
    if (debug_button.classList.contains("current-tab")) {
        debug_button.classList.toggle("current-tab");
        step_button.classList.toggle("hide");
        rud_button.classList.toggle("hide");
    }

    switch (opt) {
        case 0: //switch to output
            output_button.classList.toggle("current-tab");
            output_textarea.style.display = 'block';
            c_textarea.style.display = 'none';
            debug_textarea.style.display = 'none';
            break;
        case 1: //switch to c code
            c_code_button.classList.toggle("current-tab");
            c_textarea.style.display = 'block'
            output_textarea.style.display = 'none';
            debug_textarea.style.display = 'none';
            break;
        case 2: //switch to debug
            debug_button.classList.toggle("current-tab");
            step_button.classList.toggle("hide");
            rud_button.classList.toggle("hide");
            debug_textarea.style.display = 'block';
            c_textarea.style.display = 'none';
            output_textarea.style.display = 'none';
            break;
        default:
            output_button.classList.toggle("current-tab");
            output_textarea.style.display = 'block';
            c_textarea.style.display = 'none';
            debug_textarea.style.display = 'none';

    }
}
// function switch_to_debug() {
//     document.getElementById("output-button").classList.toggle("current-tab");
//     document.getElementById("debug-button").classList.toggle("current-tab");

// }

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

/*
other notes
- shared js/css is in navbar files

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
