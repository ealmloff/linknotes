import React, { useMemo, useState, useCallback } from 'react';
import { createEditor, Descendant, Transforms } from 'slate';
import { Slate, Editable, withReact, RenderElementProps } from 'slate-react';
import { BaseEditor } from 'slate';
import { ReactEditor } from 'slate-react';
// import { invoke } from '@tauri-apps/api/core'; // Import Tauri's invoke
import {core} from '@tauri-apps/api';


// Define custom types for the elements and text
type ParagraphElement = { type: 'paragraph'; children: CustomText[] };
type HeadingElement = { type: 'heading'; children: CustomText[] };
type CustomText = { text: string; bold?: boolean };

// Extend the Element type with custom element types
type CustomElement = ParagraphElement | HeadingElement;

// Extend the Editor type
type CustomEditor = BaseEditor & ReactEditor;

// Declare module augmentation for Slate to use custom types
declare module 'slate' {
  interface CustomTypes {
    Editor: CustomEditor;
    Element: CustomElement;
    Text: CustomText;
  }
}

const TextEditor: React.FC = () => {
  const editor = useMemo(() => withReact(createEditor()), []);
  console.log('TextEditor component rendered');

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

  const handleSaveNote = async () => {
    console.log('Preparing to save note:', title, value);
    const noteContent = JSON.stringify(value);
    const note = {
        title: title || 'Untitled',
        content: noteContent,
    };

    console.log('Invoke function:', core.invoke); // Check if invoke is defined
    try {
        // Check if the workplace ID can be retrieved
        // const workplaceId = await invoke('get_workplace_id');
        console.log('Workplace ID:', 0); // Log the workplace ID

        // Call the add_note function
        console.log('Calling add_note function...');
        console.log('Note:', note);
        var x = title ? `./${title}.txt` : './untitled.txt'
        console.log(note.title, note.content, x);
        const path = './test_note.txt';

        // Extract text from content
        const parsedContent = JSON.parse(note.content);
        let extractedText = '';
        parsedContent.forEach((block: any) => {
            if (block.type === 'paragraph') {
                block.children.forEach((child: any) => {
                    extractedText += child.text + ' ';
                });
            }
        });
        extractedText = extractedText.trim(); // Remove trailing space

        console.log(extractedText)

        const result = await core.invoke('add_note', {
            title: note.title,
            text: extractedText,
            workplace_id: 0,
            path: path,
        });

        console.log('Invoke result:', result);

        console.log('Note saved successfully');
    } catch (error) {
        console.error('Failed to save note:', error);
    }
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
            />
          </Slate>
        </div>
      </div>
    </div>
  );
};

export default TextEditor;
