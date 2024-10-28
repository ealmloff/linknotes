import React, { useMemo, useState, useCallback, useEffect } from 'react';
import { createEditor, Descendant, Transforms } from 'slate';
import { Slate, Editable, withReact, RenderElementProps } from 'slate-react';
import { BaseEditor } from 'slate';
import { ReactEditor } from 'slate-react';
import { HistoryEditor, withHistory } from 'slate-history';
import { invoke } from "@tauri-apps/api/core";
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

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

  const [files, setFiles] = useState<string[]>([]);

  useEffect(() => {
    const fetchFiles = async () => {
      const workspaceId = await invoke('get_workspace_id', { path: "./testing-workspace" });
      const files: string[] = await invoke('files_in_workspace', {
        workspaceId,
      });
      setFiles(files);
    };
    fetchFiles();
  },
  []);

  const handleNewNote = () => {
    setTitle('');
    setValue([{ type: 'paragraph', children: [{ text: '' }] }]);
    Transforms.select(editor, { anchor: { path: [0, 0], offset: 0 }, focus: { path: [0, 0], offset: 0 } });
    toast.success('New note created');
  };

  const handleSaveNote = async () => {
    toast.info('Preparing to save note');
    console.log('Preparing to save note:', title, value);
    const noteContent = JSON.stringify(value);
    const note = {
        title: title || 'Untitled',
        content: noteContent,
    };
  
    console.log('Invoke function:', invoke); // Check if invoke is defined
    try {
        // Check if the workplace ID can be retrieved
        const workplaceId = await invoke('get_workspace_id', { path: "./testing-workspace" });
        console.log('Workplace ID:', 0); // Log the workplace ID
        toast.info(`Workspace Loaded: ${JSON.stringify(workplaceId)}`);

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

        await invoke('add_note', {
            title: note.title,
            text: extractedText,
            workspaceId: workplaceId,
            path: path,
        });
        toast.success(`Note saved successfully`);
    } catch (error) {
        console.error('Failed to save note:', error);
        toast.error(`Failed to save note ${JSON.stringify(error)}`);
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

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.metaKey && event.key === 'z') {
      event.preventDefault();
      editor.undo();
      toast.info('Undo action performed');
    } else if (event.metaKey && event.key === 's') {
      event.preventDefault();
      handleSaveNote();
    }
  };

  return (
    <div className="text-editor">
      <ToastContainer />
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
          {(files.length > 0 ? files.map((file: string) => (
            <p>{file}</p>
          )) : <p>No files found</p>)}
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