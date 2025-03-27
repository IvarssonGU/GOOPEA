let theme_button = document.getElementById("theme-button");
let nav = document.getElementById("nav");
let styled_buttons = document.getElementsByClassName("styled-button");

let editor_body = document.getElementById("editor-body");
let editor_output = document.getElementById("output");
let editor_memory = document.getElementById("memory");
let editor_runtime = document.getElementById("runtime");

let examples_body = document.getElementById("examples-body");
let examples_output = document.getElementById("output-field");

let documentation_body = document.getElementById("documentation-body");
let documentation_headers = document.getElementsByClassName("toggle-header");
let documentation_mark = document.getElementsByTagName("mark");

function change_theme(opt) {
    //if classlist contains dark, the playground has dark mode on
    theme_button.classList.toggle("dark");
    nav.classList.toggle("dark");

    for (let x = 0; x < styled_buttons.length; x++) {
        styled_buttons[x].classList.toggle("dark");
    }

    switch (opt) {
        case 0: //case current page is editor
            editor_body.classList.toggle("dark");
            editor_output.classList.toggle("dark");
            editor_memory.classList.toggle("dark");
            editor_runtime.classList.toggle("dark");
            break;
        case 1: //case current page is examples
            examples_body.classList.toggle("dark");
            examples_output.classList.toggle("dark");
            break;
        case 2: //case current page is documentation
            documentation_body.classList.toggle("dark");
            for (let i = 0; i < documentation_headers.length; i++) {
                documentation_headers[i].classList.toggle("dark");
            }
            for (let i = 0; i < documentation_mark.length; i++) {
                documentation_mark[i].classList.toggle("dark");
            }
            break;
        default:
            break;
    }

    if (theme_button.classList.contains("dark")) {
        // dark_theme = true;
        theme_button.innerHTML = '<p>&#X2600;</p>' //sun

        //editors
        if (opt === 0) change_editor_theme(0);
        if (opt === 1) change_example_editor_theme(0);
        
        //
        
    } else {
        // dark_theme = false;
        theme_button.innerHTML = '<p>&#X263E;</p>' //moon

        //editors
        if (opt === 0) change_editor_theme(1);
        if (opt === 1) change_example_editor_theme(1);
    }
}

function change_page(opt) {
    if (theme_button.classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }

    switch(opt) {
        case 0:
            window.location.href = "index.html";
            break;
        case 1:
            window.location.href = "example_page.html";
            break;
        case 2:
            window.location.href = "documentation_page.html";
            break;                
        default:
            break;
    }
}