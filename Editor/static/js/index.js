import { basicSetup, EditorState, EditorView } from '@codemirror/basic-setup';

let user_code_textarea = document.getElementById("editor");
let code_output_textarea = document.getElementById("output");
let run_button = document.getElementById("run-button");

run_button.addEventListener("click", run_button_clicked);


const initialState = EditorState.create({
  doc: '',
  extensions: [basicSetup],
});

const view = new EditorView({
  parent: document.getElementById('editor'),
  state: initialState,
});

async function run_button_clicked() {
    await wasm_bindgen('./pkg/editor_bg.wasm');

    let user_code = user_code_textarea.getValue();
    const code_output = wasm_bindgen.rust_function(user_code);
    code_output_textarea.value = code_output;
}

// document.addEventListener('DOMContentLoaded', () => {
//     user_code_textarea.addEventListener('change', resize_textareas, false)
//     code_output_textarea.addEventListener('change', resize_textareas, false)
    
// }, false)

// function resize_textareas() {
    
// }