import React, { useMemo, useState, useCallback } from 'react';
import { createEditor, Descendant, Transforms } from 'slate';
import { Slate, Editable, withReact, RenderElementProps } from 'slate-react';
import { BaseEditor } from 'slate';
import { ReactEditor } from 'slate-react';
import { HistoryEditor, withHistory } from 'slate-history';

// Define custom types for the elements and text
type ParagraphElement = { type: 'paragraph'; children: CustomText[] };
type HeadingElement = { type: 'heading'; children: CustomText[] };
type CustomText = { text: string; bold?: boolean };

// Extend the Element type with custom element types
type CustomElement = ParagraphElement | HeadingElement;

// Extend the Editor type
type CustomEditor = BaseEditor & ReactEditor & HistoryEditor;

// Declare module augmentation for Slate to use custom types
declare module 'slate' {
  interface CustomTypes {
    Editor: CustomEditor;
    Element: CustomElement;
    Text: CustomText;
  }
}

const TextEditor: React.FC = () => {
  const editor = useMemo(() => withHistory(withReact(createEditor())), []);

  const initialValue: Descendant[] = [
    {
      type: 'paragraph',
      children: [{ text: '' }],
    },
  ];

  const [value, setValue] = useState<Descendant[]>(initialValue);
  const [title, setTitle] = useState<string>(''); // State to store the heading/title
  const [searchQuery, setSearchQuery] = useState<string>(''); // State for the search bar

  const handleNewNote = () => {
    setTitle('');
    setValue([{ type: 'paragraph', children: [{ text: '' }] }]);
    Transforms.select(editor, { anchor: { path: [0, 0], offset: 0 }, focus: { path: [0, 0], offset: 0 } });
  };

  const handleSaveNote = () => {
    const note = {
      title: title || 'Untitled',
      content: value,
    };
    console.log('Saving note:', note);
  };

  const renderElement = useCallback((props: RenderElementProps) => {
    switch (props.element.type) {
      case 'heading':
        return <h1 {...props.attributes}>{props.children}</h1>;
      case 'paragraph':
      default:
        return <p {...props.attributes}>{props.children}</p>;
    }
  }, []);

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.metaKey && event.key === 'z') {
      event.preventDefault();
      editor.undo();
    } else if (event.metaKey && event.key === 's') {
      event.preventDefault();
      handleSaveNote();
    }
  };

  return (
    <div className="text-editor">
      <div className="editor-header">
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search..."
          className="search-input"
        />
        <button onClick={handleNewNote} className="new-note-btn">New +</button>
        <div className="editor-title">LinkedNotes</div>
        <button onClick={handleSaveNote} className="save-note-btn">Save</button>
      </div>
      <div className="editor-body">
        <div className="sidepanel">
          <h2>Panel</h2>
          <p>Some content here...</p>
        </div>
        <div className="editor-content">
          <div className="editor-input">
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Title"
              className="title-input"
            />
          </div>
          <Slate editor={editor} initialValue={value} onChange={setValue}>
            <Editable
              renderElement={renderElement}
              placeholder="Start typing your note here..."
              className="editable"
              onKeyDown={handleKeyDown}
            />
          </Slate>
        </div>
      </div>
    </div>
  );
};

export default TextEditor;