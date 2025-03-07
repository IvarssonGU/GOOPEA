CodeMirror.defineSimpleMode("GOOPEA", {
    start: [
        {regex: /(?:fip|match|enum|let|in|Cons)\b/, token: "keyword"},
        {regex: /true|false|Nil/, token: "atom"},
        {regex: /0x[a-f\d]+|[-+]?(?:\.\d+|\d+\.?\d*)(?:e[-+]?\d+)?/i, token: "number"},
        {regex: /\/\/.*/, token: "comment"},
        {regex: /\/\*/, token: "comment", next: "comment"},
        {regex: /[-+\/*]+/, token: "operator"},
        {regex: /[\{\[\(]/, token: "variable-2", indent: true},
        {regex: /[\}\]\)]/, token: "variable-2", dedent: true},
        {regex: /[a-z$][\w$]*/, token: "variable"},
        {regex: /:|=/, token: "variable-3"},
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