var example1_code = CodeMirror.fromTextArea(document.getElementById("example1-code"), {
    lineNumbers: true,
    styleActiveLine: true,
    readOnly: true,
    mode: "GOOPEA",
});

//slideshow
let slide_index = 0;
show_slide(slide_index);

function change_slide(n) {
    show_slide(slide_index += n);
}

function show_slide(i) {
    let slides = document.getElementsByClassName("slide");

    //make it circular
    if (i >= slides.length) {
        slide_index = 0;
    }
    if (i < 0) {
        slide_index = slides.length - 1;
    }

    for (x = 0; x < slides.length; x++) {
        slides[x].style.display = 'none';
    }

    slides[slide_index].style.display = "block";
}

document.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key === 's') {
        event.preventDefault();
    }
});