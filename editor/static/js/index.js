let output_textarea = document.getElementById("output");

var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
	autofocus: true,
    styleActiveLine: true,
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