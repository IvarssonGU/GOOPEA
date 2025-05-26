let output_textarea = document.getElementById("output");
let debug_textarea = document.getElementById("debug");
let steps_container = document.getElementById("steps-container");

let visualization_div = document.getElementById("visualization-div");

let output_button = document.getElementById("output-tab-button");
let compiler_button = document.getElementById("compiler-tab-button");
let debug_tab_button = document.getElementById("debug-tab-button");

let diff_button = document.getElementById("diff-tab");
let steps_button = document.getElementById("steps-tab");

let compiler_message = "Click Compile, Run, or Debug to get a compiler message"
let ccode_value = "Click Compile, Run, or Debug to view C code";
let stir_value = "Click Compile, Run, or Debug to view stir";
let reuse_value = "Click Compile, Run, or Debug to view reuse";
let rc_value = "Click Compile, Run, or Debug to view rc";

let error_highlight = null;
let debugging = false;

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
    indentUnit: 4,
    smartIndent: false,
});

if (!navigator.userAgent.includes("Chrome")) {
    editor.setSize("100%", "100%");
}
editor.on('keyup', function (event) {
    if (!(event.ctrlKey && event.key === 'q')) continuous_compilation();
})

function autocomplete_hints(cm) {
    let replaced_with_space = editor.getValue().replace(/(\s|[^A-Za-z_\d*]|(?<![A-Za-z_])\d+(?![A-Za-z_]))+/g, ' ');
    replaced_with_space = replaced_with_space.concat(" match let in fip enum Int"); //keywords
    let completion_values = [...new Set(replaced_with_space.split(' '))];

    CodeMirror.showHint(cm, function () {
        var cursor = editor.getCursor(), line = editor.getLine(cursor.line)
        var start = cursor.ch, end = cursor.ch
        while (start && /\w/.test(line.charAt(start - 1))) --start
        while (end < line.length && /\w/.test(line.charAt(end))) ++end
        var word = line.slice(start, end)//gets word cursor is on
        let hints = [];
        for (var i = 0; i < completion_values.length; i++) {
            if (completion_values[i].includes(word) && completion_values[i].indexOf(word) === 0) {
                if (completion_values[i] === word) {
                    completion_values.splice(i, 1);
                } else {
                    hints.push(completion_values[i])
                }
            }
        }
        return {list: hints.length ? hints : completion_values,
                from: CodeMirror.Pos(cursor.line, start),
                to: CodeMirror.Pos(cursor.line, end)};
    }, {completeSingle: true});
}

//loading and unloading
document.addEventListener("DOMContentLoaded", () => {
    output_textarea.innerHTML = "";
    debugging = false;
    selected_changed(0);
    selected_changed(1);
    selected_changed(2);
    
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
            case "diff": switch_compiler_tab(1); break;
            case "steps": switch_compiler_tab(2); break;
            default: switch_compiler_tab(2);
        }
    } else {
        switch_compiler_tab(2);
    }

    debug_textarea.value = "";

    if ("code" in localStorage) {
        if ("exported" in localStorage) {
            if (localStorage.getItem("exported") === "true") {
                editor.setValue(localStorage.getItem("exported_code"));
                document.getElementById("restore-button").classList.toggle("hide");
                localStorage.setItem("exported", "false");
            } else {
                editor.setValue(localStorage.getItem("code"));
            }
        } else {
            editor.setValue(localStorage.getItem("code"));
        }
    }

    if ("theme" in localStorage) {
        if (localStorage.getItem("theme") === "dark") {
            document.documentElement.setAttribute("theme", "dark");
            change_theme(0);
        } else {
            document.documentElement.setAttribute("theme", "default");
        }
    }

    continuous_compilation();
});

window.onbeforeunload = function() {
    localStorage.setItem("code", editor.getValue());
   
    if (output_button.classList.contains("current-tab")) localStorage.setItem("tab", "output");
    if (compiler_button.classList.contains("current-tab")) localStorage.setItem("tab", "compiler");
    if (debug_tab_button.classList.contains("current-tab")) localStorage.setItem("tab", "debug");

    console.log(localStorage.getItem("tab"));
    
    if (diff_button.classList.contains("current-tab")) localStorage.setItem("compiler", "diff");
    if (steps_button.classList.contains("current-tab")) localStorage.setItem("compiler", "steps");

    if (document.getElementById("theme-button").classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }
};

async function continuous_compilation() {
    if (await compile_and_populate()) {
        write_compilation_message();
    } else {
        write_error_message();
    }
}

async function compile_and_populate() {
    //returns true = error free
    error_free = false;

    if (error_highlight != null) error_highlight.clear();

    if (document.getElementById("copy-button").classList.contains("hide")) document.getElementById("copy-button").classList.toggle("hide");

    await wasm_bindgen('./pkg/editor_bg.wasm');

    let code = editor.getValue();

    let result = wasm_bindgen.compile(code);

    if (result.is_ok()) {
        let prog = result.unwrap()

        //compilation (get all the steps and assign them to their vars here)
        compiler_message = "-- executed without errors --";
        ccode_value = prog.get_c_code();
        stir_value = prog.get_stir_str();
        reuse_value = prog.get_reuse_str();
        rc_value = prog.get_rc_str();
        debug_textarea.value = "";

        error_free = true;
    } else {
        let err = result.unwrap_err();

        //populate error messages
        compiler_message = err.get_error_string();
        ccode_value = "error C code";
        stir_value = "error stir (not implemented yet)";
        reuse_value = "error reuse (not implemented yet)";
        rc_value = "error rc (not implemented yet)";
        debug_textarea.value = "doesn't compile";

        //get error char, search for string, highlight string (not implemented)
        if (err.has_source()) {
            let source = err.get_source()
            let from = {line: source.start_line-1, ch: source.start_line_char-1};
            let to = {line: source.end_line-1, ch: source.end_line_char-1};

            error_highlight = editor.markText(from, to, {className: "error-highlight"});
        }
    }

    //populate the compilation selections
    selected_changed(0);
    selected_changed(1);
    selected_changed(2);

    return error_free;
}

//clear-debug-run button functions
async function run_button_clicked() {
    output_textarea.innerHTML = "";

	const startTime = performance.now();

	await wasm_bindgen('./pkg/editor_bg.wasm');

    if (!document.getElementById("restore-button").classList.contains("hide")) document.getElementById("restore-button").classList.toggle("hide");

    let code = editor.getValue();
    localStorage.setItem("code", code);

    if (await compile_and_populate()) {
        //run until done interpreter
        wasm_bindgen.start_interpreter(code);
        wasm_bindgen.get_run();

        let run_value = wasm_bindgen.get_interpreter_output();
        run_value = run_value.substring(1, run_value.length-1); //remove ""
        let output_value = "";
        if (run_value === "") {
            output_value = `<span style=\"color: green;\">${compiler_message}</span>`;
        } else {
            output_value = 
`<span style=\"white-space: pre-line;\">${run_value}

<span style=\"color: green;\">${compiler_message}</span></span>`;
        }
        output_textarea.innerHTML = output_value;

        switch_tab(0);
    } else {
        write_error_message();

        switch_tab(0);
    }

	const endTime = performance.now();

	update_runtime(endTime - startTime);
}

function write_compilation_message() {
    output_textarea.innerHTML = `<span style=\"white-space: pre-wrap;\"><span style=\"color: green;\">-- compiled without error --</span></span>`;
}
function write_error_message() {
    output_textarea.innerHTML = `<span style=\"white-space: pre-wrap;\"><span style=\"color: red;\">-- compiled with error --</span>
    
${compiler_message}</span>`;
}

function update_runtime(runtime) {
	let runtime_div = document.getElementById("runtime");

	runtime_div.innerHTML = "Runtime: " + runtime + " ms";
}

function clear_button_clicked() {
    editor.setValue("");
    output_textarea.innerHTML = "";
}

function restore_code() {
    editor.setValue(localStorage.getItem("code"));
    continuous_compilation();
    document.getElementById("restore-button").classList.toggle("hide");
}

async function debug_button_clicked() {
    await wasm_bindgen('./pkg/editor_bg.wasm');
    // let debug_buttons = document.getElementsByClassName("debug-button");

    let code = editor.getValue();
    localStorage.setItem("code", code);
    if (!document.getElementById("restore-button").classList.contains("hide")) document.getElementById("restore-button").classList.toggle("hide");

    if (await compile_and_populate()) {
        write_compilation_message();
        //display starting state
        wasm_bindgen.start_interpreter(code);
        if (!debug_textarea.classList.contains("hide")) debug_textarea.classList.toggle("hide");
        if (visualization_div.classList.contains("hide")) visualization_div.classList.toggle("hide");
        update_visualization();

        debugging = true;
        // if (debugging) for (var i = 0; i < debug_buttons.length; i++) debug_buttons[i].classList.toggle("hide");
        
        switch_tab(1);

    } else {
        if (debug_textarea.classList.contains("hide")) debug_textarea.classList.toggle("hide");
        if (!visualization_div.classList.contains("hide")) visualization_div.classList.toggle("hide");

        write_error_message();

        switch_tab(0);
    }
}

async function compile_button_clicked() {
    let code = editor.getValue();
    localStorage.setItem("code", code);
    if (!document.getElementById("restore-button").classList.contains("hide")) document.getElementById("restore-button").classList.toggle("hide");

    if (await compile_and_populate()) {
        write_compilation_message();
        switch_tab(2);
        switch_compiler_tab(2);
    } else {
        write_error_message();

        switch_tab(0);
    }
}


//interpreter functions
function step_back_clicked() {
    wasm_bindgen.get_back_step();
    update_visualization();
}
function step_forward_clicked() {
    wasm_bindgen.get_one_step();
    update_visualization();
}
function run_mem_clicked() {
    wasm_bindgen.get_until_mem();
    update_visualization();
}
function run_return_clicked() {
    wasm_bindgen.get_until_return();
    update_visualization();
}
function run_done_clicked() {
    wasm_bindgen.get_run();
    update_visualization();
}
function delta_data_clicked(){
    wasm_bindgen.get_delta_data();
    update_visualization();
}

//saves which tab is active
function save_state(opt) {
    localStorage.setItem("code", editor.getValue());

    if (output_button.classList.contains("current-tab")) localStorage.setItem("tab", "output");
    if (compiler_button.classList.contains("current-tab")) localStorage.setItem("tab", "compiler");
    if (debug_tab_button.classList.contains("current-tab")) localStorage.setItem("tab", "debug");

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
        if (debugging && !debug_buttons[0].classList.contains("hide")) for (var i = 0; i < debug_buttons.length; i++) debug_buttons[i].classList.toggle("hide");
        document.getElementById("debug-container").classList.toggle("hide");
    }
    if (compiler_button.classList.contains("current-tab")) {
        compiler_button.classList.toggle("current-tab");
        for (var i = 0; i < code_step_buttons.length; i++) code_step_buttons[i].classList.toggle("hide");
        
        //save current compiler tab
        if (diff_button.classList.contains("current-tab")) localStorage.setItem("compiler", "diff");
        if (steps_button.classList.contains("current-tab")) localStorage.setItem("compiler", "steps");
        //switch for ease
        switch_compiler_tab(2);
        steps_container.classList.toggle("hide");
    }

    switch (opt) {
        case 0: //switch to output
            output_button.classList.toggle("current-tab");
            output_textarea.classList.toggle("hide");
            localStorage.setItem("tab", "output");
            break;
        case 1: //switch to debugging
            debug_tab_button.classList.toggle("current-tab");
            if (debugging && debug_buttons[0].classList.contains("hide")) for (var i = 0; i < debug_buttons.length; i++) debug_buttons[i].classList.toggle("hide");
            document.getElementById("debug-container").classList.toggle("hide");
            localStorage.setItem("tab", "debug");
            break;
        case 2: //switch to compiler
            compiler_button.classList.toggle("current-tab");
            for (var i = 0; i < code_step_buttons.length; i++) code_step_buttons[i].classList.toggle("hide");
            steps_container.classList.toggle("hide");
            if ("compiler" in localStorage) {
                let current_tab = localStorage.getItem("compiler");
                switch (current_tab) {
                    case "diff": switch_compiler_tab(1); break;
                    case "steps": switch_compiler_tab(2); break;
                    default: switch_compiler_tab(2);
                }
            } else {
                switch_compiler_tab(2);
            }
            break;
        default:
            output_button.classList.toggle("current-tab");
            output_textarea.classList.toggle("hide");

    }
}

function switch_compiler_tab(opt) {
    //unselect previous tab
    if (steps_button.classList.contains("current-tab")) {
        steps_button.classList.toggle("current-tab");
        document.getElementById("steps-container").classList.toggle("hide");
    }
    if (diff_button.classList.contains("current-tab")) {
        diff_button.classList.toggle("current-tab");
        document.getElementById("diff-container").classList.toggle("hide");
    }

    //switch to clicked tab
    switch (opt) {
        case 1: //switch to diff
            diff_button.classList.toggle("current-tab");
            document.getElementById("diff-container").classList.toggle("hide");
            selected_changed(1);
            selected_changed(2);
            break;
        case 2: //switch to intermediate steps
            steps_button.classList.toggle("current-tab");
            steps_container.classList.toggle("hide");
            selected_changed(0);
            break;
        default:
            steps_button.classList.toggle("current-tab");
            steps_container.classList.toggle("hide");

    }
}

//dropdowns under compiler->diff view and in intermediate
function selected_changed(opt) {
    switch(opt) {
        case 0:
            selector = "step-select"
            target = "steps"
            break;
        case 1: 
            selector = "diff1-select";
            target = "diff1";
            break;
        case 2:
            selector = "diff2-select";
            target = "diff2";
            break;
        default:
            selector = "step-select"
            target = "steps"
    }

    let target_textarea = document.getElementById(target);

    switch(document.getElementById(selector).value) {
        case "c":
            target_textarea.value = ccode_value;
            break;
        case "stir":
            target_textarea.value = stir_value;
            break;
        case "reuse":
            target_textarea.value = reuse_value;
            break;
        case "rc":
            target_textarea.value = rc_value;
            break;
        default:
            target_textarea.value = stir_value;
    }
}

function copy_step() {
    let copied_ack = document.getElementById("copied-ack");

    copied_ack.classList.toggle("appearing");
    navigator.clipboard.writeText(document.getElementById("steps").value);
    setTimeout(function() {copied_ack.classList.toggle("appearing");}, 1000); //untoggles after 1s
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
        
        compile_and_populate();
    }
    else if (event.ctrlKey && event.key === 'q') {
        event.preventDefault();
        run_button_clicked();
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

or:
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
