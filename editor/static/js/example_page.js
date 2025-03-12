CodeMirror.defineSimpleMode("GOOPEA", {
    start: [
        {regex: /(?:fip|match|enum|let|in)\b/, token: "keyword"},
        {regex: /Nil|Cons/, token: "atom"},
        {regex: /true|false/, token: "def"},
        {regex: /[\{\[\()]/, token: "bracket", indent: true},
        {regex: /[\}\]\)]/, token: "bracket", dedent: true},
        // {regex: /[A-Z][a-z]*([A-Z][a-z]*)*(?=\()/, token: "def"},
        {regex: /0x[a-f\d]+|[-+]?(?:\.\d+|\d+\.?\d*)(?:e[-+]?\d+)?/i, token: "number"},
        {regex: /[A-Z][a-z]*([A-Z][a-z]*)*\b/, token: "variable-2"},
        {regex: /\/\/.*/, token: "comment"},
        {regex: /\/\*/, token: "comment", next: "comment"},
        {regex: /[-+\/*]+/, token: "operator"},
        {regex: /[a-z$][\w$]*/, token: "variable"},
        {regex: /:|=/, token: "punctuation"},
        // {regex: /:|=/, token: "variable-3"},
    ],
    comment: [
        {regex: /.*?\*\//, token: "comment", next: "start"},
        {regex: /.*/, token: "comment"}
    ],
    meta: {
        dontIndentStates: ["comment"],
        lineComment: "//"
    }
});

var example1_code = CodeMirror.fromTextArea(document.getElementById("example1-code"), {
    lineNumbers: true,
    styleActiveLine: true,
    readOnly: true,
    autoCloseBrackets: true,
    mode: "GOOPEA",
});

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