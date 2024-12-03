import React, { useMemo, useState, useCallback, useEffect } from 'react';
import { createEditor, Descendant, Transforms, BaseEditor, Node } from 'slate';
import { Slate, Editable, withReact, ReactEditor, RenderElementProps } from 'slate-react';
import { HistoryEditor, withHistory } from 'slate-history';
import { invoke } from "@tauri-apps/api/core";
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import Search from './Search';
import TagsPanel from './TagsPanel';
import './App.css';
import { FaTrash } from 'react-icons/fa';

// Type definitions
type ParagraphElement = { type: 'paragraph'; children: CustomText[] };
type HeadingElement = { type: 'heading'; children: CustomText[] };
type CustomText = { text: string; bold?: boolean };
type CustomElement = ParagraphElement | HeadingElement;
type CustomEditor = BaseEditor & ReactEditor & HistoryEditor;
const threshold = 20;

declare module 'slate' {
  interface CustomTypes {
    Editor: CustomEditor;
    Element: CustomElement;
    Text: CustomText;
  }
}

// Define the WorkspaceId type if it's not already defined
type WorkspaceId = number; // Adjust this based on the actual type

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
  const [workspaceId, setWorkspaceId] = useState<WorkspaceId>(0); // Use WorkspaceId type here
  const [tags, setTags] = useState<{ name: string, manual: boolean }[]>([]);
  const [selectedNoteId, setSelectedNoteId] = useState<number | null>(null);
  const [showConfirmation, setShowConfirmation] = useState<boolean>(false);
  const [cursorPosition, setCursorPosition] = useState<number>(0); // Add state for cursor position
  const [activeTab, setActiveTab] = useState<string>('saved'); // Track active tab state
  const [context, setContext] = useState<any>(null);  // State to store context result



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

  useEffect(() => {
    // Pass the text content and cursor position to the side panel
    const textContent = Node.string(editor);
    console.log("Cursor position:", cursorPosition);
    // toast.info(`Cursor position: ${cursorPosition}`);
    console.log("Text content:", textContent);
    // Implement the logic to pass the information to the side panel here
  }, [cursorPosition, value]);

  // Helper functions
  const loadWorkspace = async () => {
    try {
      const id = await invoke('get_workspace_id', { path: "./testing-workspace" }) as WorkspaceId; // Use WorkspaceId type here
      setWorkspaceId(id);
      loadSavedNotes(id);
    } catch (error) {
      console.error('Failed to load workspace:', error);
      toast.error('Failed to load workspace');
    }
  };

  const loadSavedNotes = async (workspaceId: WorkspaceId) => { // Use WorkspaceId type here
    try {
      const files = await invoke('files_in_workspace', { workspaceId }) as { document: {title: string, body: string}, tags: { name: string, manual: boolean }[] }[];
      const notes = files.map((doc) => {
        return { title: doc.document.title, content: doc.document.body, tags: doc.tags };
      });
      setSavedNotes(notes);
      // set cursor to 0
      setCursorPosition(0);
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
    setTags([]); // Clear the tags
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
            extractedText += child.text + '\n';
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

  const handleDeleteClick = (noteId: number) => {
    setSelectedNoteId(noteId);
    setShowConfirmation(true);
  };

  const confirmDelete = async (noteId: number) => {
    try {
      console.log('Confirming delete for note ID:', noteId);
      console.log('Deleting note with title:', savedNotes[noteId]?.title, 'Workspace ID:', workspaceId);
  
      // Call the Tauri backend
      const result = await invoke('remove_note', { title: savedNotes[noteId].title, workspaceId });
      console.log('Backend response:', result);
  
      // Update the state
      setSavedNotes(prevNotes => {
        const updatedNotes = prevNotes.filter((_, index) => index !== noteId);
        console.log('Updated notes:', updatedNotes);
        setTitle(''); // Clear the title
        setTags([]);
        setValue(INITIAL_VALUE); // Clear the content
        editor.children = INITIAL_VALUE; // Reset the editor content
        Transforms.select(editor, { path: [0, 0], offset: 0 }); 
        return updatedNotes;
      });
  
      setSelectedNoteId(null);
      setShowConfirmation(false);
      toast.success('Note deleted successfully');
    } catch (error) {
      console.error('Failed to delete note:', error);
      toast.error('Failed to delete note');
    }
  };

  const cancelDelete = () => {
    setSelectedNoteId(null);
    setShowConfirmation(false);
  };

  // File change handler
  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = async (e) => {
        const text = e.target?.result as string;
        const newValue: Descendant[] = text.split('\n').map(line => ({
          type: 'paragraph' as const,
          children: [{ text: line }],
        }));
        setValue(newValue);
        editor.children = newValue;
        Transforms.select(editor, { path: [0, 0], offset: 0 });
      };
      reader.readAsText(file);
    }
  };

  // Event handler to update cursor position
  const handleCursorPosition = () => {
    const { selection } = editor;
    if (selection) {
      const { anchor } = selection;
      let cursorIndex = 0;

      // Traverse the nodes to calculate the cursor position
      for (let i = 0; i < anchor.path[0]; i++) {
        const node = editor.children[i];
        cursorIndex += Node.string(node).length;
      }
      cursorIndex += anchor.offset;

      // only update the cursor position if it has changed by a threshold value
      // if (Math.abs(cursorIndex - cursorPosition) > threshold) {
        setCursorPosition(cursorIndex);
        getContextResult(cursorIndex, Node.string(editor));
      // }
      

      // setCursorPosition(cursorIndex);
    }
  };

  const getContextResult = async (cursor_utf16_index: number, document_text: string) => {
  try {
    // Call the backend function and pass cursor position and text content
    let documentTitle = null;
    // Don't try to provide context for an empty document
    if (document_text.length  == 0) {
      return;
    }
    if (title.length > 0) {
      documentTitle = title;
    }
    const contextResult = await invoke('context_search', {
      documentText: document_text,
      cursorUtf16Index: cursor_utf16_index,
      results: 1, // Adjust the number of results as needed
      contextSentences: 3, // Adjust the number of context sentences as needed
      workspaceId: workspaceId,
      documentTitle,
    });
    console.log("Received context result:", contextResult);
    // You can now display the context result (e.g., in a toast or sidebar)
    displayContextResult(contextResult);
  } catch (error) {
    console.error("Failed to get context:", error);
    toast.error(`Failed to get context ${error}`);
  }
};
  
  // Display the context result (for example, using a toast or modal)
  const displayContextResult = (contextResult: any) => {
    toast.info(`Context: ${contextResult}`);

    // const { distance, title, relevant_range, text } = contextResult;
  
    // For example, display this in a toast notification
    // toast.info(
    //   `Context from ${title}: ${text} (Distance: ${distance}, Range: ${relevant_range})`
    // );
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

  const handleTabClick = (tabId: string) => {
    setActiveTab(tabId); // Set the active tab to the clicked tab
  };


  return (
    <div className="text-editor">
      <ToastContainer />
      <div className="editor-header">
        <div className="search-wrapper">
          <Search onTagClick={handleTagClick} selectedTags={selectedTags} searchQuery={searchQuery} setSearchQuery={setSearchQuery} workspace_id={workspaceId} handleNoteSelect={handleNoteSelect} />
        </div>
        <div className="editor-title">LinkedNotes</div>
        <button onClick={handleNewNote} className="new-note-btn">New +</button>
        <button onClick={handleSaveNote} className="save-note-btn">Save</button>
        <button onClick={() => document.getElementById('fileInput')?.click()}>Import</button>
        <input
          type="file"
          id="fileInput"
          style={{ display: 'none' }}
          accept=".txt"
          onChange={handleFileChange}
        />
      </div>
      <div className="editor-body">
        <div className="sidepanel">
        <div className="sidepanel-tabs">
            <div
              className={`sidepanel-tab ${activeTab === 'saved' ? 'active' : ''}`}
              onClick={() => handleTabClick('saved')}
            >
              Saved Notes
            </div>
            <div
              className={`sidepanel-tab ${activeTab === 'other' ? 'active' : ''}`}
              onClick={() => handleTabClick('other')}
            >
              Other Panel
            </div>
          </div>
          <div className="sidepanel-content" id="saved-notes" style={{ display: activeTab === 'saved' ? 'block' : 'none' }}>
          {savedNotes.length > 0 ? (
            <ul>
              {savedNotes.map((note, index) => (
                <li key={index} className="note-item" onClick={() => handleNoteSelect(note.title)}>
                <div className="note-content">
                  <strong>{note.title}</strong>
                  <span>{note.content.substring(0, 50)}...</span>
                </div>
                <div className="note-actions">
                  <FaTrash className="trash-icon" onClick={(e) => { e.stopPropagation(); handleDeleteClick(index); }} />
                </div>
              </li>
              ))}
            </ul>
          ) : (
            <p>No notes saved yet.</p>
          )}
        </div>
          <div className="sidepanel-content" id="other-panel" style={{ display: activeTab === 'other' ? 'block' : 'none' }}>
                <p>No notes saved yet.</p>
          </div>
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
              onKeyUp={handleCursorPosition} // Add onKeyUp event
              onMouseUp={handleCursorPosition} // Add onMouseUp event
            />
          </Slate>
        </div>
      </div>
      {showConfirmation && (
        <>
          <div className="confirmation-overlay"></div>
          <div className="confirmation-dialog">
            <p>Are you sure you want to delete this note?</p>
            <button onClick={() => confirmDelete(selectedNoteId!)}>Yes</button>
            <button className="cancel" onClick={cancelDelete}>No</button>
          </div>
        </>
      )}
    </div>
  );
};

export default TextEditor;