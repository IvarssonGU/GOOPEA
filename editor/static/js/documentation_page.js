let toggle_headers = document.getElementsByClassName("toggle-header");
let toggle_contents = document.getElementsByClassName("toggle-content");

window.onload = function() {
    for (var i = 0; i < toggle_headers.length; i++) {
        toggle_contents[i].style.display = 'block'; //change back to none later
        toggle_headers[i].classList.toggle("open"); //remove later

        toggle_headers[i].addEventListener("click", function() {
            this.classList.toggle("open");

            let content = this.nextElementSibling;
            if (content.style.display === 'none') content.style.display = 'block';
            else content.style.display = 'none';
        });
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