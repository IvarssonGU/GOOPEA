CodeMirror.defineSimpleMode("GOOPEA", {
    start: [
        {regex: /(?:fip|match|enum|let|in|Int)\b/, token: "keyword"},
        {regex: /Nil|Cons|Empty|Node/, token: "def"}, //constructors
        {regex: /true|false/, token: "atom"},
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
