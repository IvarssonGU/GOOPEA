let user_code_textarea = document.getElementById("editor");
let code_output_textarea = document.getElementById("output");

var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
    styleActiveLine: true,
    mode: "htmlmixed",
});

async function run_button_clicked() {
	await wasm_bindgen('./pkg/editor_bg.wasm');

    let user_code = editor.getValue();
    const code_output = wasm_bindgen.rust_function(user_code);
    code_output_textarea.value = code_output;
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
*/