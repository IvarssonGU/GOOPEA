let user_code_textarea = document.getElementById("editor");
let code_output_textarea = document.getElementById("output");
// let cm = CodeMirror.fromTextArea(user_code_textarea);
// cm.setSize("100%", "100%");

// document.addEventListener("onchange", resize);

var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
	autofocus: true,
    styleActiveLine: true,
    // mode: "htmlmixed",
});

editor.setSize("100%", "100%");
// editor.focus();

async function run_button_clicked() {
	await wasm_bindgen('./pkg/editor_bg.wasm');

    let user_code = editor.getValue();
    const code_output = wasm_bindgen.rust_function(user_code);
    code_output_textarea.value = code_output;
}

// function resize() {
// 	code_output_textarea.style.height = code_output_textarea.scrollHeight;
// }


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