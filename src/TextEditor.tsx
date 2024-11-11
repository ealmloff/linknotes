import React, { useMemo, useState, useCallback, useEffect } from 'react';
import { createEditor, Descendant, Transforms, BaseEditor } from 'slate';
import { Slate, Editable, withReact, ReactEditor, RenderElementProps } from 'slate-react';
import { HistoryEditor, withHistory } from 'slate-history';
import { invoke } from "@tauri-apps/api/core";
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import Search from './Search';
import TagsPanel from './TagsPanel';

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
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [savedNotes, setSavedNotes] = useState<{ title: string, content: string, tags: { name: string, manual: boolean }[] }[]>([]);
  const [workspaceId, setWorkspaceId] = useState<number>(0);
  const [tags, setTags] = useState<{ name: string, manual: boolean }[]>([]);

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
      // TODO: Save the state here. This used to try to call the non-existent save_title and save_content functions on the backend.
      // This should call save_note instead.
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
      const id = await invoke('get_workspace_id', { path: "./testing-workspace" }) as number;
      setWorkspaceId(id);
      loadSavedNotes(id);
    } catch (error) {
      console.error('Failed to load workspace:', error);
      toast.error('Failed to load workspace');
    }
  };

  const loadSavedNotes = async (workspaceId: number) => {
    try {
      const files = await invoke('files_in_workspace', { workspaceId }) as { document: {title: string, body: string}, tags: { name: string, manual: boolean }[] }[];
      const notes = files.map((doc) => {
        return { title: doc.document.title, content: doc.document.title, tags: doc.tags };
      });
      setSavedNotes(notes);
    } catch (error) {
      console.error('Failed to load saved notes:', error);
      toast.error('Failed to load saved notes');
    }
  };

  const loadSavedState = async () => {
    try {
      // // TODO: these functions don't exist on the backend? Do they need to? You can get the content from the read_note function.
      // const savedTitle = await invoke('load_title');
      // const savedContent = await invoke('load_content');
      // if (savedTitle) setTitle(savedTitle as string);
      // if (savedContent) setValue(JSON.parse(savedContent as string));
    } catch (error) {
      console.error('Failed to load saved state:', error);
    }
  };

  const loadTags = async (noteTitle: string) => {
    try {
      const tags = await invoke('get_tags', { title: noteTitle, workspaceId }) as { name: string, manual: boolean }[];
      setTags(tags);
    } catch (error) {
      console.error('Failed to load tags:', error);
      toast.error('Failed to load tags');
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
      tags: [],
    };
  
    try {
      console.log('Calling save_note function...');
  
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
  
      await invoke('save_note', {
        title: note.title,
        text: extractedText,
        workspaceId,
      });
  
      setSavedNotes(prevNotes => {
        const index = prevNotes.findIndex(n => n.title === title);
        if (index !== -1) {
          // Update existing note
          const updatedNotes = [...prevNotes];
          updatedNotes[index] = { title: note.title, content: extractedText, tags: note.tags };
          return updatedNotes;
        } else {
          // Add new note
          return [...prevNotes, { title: note.title, content: extractedText, tags: note.tags }];
        }
      });

      await loadTags(note.title);
  
      toast.success(`Note saved successfully`);
    } catch (error) {
      console.error('Failed to save note:', error);
      toast.error(`Failed to save note ${JSON.stringify(error)}`);
    }
  }, [title, value, workspaceId]);

  const handleNoteSelect = useCallback(async (title: string) => {
    try {
      console.log("Attempting to read file:", title);
      const content = await invoke('read_note', { title, workspaceId }) as { document: {title: string, body: string}, tags: { name: string, manual: boolean }[] };
      console.log("Received content from backend:", content);

      // Set the title
      setTitle(title);

      // Create a new Slate-compatible value
      const newValue: Descendant[] = content.document.body.split('\n').map(line => {
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

      await loadTags(title);

      toast.success(`Loaded note: ${title}`);
    } catch (error) {
      console.error('Failed to load note:', error);
      toast.error(`Failed to load note: ${error}`);
    }
  }, [editor, setTitle, setValue, workspaceId]);

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

  const handleTagClick = (tag: string) => {
    setSelectedTags(prevTags => {
      if (prevTags.includes(tag)) {
        return prevTags.filter(t => t !== tag);
      } else {
        return [...prevTags, tag];
      }
    });
    setSearchQuery(tag); // Update search query when a tag is clicked
  };

  const handleAddTag = async (newTag: string) => {
    try {
      await invoke('set_tags', {
        title,
        tags: [...tags, { name: newTag, manual: true }],
        workspaceId,
      });
      await loadTags(title);
    } catch (error) {
      console.error('Failed to add tag:', error);
      toast.error('Failed to add tag');
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
        <div className="search-wrapper">
          <Search onTagClick={handleTagClick} selectedTags={selectedTags} searchQuery={searchQuery} setSearchQuery={setSearchQuery} />
        </div>
        <div className="editor-title">LinkedNotes</div>
        <button onClick={handleNewNote} className="new-note-btn">New +</button>
        <button onClick={handleSaveNote} className="save-note-btn">Save</button>
      </div>
      <div className="editor-body">
        <div className="sidepanel">
          <h2>Your Notes</h2>
          {savedNotes.length > 0 ? (
            <ul>
              {savedNotes.map((note, index) => (
                <li key={index} onClick={() => handleNoteSelect(note.title)}>
                  <strong>{note.title} with tags {note.tags.map(tag => tag.name).join(', ')}</strong>
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
            <div className="title-tags-container">
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Title"
                className="title-input"
              />
              <TagsPanel onTagClick={handleTagClick} tags={tags} onAddTag={handleAddTag} />
            </div>
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