import React, { useMemo, useState, useCallback, useEffect } from 'react';
import { createEditor, Descendant, Transforms, BaseEditor } from 'slate';
import { Slate, Editable, withReact, ReactEditor, RenderElementProps } from 'slate-react';
import { HistoryEditor, withHistory } from 'slate-history';
import { invoke } from "@tauri-apps/api/core";
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

// Type definitions
type ParagraphElement = { type: 'paragraph'; children: CustomText[] };
type HeadingElement = { type: 'heading'; children: CustomText[] };
type CustomText = { text: string; bold?: boolean };
type CustomElement = ParagraphElement | HeadingElement;
type CustomEditor = BaseEditor & ReactEditor & HistoryEditor;

declare module 'slate' {
  interface CustomTypes {
    Editor: CustomEditor;
    Element: CustomElement;
    Text: CustomText;
  }
}

// Constants
const INITIAL_VALUE: Descendant[] = [
  {
    type: 'paragraph' as const,
    children: [{ text: '' }],
  },
];

const TextEditor: React.FC = () => {
  // State
  const [value, setValue] = useState<Descendant[]>(INITIAL_VALUE);
  const [title, setTitle] = useState<string>('');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [savedNotes, setSavedNotes] = useState<{ title: string, content: string, path: string }[]>([]);
  const [workspaceId, setWorkspaceId] = useState<string>('');

  // Memoized values
  const editor = useMemo(() => withHistory(withReact(createEditor())), []);

  // Effects
  useEffect(() => {
    loadWorkspace();
  }, []);

  useEffect(() => {
    loadSavedState();
  }, []);

  useEffect(() => {
    if (title && value !== INITIAL_VALUE) {
      saveState();
    }
  }, [title, value]);

  useEffect(() => {
    console.log("Value changed:", value);
  }, [value]);

  useEffect(() => {
    console.log("Title changed:", title);
  }, [title]);

  // Helper functions
  const loadWorkspace = async () => {
    try {
      const id = await invoke('get_workspace_id', { path: "./testing-workspace" });
      setWorkspaceId(id as string);
      loadSavedNotes(id as string);
    } catch (error) {
      console.error('Failed to load workspace:', error);
      toast.error('Failed to load workspace');
    }
  };

  const loadSavedNotes = async (workspaceId: string) => {
    try {
      const files = await invoke('files_in_workspace', { workspaceId }) as string[];
      const notes = await Promise.all(files.map(async (path) => {
        const content = await invoke('read_file', { path }) as string;
        const title = path.split('/').pop()?.replace('.txt', '') || 'Untitled';
        return { title, content, path };
      }));
      setSavedNotes(notes);
    } catch (error) {
      console.error('Failed to load saved notes:', error);
      toast.error('Failed to load saved notes');
    }
  };

  const loadSavedState = async () => {
    try {
      const savedTitle = await invoke('load_title');
      const savedContent = await invoke('load_content');
      if (savedTitle) setTitle(savedTitle as string);
      if (savedContent) setValue(JSON.parse(savedContent as string));
    } catch (error) {
      console.error('Failed to load saved state:', error);
    }
  };

  const saveState = async () => {
    try {
      await invoke('save_title', { title });
      await invoke('save_content', { content: JSON.stringify(value) });
    } catch (error) {
      console.error('Failed to save state:', error);
      // Remove the toast notification from here
    }
  };

  // Event handlers
  const handleNewNote = () => {
    setTitle('');
    setValue(INITIAL_VALUE);
    editor.children = INITIAL_VALUE;
    Transforms.select(editor, { path: [0, 0], offset: 0 });
    toast.success('New note created');
  };

  const handleSaveNote = useCallback(async () => {
    toast.info('Preparing to save note');
    const noteContent = JSON.stringify(value);
    const note = {
      title: title || 'Untitled',
      content: noteContent,
    };
  
    try {
      console.log('Calling add_note function...');
      const path = `./testing-workspace/notes/${note.title}.txt`;
  
      const parsedContent = JSON.parse(note.content);
      let extractedText = '';
      parsedContent.forEach((block: any) => {
        if (block.type === 'paragraph') {
          block.children.forEach((child: any) => {
            extractedText += child.text + ' ';
          });
        }
      });
      extractedText = extractedText.trim();
  
      await invoke('add_note', {
        title: note.title,
        text: extractedText,
        workspaceId,
        path,
      });
  
      setSavedNotes(prevNotes => {
        const index = prevNotes.findIndex(n => n.path === path);
        if (index !== -1) {
          // Update existing note
          const updatedNotes = [...prevNotes];
          updatedNotes[index] = { title: note.title, content: extractedText, path };
          return updatedNotes;
        } else {
          // Add new note
          return [...prevNotes, { title: note.title, content: extractedText, path }];
        }
      });
  
      toast.success(`Note saved successfully`);
    } catch (error) {
      console.error('Failed to save note:', error);
      toast.error(`Failed to save note ${JSON.stringify(error)}`);
    }
  }, [title, value, workspaceId]);

  const handleNoteSelect = useCallback(async (path: string) => {
    try {
      console.log("Attempting to read file:", path);
      const content = await invoke('read_file', { path }) as string;
      console.log("Received content from backend:", content);

      // Set the title
      const title = path.split('/').pop()?.replace('.txt', '') || 'Untitled';
      setTitle(title);

      // Create a new Slate-compatible value
      const newValue: Descendant[] = content.split('\n').map(line => {
        if (line.startsWith('#')) {
          return {
            type: 'heading' as const,
            children: [{ text: line.slice(1).trim() }],
          };
        }
        return {
          type: 'paragraph' as const,
          children: [{ text: line }],
        };
      });

      // Update the editor's content
      setValue(newValue);
      editor.children = newValue;
      Transforms.select(editor, { path: [0, 0], offset: 0 });

      toast.success(`Loaded note: ${title}`);
    } catch (error) {
      console.error('Failed to load note:', error);
      toast.error(`Failed to load note: ${error}`);
    }
  }, [editor, setTitle, setValue]);

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


  // Render functions
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
  <h2>Your Notes</h2>
  {savedNotes.length > 0 ? (
    <ul>
      {savedNotes.map((note, index) => (
        <li key={index} onClick={() => handleNoteSelect(note.path)}>
          <strong>{note.title}</strong>
          <span>{note.content.substring(0, 50)}...</span>
        </li>
      ))}
    </ul>
  ) : (
    <p>No notes saved yet.</p>
  )}
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
          <Slate 
  editor={editor} 
  initialValue={value} 
  onChange={(newValue) => setValue(newValue)}
>
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