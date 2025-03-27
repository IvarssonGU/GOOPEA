let toggle_headers = document.getElementsByClassName("toggle-header");
let toggle_contents = document.getElementsByClassName("toggle-content");

window.onload = function() {
    for (var i = 0; i < toggle_headers.length; i++) {
        toggle_contents[i].style.display = 'block'; //change back to none for starting collapsed p1
        toggle_headers[i].classList.toggle("open"); //remove for starting collapsed p2

        toggle_headers[i].addEventListener("click", function() {
            this.classList.toggle("open");

            let content = this.nextElementSibling;
            if (content.style.display === 'none') content.style.display = 'block';
            else content.style.display = 'none';
        });
    }

    if ("theme" in localStorage) {
        let theme = localStorage.getItem("theme");
        if (theme === "dark") {
            change_theme(2);
        }
    }
};

window.onbeforeunload = function() {    
    if (document.getElementById("documentation-body").classList.contains("dark")) {
        localStorage.setItem("theme", "dark");
    } else {
        localStorage.setItem("theme", "default");
    }
};

document.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key === 's') {
        event.preventDefault();
    }
});

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