let toggle_headers = document.getElementsByClassName("toggle-header");
let toggle_contents = document.getElementsByClassName("toggle-content");

window.onload = function() {
    for (var x = 0; x < toggle_headers.length; x++) {
        toggle_contents[x].style.display = 'none';

        toggle_headers[x].addEventListener("click", function() {
            this.classList.toggle("open");

            let content = this.nextElementSibling;
            if (content.style.display === 'none') content.style.display = 'block';
            else content.style.display = 'none';
        });
    }
};