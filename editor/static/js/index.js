let output_textarea = document.getElementById("output");
let debug_textarea = document.getElementById("debug");
let messages_textarea = document.getElementById("messages");
let c_textarea = document.getElementById("c-code");
let diff1_textarea = document.getElementById("diff1");
let diff2_textarea = document.getElementById("diff2");
let step1_textarea = document.getElementById("step1");
let step2_textarea = document.getElementById("step2");
let step3_textarea = document.getElementById("step3");

let output_button = document.getElementById("output-tab-button");
let compiler_button = document.getElementById("compiler-tab-button");
let debug_tab_button = document.getElementById("debug-tab-button");

let messages_button = document.getElementById("messages-tab");
let ccode_button = document.getElementById("ccode-tab");
let diff_button = document.getElementById("diff-tab");
let step1_button = document.getElementById("step1-tab");
let step2_button = document.getElementById("step2-tab");
let step3_button = document.getElementById("step3-tab");

let diff1_select = document.getElementById("diff1-select");
let diff2_select = document.getElementById("diff2-select");

let compiler_message = "Click Compile, Run, or Debug to get a compiler message"
let ccode_value = "Click Compile, Run, or Debug to view C code";
let step1_value = "Click Compile, Run, or Debug to view step1";
let step2_value = "Click Compile, Run, or Debug to view step2";
let step3_value = "Click Compile, Run, or Debug to view step3";

//establish codemirror editor
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


//loading and unloading
document.addEventListener("DOMContentLoaded", () => {
    output_textarea.value = "";
    c_textarea.value = "";
    // messages_textarea.style.display = 'none';
    // debug_textarea.style.display = 'none';
    // c_textarea.style.display = 'none';
    // diff1_textarea.style.display = 'none';
    // diff2_textarea.style.display = 'none';
    // step1_textarea.style.display = 'none';
    // step2_textarea.style.display = 'none';
    // step3_textarea.style.display = 'none';
    diff1_selected();
    diff2_selected();
    
    if ("tab" in localStorage) {
        let current_tab = localStorage.getItem("tab");
        switch (current_tab) {
            case "output": switch_tab(0); break;
            case "compiler": switch_tab(1); break;
            case "debug": switch_tab(2); break;
            default: switch_tab(0);
        }
    } else {
        switch_tab(0);
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

window.onbeforeunload = function() {
    localStorage.setItem("code", editor.getValue());
   
    if (output_button.classList.contains("current-tab")) localStorage.setItem("tab", "output");
    if (compiler_button.classList.contains("current-tab")) localStorage.setItem("tab", "compiler");
    if (debug_tab_button.classList.contains("current-tab")) localStorage.setItem("tab", "debug");
    
    if (document.getElementById("theme-button").classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }
};

async function compile_and_populate() {
    await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();

    try {
        //compilation (get all the steps and assign them to their vars here)
        compiler_message = "looks good";
        ccode_value = wasm_bindgen.get_c_code(code);
        step1_value = "compiled step1 (not implemented yet)";
        step2_value = "compiled step2 (not implemented yet)";
        step3_value = "compiled step3 (not implemented yet)";
    } catch(error) {
        //populate error messages
        compiler_message = "error message";
        ccode_value = "error C code";
        step1_value = "error step1 (not implemented yet)";
        step2_value = "error step2 (not implemented yet)";
        step3_value = "error step3 (not implemented yet)";
        output_textarea.value = "doesn't compile"
        debug_textarea.value = "doesn't compile";
    }
    
    //assign to textareas
    messages_textarea.value = compiler_message;
    c_textarea.value = ccode_value;
    step1_textarea.value = step1_value;
    step2_textarea.value = step2_value;
    step3_textarea.value = step3_value;

    //populate the different compiler steps here
    diff1_selected();
    diff2_selected();
}

//clear-debug-run button functions
async function run_button_clicked() {
    output_textarea.value = "";

	const startTime = performance.now();

	await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();
    localStorage.setItem("code", code);

    compile_and_populate();

    try {
        //get output (not implemented)
        output_textarea.value = "output not implemented";

        //show only the final debug print
        wasm_bindgen.start_interpreter(code);
        debug_textarea.value = wasm_bindgen.get_run();

        switch_tab(0);
    } catch(error) {
        output_textarea.value = "doesn't compile"
        debug_textarea.value = "doesn't compile";
        c_textarea.value = "error message?";
        switch_tab(1);
    }

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
    diff1_textarea.value = "";
    diff2_textarea.value = "";
    step1_textarea.value = "";
    step2_textarea.value = "";
    step3_textarea.value = "";
    debug_textarea.value = "";
}

async function debug_button_clicked() {
    await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();
    localStorage.setItem("code", code);

    compile_and_populate();

    try {
        //display starting state
        wasm_bindgen.start_interpreter(code);
        debug_textarea.value = wasm_bindgen.get_state();

        switch_tab(2);
    } catch(error) {
        output_textarea.value = "doesn't compile"
        debug_textarea.value = "doesn't compile";
        c_textarea.value = "error message?";
        switch_tab(1);
    }
}

async function compile_button_clicked() {
    let code = editor.getValue();
    localStorage.setItem("code", code);

    compile_and_populate();

    debug_textarea.value = "";

    switch_tab(1);
}


//interpreter functions
function step_back_clicked() {
    debug_textarea.value = wasm_bindgen.get_back_step();
}
function step_forward_clicked() {
    debug_textarea.value = wasm_bindgen.get_one_step();
}
function run_mem_clicked() {
    debug_textarea.value = wasm_bindgen.get_until_mem();
}
function run_return_clicked() {
    debug_textarea.value = wasm_bindgen.get_until_return();
}
function run_done_clicked() {
    debug_textarea.value = wasm_bindgen.get_run();
}

//saves which tab is active
function save_state(opt) {
    localStorage.setItem("code", editor.getValue());

    if (output_button.classList.contains("current-tab")) localStorage.setItem("tab", "output");
    if (compiler_button.classList.contains("current-tab")) localStorage.setItem("tab", "compiler");
    if (debug_tab_button.classList.contains("current-tab")) localStorage.setItem("tab", "debug");

    change_page(opt);
}

//switch between the tabs on the right half of page
function switch_tab(opt) {    
    let debug_buttons = document.getElementsByClassName("debug-button");
    let code_step_buttons = document.getElementsByClassName("compiler-button")

    if (output_button.classList.contains("current-tab")) {
        output_button.classList.toggle("current-tab");
        output_textarea.classList.toggle("hide");
    }
    if (compiler_button.classList.contains("current-tab")) {
        compiler_button.classList.toggle("current-tab");
        for (var i = 0; i < code_step_buttons.length; i++) code_step_buttons[i].classList.toggle("hide");
        switch_compiler_tab(-2);
        messages_textarea.classList.toggle("hide");
    }
    if (debug_tab_button.classList.contains("current-tab")) {
        debug_tab_button.classList.toggle("current-tab");
        for (var i = 0; i < debug_buttons.length; i++) debug_buttons[i].classList.toggle("hide");
        debug_textarea.classList.toggle("hide");
    }

    switch (opt) {
        case 0: //switch to output
            output_button.classList.toggle("current-tab");
            output_textarea.classList.toggle("hide");
            break;
        case 1: //switch to compiler
            compiler_button.classList.toggle("current-tab");
            for (var i = 0; i < code_step_buttons.length; i++) code_step_buttons[i].classList.toggle("hide");
            messages_textarea.classList.toggle("hide");
            break;
        case 2: //switch to debug
            debug_tab_button.classList.toggle("current-tab");
            for (var i = 0; i < debug_buttons.length; i++) debug_buttons[i].classList.toggle("hide");
            debug_textarea.classList.toggle("hide");
            break;
        default:
            output_button.classList.toggle("current-tab");
            output_textarea.classList.toggle("hide");

    }
}

function switch_compiler_tab(opt) {
    if (messages_button.classList.contains("current-tab")) {
        messages_button.classList.toggle("current-tab");
        messages_textarea.classList.toggle("hide");
        if (messages_textarea.classList.contains("hide")) console.log("hidden");
    }
    if (ccode_button.classList.contains("current-tab")) {
        ccode_button.classList.toggle("current-tab");
        c_textarea.classList.toggle("hide");
    }
    if (diff_button.classList.contains("current-tab")) {
        diff_button.classList.toggle("current-tab");
        document.getElementById("diff-container").classList.toggle("hide");
    }
    if (step1_button.classList.contains("current-tab")) {
        step1_button.classList.toggle("current-tab");
        step1_textarea.classList.toggle("hide");
    }
    if (step2_button.classList.contains("current-tab")) {
        step2_button.classList.toggle("current-tab");
        step2_textarea.classList.toggle("hide");
    }
    if (step3_button.classList.contains("current-tab")) {
        step3_button.classList.toggle("current-tab");
        step3_textarea.classList.toggle("hide");
    }

    switch (opt) {
        case -2: //switch to messages
            messages_button.classList.toggle("current-tab");
            messages_textarea.classList.toggle("hide");
            break;
        case -1: //switch to diff
            diff_button.classList.toggle("current-tab");
            document.getElementById("diff-container").classList.toggle("hide");
            break;
        case 0: //switch to c code
            ccode_button.classList.toggle("current-tab");
            c_textarea.classList.toggle("hide");
            break;
        case 1: //switch to step1
            step1_button.classList.toggle("current-tab");
            step1_textarea.classList.toggle("hide");
            break;
        case 2: //switch to step2
            step2_button.classList.toggle("current-tab");
            step2_textarea.classList.toggle("hide");
            break;
        case 3: //switch to step3
            step3_button.classList.toggle("current-tab");
            step3_textarea.classList.toggle("hide");
            break;
        default:
            ccode_button.classList.toggle("current-tab");
            c_textarea.classList.toggle("hide");

    }
}

//dropdowns under compiler->diff view
function diff1_selected() {
    switch(diff1_select.value) {
        case "c":
            diff1_textarea.value = ccode_value;
            break;
        case "step1":
            diff1_textarea.value = step1_value;
            break;
        case "step2":
            diff1_textarea.value = step2_value;
            break;
        case "step3":
            diff1_textarea.value = step3_value;
            break;
        default:
            diff1_textarea.value = step1_value;
    }
}
function diff2_selected() {
    switch(diff2_select.value) {
        case "c":
            diff2_textarea.value = ccode_value;
            break;
        case "step1":
            diff2_textarea.value = step1_value;
            break;
        case "step2":
            diff2_textarea.value = step2_value;
            break;
        case "step3":
            diff2_textarea.value = step3_value;
            break;
        default:
            diff2_textarea.value = step1_value;
    }
}

//changes theme of codemirror editor
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

//changes default ctrl-s action
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
    cargo clean (optional)
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

*/
