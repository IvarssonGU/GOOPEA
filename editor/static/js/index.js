let output_textarea = document.getElementById("output");
let debug_textarea = document.getElementById("debug");
let info_textarea = document.getElementById("info");

let output_button = document.getElementById("output-tab-button");
let compiler_button = document.getElementById("compiler-tab-button");
let debug_tab_button = document.getElementById("debug-tab-button");

let info_button = document.getElementById("info-tab");
let diff_button = document.getElementById("diff-tab");
let steps_button = document.getElementById("steps-tab");

let compiler_message = "Click Compile, Run, or Debug to get a compiler message"
let ccode_value = "Click Compile, Run, or Debug to view C code";
let step1_value = "Click Compile, Run, or Debug to view step1";
let step2_value = "Click Compile, Run, or Debug to view step2";
let step3_value = "Click Compile, Run, or Debug to view step3";

let error_highlight = null;

//establish codemirror editor
var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
	autofocus: true,
    styleActiveLine: true,
    mode: "GOOPEA",
    autoCloseBrackets: true,
    matchBrackets: true,
    extraKeys: key_binds,
    styleSelectedText: true,
});

editor.setSize("100%", "100%");


//loading and unloading
document.addEventListener("DOMContentLoaded", () => {
    output_textarea.value = "";
    info_textarea.value = "";
    selected_changed("diff1-select", "diff1");
    selected_changed("diff2-select", "diff2");
    selected_changed("step-select", "steps");
    
    if ("tab" in localStorage) {
        let current_tab = localStorage.getItem("tab");
        switch (current_tab) {
            case "output": switch_tab(0); break;
            case "debug": switch_tab(1); break;
            case "compiler": switch_tab(2); break;
            default: switch_tab(0);
        }
    } else {
        switch_tab(0);
    }

    if ("compiler" in localStorage) {
        let current_tab = localStorage.getItem("compiler");
        switch (current_tab) {
            case "info": switch_compiler_tab(0); break;
            case "diff": switch_compiler_tab(1); break;
            case "steps": switch_compiler_tab(2); break;
            default: switch_compiler_tab(0);
        }
    } else {
        switch_compiler_tab(0);
    }

    debug_textarea.value = "";

    if ("code" in localStorage) {
        editor.setValue(localStorage.getItem("code"));
    }

    if ("theme" in localStorage) {
        if (localStorage.getItem("theme") === "dark") {
            document.documentElement.setAttribute("theme", "dark");
            change_theme(0);
        } else {
            document.documentElement.setAttribute("theme", "default");
        }
    }

    switch_tab(2);
    continuous_compiling();
});

window.onbeforeunload = function() {
    localStorage.setItem("code", editor.getValue());
   
    if (output_button.classList.contains("current-tab")) localStorage.setItem("tab", "output");
    if (compiler_button.classList.contains("current-tab")) localStorage.setItem("tab", "compiler");
    if (debug_tab_button.classList.contains("current-tab")) localStorage.setItem("tab", "debug");
    
    if (info_button.classList.contains("current-tab")) localStorage.setItem("compiler", "info");
    if (diff_button.classList.contains("current-tab")) localStorage.setItem("compiler", "diff");
    if (steps_button.classList.contains("current-tab")) localStorage.setItem("compiler", "steps");

    if (document.getElementById("theme-button").classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }
};

async function compile_and_populate() {
    if (error_highlight != null) error_highlight.clear();

    await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();

    let result = wasm_bindgen.compile(code);

    if (result.is_ok()) {
        let prog = result.unwrap()

        //compilation (get all the steps and assign them to their vars here)
        ccode_value = prog.get_c_code();
        compiler_message = "looks good";
        step1_value = "compiled step1 (not implemented yet)";
        step2_value = "compiled step2 (not implemented yet)";
        step3_value = "compiled step3 (not implemented yet)";
    } else {
        let err = result.unwrap_err();

        //populate error messages
        compiler_message = err.get_error_string();
        ccode_value = "error C code";
        step1_value = "error step1 (not implemented yet)";
        step2_value = "error step2 (not implemented yet)";
        step3_value = "error step3 (not implemented yet)";
        output_textarea.value = "doesn't compile"
        debug_textarea.value = "doesn't compile";

        //get error char, search for string, highlight string (not implemented)
        if (err.has_source()) {
            let source = err.get_source()
            let from = {line: source.start_line-1, ch: source.start_line_char-1};
            let to = {line: source.end_line-1, ch: source.end_line_char-1};

            // console.log(from)
            // console.log(to)

            error_highlight = editor.markText(from, to, {className: "error-highlight"});
        }
    }
    
    //assign to textarea
    info_textarea.value = compiler_message;

    //populate the compiler selections
    selected_changed("diff1-select", "diff1");
    selected_changed("diff2-select", "diff2");
}

// CodeMirror.addEventListener("keyup", compile_and_populate());

// async function continuous_compiling() {
//     compile_and_populate();

//     // 1/sec
//     setTimeout(continuous_compiling, 1000);
// }

//clear-debug-run button functions
async function run_button_clicked() {
    output_textarea.value = "";

	const startTime = performance.now();

	await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();
    localStorage.setItem("code", code);

    await compile_and_populate();

    if (compiler_message === "looks good") {
        //get output (not implemented)
        output_textarea.value = "output not implemented";

        //show only the final debug print
        wasm_bindgen.start_interpreter(code);
        debug_textarea.value = wasm_bindgen.get_run();

        switch_tab(0);
    } else {
        switch_tab(2);
        switch_compiler_tab(0);
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
    debug_textarea.value = "";
    info_textarea = "";
}

async function debug_button_clicked() {
    await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();
    localStorage.setItem("code", code);

    await compile_and_populate();

    if (compiler_message === "looks good") {
        //display starting state
        wasm_bindgen.start_interpreter(code);
        debug_textarea.value = wasm_bindgen.get_state();

        switch_tab(1);
    } else {
        switch_tab(2);
        switch_compiler_tab(0);
    }
}

async function compile_button_clicked() {
    let code = editor.getValue();
    localStorage.setItem("code", code);

    await compile_and_populate();

    debug_textarea.value = "";

    switch_tab(2);
    switch_compiler_tab(0);
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

    if (info_button.classList.contains("current-tab")) localStorage.setItem("compiler", "info");
    if (diff_button.classList.contains("current-tab")) localStorage.setItem("compiler", "diff");
    if (steps_button.classList.contains("current-tab")) localStorage.setItem("compiler", "steps");

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
    if (debug_tab_button.classList.contains("current-tab")) {
        debug_tab_button.classList.toggle("current-tab");
        for (var i = 0; i < debug_buttons.length; i++) debug_buttons[i].classList.toggle("hide");
        debug_textarea.classList.toggle("hide");
    }
    if (compiler_button.classList.contains("current-tab")) {
        compiler_button.classList.toggle("current-tab");
        for (var i = 0; i < code_step_buttons.length; i++) code_step_buttons[i].classList.toggle("hide");
        info_textarea.classList.toggle("hide");

        //save current compiler tab
        if (info_button.classList.contains("current-tab")) localStorage.setItem("compiler", "info");
        if (diff_button.classList.contains("current-tab")) localStorage.setItem("compiler", "diff");
        if (steps_button.classList.contains("current-tab")) localStorage.setItem("compiler", "steps");
        //switch for ease
        switch_compiler_tab(0);
    }

    switch (opt) {
        case 0: //switch to output
            output_button.classList.toggle("current-tab");
            output_textarea.classList.toggle("hide");
            break;
        case 1: //switch to debugging
            debug_tab_button.classList.toggle("current-tab");
            for (var i = 0; i < debug_buttons.length; i++) debug_buttons[i].classList.toggle("hide");
            debug_textarea.classList.toggle("hide");
            break;
        case 2: //switch to compiler
            compiler_button.classList.toggle("current-tab");
            for (var i = 0; i < code_step_buttons.length; i++) code_step_buttons[i].classList.toggle("hide");
            info_textarea.classList.toggle("hide");
            if ("compiler" in localStorage) {
                let current_tab = localStorage.getItem("compiler");
                switch (current_tab) {
                    case "info": switch_compiler_tab(0); break;
                    case "diff": switch_compiler_tab(1); break;
                    case "steps": switch_compiler_tab(2); break;
                    default: switch_compiler_tab(0);
                }
            }
            break;
        default:
            output_button.classList.toggle("current-tab");
            output_textarea.classList.toggle("hide");

    }
}

function switch_compiler_tab(opt) {
    if (info_button.classList.contains("current-tab")) {
        info_button.classList.toggle("current-tab");
        info_textarea.classList.toggle("hide");
    }
    if (steps_button.classList.contains("current-tab")) {
        steps_button.classList.toggle("current-tab");
        document.getElementById("steps-container").classList.toggle("hide");
    }
    if (diff_button.classList.contains("current-tab")) {
        diff_button.classList.toggle("current-tab");
        document.getElementById("diff-container").classList.toggle("hide");
    }

    switch (opt) {
        case 0: //switch to info
            info_button.classList.toggle("current-tab");
            info_textarea.classList.toggle("hide");
            break;
        case 1: //switch to diff
            diff_button.classList.toggle("current-tab");
            document.getElementById("diff-container").classList.toggle("hide");
            break;
        case 2: //switch to intermediate steps
            steps_button.classList.toggle("current-tab");
            document.getElementById("steps-container").classList.toggle("hide");
            break;
        default:
            info_button.classList.toggle("current-tab");
            info_textarea.classList.toggle("hide");

    }
}

//dropdowns under compiler->diff view and in intermediate
function selected_changed(name, field) {
    let to_textarea = document.getElementById(field);

    switch(document.getElementById(name).value) {
        case "c":
            to_textarea.value = ccode_value;
            break;
        case "step1":
            to_textarea.value = step1_value;
            break;
        case "step2":
            to_textarea.value = step2_value;
            break;
        case "step3":
            to_textarea.value = step3_value;
            break;
        default:
            to_textarea.value = step1_value;
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
    // navigator.clipboard.writeText(editor.getValue()); 
        compile_button_clicked();
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

//for later: // CodeMirror.runMode(ccode_value, "application/c", steps_textarea);


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
