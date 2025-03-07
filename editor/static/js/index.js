let user_code_textarea = document.getElementById("editor");
let code_output_textarea = document.getElementById("output");

var editor = CodeMirror.fromTextArea(document.getElementById("code"), {
    lineNumbers: true,
	autofocus: true,
    styleActiveLine: true,
});

editor.setSize("100%", "100%");

async function run_button_clicked() {
	const startTime = performance.now();

	// let temp_output = "Cons(3, Cons(2, Cons(1, Nil)))"
    // code_output_textarea.value = temp_output;

	await wasm_bindgen('./pkg/editor_bg.wasm');

    let user_code = editor.getValue();
    const code_output = wasm_bindgen.rust_function(user_code);
    code_output_textarea.value = code_output;

	const endTime = performance.now();

	update_runtime(endTime - startTime);
	// update_runtime(5);

}

function update_runtime(runtime) {
	let runtime_div = document.getElementById("runtime");

	runtime_div.innerHTML = "Runtime: " + runtime + " ms";
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

/*
code for halftime presentation mockup:

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
main = print(ReverseList(Cons(1, Cons(2, Cons(3, Nil)))));


let temp_output = "Cons(3, Cons(2, Cons(1, Nil)))"
code_output_textarea.value = temp_output;
*/