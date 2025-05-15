// CodeMirror, copyright (c) by Marijn Haverbeke and others
// Distributed under an MIT license: https://codemirror.net/5/LICENSE

// these are functions originially defined in keymap/sublime.js
//  - decided that all of the sublime keymap was overkill so decided
//      on a couple that were "most important"

var key_binds = {
    "Alt-Up": function(cm) { //swap line up
        if (cm.isReadOnly()) return CodeMirror.Pass
        var ranges = cm.listSelections(), linesToMove = [], at = cm.firstLine() - 1, newSels = [];
        for (var i = 0; i < ranges.length; i++) {
            var range = ranges[i], from = range.from().line - 1, to = range.to().line;
            newSels.push({anchor: CodeMirror.Pos(range.anchor.line - 1, range.anchor.ch),
                            head: CodeMirror.Pos(range.head.line - 1, range.head.ch)});
            if (range.to().ch == 0 && !range.empty()) --to;
            if (from > at) linesToMove.push(from, to);
            else if (linesToMove.length) linesToMove[linesToMove.length - 1] = to;
            at = to;
        }
        cm.operation(function() {
        for (var i = 0; i < linesToMove.length; i += 2) {
            var from = linesToMove[i], to = linesToMove[i + 1];
            var line = cm.getLine(from);
            cm.replaceRange("", CodeMirror.Pos(from, 0), CodeMirror.Pos(from + 1, 0), "+swapLine");
            if (to > cm.lastLine())
            cm.replaceRange("\n" + line, CodeMirror.Pos(cm.lastLine()), null, "+swapLine");
            else
            cm.replaceRange(line + "\n", CodeMirror.Pos(to, 0), null, "+swapLine");
        }
        cm.setSelections(newSels);
        cm.scrollIntoView();
        });
    },
    "Alt-Down": function(cm) { //swap line down
        if (cm.isReadOnly()) return CodeMirror.Pass
        var ranges = cm.listSelections(), linesToMove = [], at = cm.lastLine() + 1;
        for (var i = ranges.length - 1; i >= 0; i--) {
            var range = ranges[i], from = range.to().line + 1, to = range.from().line;
            if (range.to().ch == 0 && !range.empty()) from--;
            if (from < at) linesToMove.push(from, to);
            else if (linesToMove.length) linesToMove[linesToMove.length - 1] = to;
            at = to;
        }
        cm.operation(function() {
            for (var i = linesToMove.length - 2; i >= 0; i -= 2) {
                var from = linesToMove[i], to = linesToMove[i + 1];
                var line = cm.getLine(from);
                if (from == cm.lastLine())
                cm.replaceRange("", CodeMirror.Pos(from - 1), CodeMirror.Pos(from), "+swapLine");
                else
                cm.replaceRange("", CodeMirror.Pos(from, 0), CodeMirror.Pos(from + 1, 0), "+swapLine");
                cm.replaceRange(line + "\n", CodeMirror.Pos(to, 0), null, "+swapLine");
            }
            cm.scrollIntoView();
        });
    },
    "Shift-Alt-Down": function(cm) { //copy line down
        cm.operation(function() {
            var rangeCount = cm.listSelections().length;
            for (var i = 0; i < rangeCount; i++) {
                var range = cm.listSelections()[i];
                if (range.empty())
                cm.replaceRange(cm.getLine(range.head.line) + "\n", CodeMirror.Pos(range.head.line, 0));
                else
                cm.replaceRange(cm.getRange(range.from(), range.to()), range.from());
            }
            cm.scrollIntoView();
            });
    },
    "Ctrl-/": function(cm) { //toggle line comment
        cm.toggleComment();
    },
    "Ctrl-Space": function(cm) {
        autocomplete_hints(cm);
    },
};