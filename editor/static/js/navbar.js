let theme_button = document.getElementById("theme-button");

//called by theme button
function change_theme(opt) {
    //if classlist contains dark, the website has dark mode on
    theme_button.classList.toggle("dark");

    if (theme_button.classList.contains("dark")) {
        document.documentElement.setAttribute("theme", "dark");
        theme_button.innerHTML = '<p>&#X2600;</p>' //sun

        //editors
        if (opt === 0) change_editor_theme(0);
        if (opt === 1) change_example_editor_theme(0);
        
    } else {
        document.documentElement.setAttribute("theme", "default");
        theme_button.innerHTML = '<p>🌙&#Xfe0e;</p>' //moon
        
        //editors
        if (opt === 0) change_editor_theme(1);
        if (opt === 1) change_example_editor_theme(1);
    }
}

//changes the page, preserves the theme
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