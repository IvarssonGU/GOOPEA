let toggle_headers = document.getElementsByClassName("toggle-header");
let toggle_contents = document.getElementsByClassName("toggle-content");

//loading and unloading
document.addEventListener("DOMContentLoaded", () => {
    for (var i = 0; i < toggle_headers.length; i++) {
        toggle_contents[i].style.display = 'block'; //change back to none for starting collapsed p1
        toggle_headers[i].classList.toggle("open"); //remove for starting collapsed p2

        //add on-click function to toggle contents
        toggle_headers[i].addEventListener("click", function() {
            this.classList.toggle("open");

            let content = this.nextElementSibling;
            if (content.style.display === 'none') content.style.display = 'block';
            else content.style.display = 'none';
        });
    }

    if ("theme" in localStorage) {
        if (localStorage.getItem("theme") === "dark") {
            document.documentElement.setAttribute("theme", "dark");
            change_theme(-1);
        } else {
            document.documentElement.setAttribute("theme", "default");
        }
    }
});

window.onbeforeunload = function() {    
    if (document.getElementById("theme-button").classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }
};

//expand/collapse all headers/content
function expand_all() {
    for (var i = 0; i < toggle_headers.length; i++) {
        if (!toggle_headers[i].classList.contains("open")) {
            toggle_headers[i].classList.toggle("open");
            toggle_contents[i].style.display = 'block';
        }
    }
}
function collapse_all() {
    for (var i = 0; i < toggle_headers.length; i++) {
        if (toggle_headers[i].classList.contains("open")) {
            toggle_headers[i].classList.toggle("open");
            toggle_contents[i].style.display = 'none';
        }
    }
}

//changes ctrl-s default action
document.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key === 's') {
        event.preventDefault();
    }
});