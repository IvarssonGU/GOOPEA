import { basicSetup, EditorState, EditorView } from '@codemirror/basic-setup';

const initialState = EditorState.create({
  doc: '',
  extensions: [basicSetup],
});

const view = new EditorView({
  parent: document.getElementById('editor'),
  state: initialState,
});