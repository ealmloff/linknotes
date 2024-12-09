/* Prologue Comments 
• Name of code artifact
- Text Entry
- Text Editor
- Text Deletion
- Keyboard Shortcuts
- Create Notes, Main Menu
- Support UI for Creating and Viewing Notes
- Connection with Local Storage
- Search Bars UI, Instant Search
- Note Tagging
- Search by Tags
- Automatic Note Tagging UI 
- Import Notes
- Live Linking Note Viewing
- Side Panel UI 
- Frontend Input of Cursor Position and Text
- Live Update of Related Notes

• Brief description of what the code does

This code is the main connection for the frontend UI and also serves as a connection to the backend
It creates the Slate editors, and creates functional buttons and code that connects. 

*Programmer's name
- Suhaan Syed, Siddh Bharucha, Tejaswi Nimmagadda, Evan Almoff, Trisha Sheth

*Date the code was created - 
- Oct 23. 2024

*Dates the code was revised
- Oct 24, 25, 26, 27, 30, 
- Nov 10, 11, 12, 19, 21, 24, 26
- Dec 5, 7, 8

*Brief description of each revision & author
Oct Revisions: 
- Basic creation, and frontend UI, Key Shortcuts, Storage, WorkspaceID,
- Toasts, addnote, deletenote, panel, 
Nov Revisions:
styling updates, Search UI, creating new tags, Auto Tagging, Enter Key,
- Import Notes, Delete button, icon, Cursor Tracking, Context Results
- Side Panel for Live Linking, Light Dark Mode
Dec Revisions:
- Logo and remaining Panel, Context Results, clearing Toasts, 
- Light mode UI, Color blind tagging

*Preconditions
- First, the application must be running in a React environment, 
as it utilizes React components and hooks extensively. Additionally, 
the Slate.js library is required for rich text editing capabilities, 
meaning that the necessary dependencies must be installed and configured correctly. 
The code also relies on Tauri for backend interactions, so a proper Tauri setup is essential to 
invoke functions like invoke('get_workspace_id'). Other preconditions include the presence of a 
valid workspace ID to manage notes effectively and the initial state setup for various variables 
such as savedNotes, title, and tags. Furthermore, user permissions are necessary for file handling 
when importing notes, and the saved notes must conform to a specific structure to ensure they render 
correctly in the UI. Meeting these preconditions is crucial for delivering a seamless user experience 
within the text editor application.

*Postconditions
After a user creates a new note, the postconditions include that the note's title 
and content are reset to their initial states, and any previously selected tags are 
cleared, ensuring a fresh start for the next note. When a note is saved, the expected 
postconditions involve confirming that the note is successfully stored in the application's 
state and reflected in the list of saved notes, along with a success notification displayed to 
the user. If a note is deleted, the postcondition is that the note is removed from 
both the saved notes list and the editor's current view, accompanied by a confirmation message 
indicating successful deletion. Additionally, when a file is imported, the content of that file 
should be accurately parsed and displayed in the editor, replacing any existing content. Overall, 
these postconditions ensure that the application behaves as intended and provides appropriate feedback 
to users after their actions, maintaining a seamless user experience throughout.

*Error and exception condition values or types that can occur, and their meanings

When loading the workspace, if the invoke function fails to retrieve the workspace ID, 
an error will be caught in the loadWorkspace function, resulting in a console error log and a 
toast notification indicating "Failed to load workspace." Similarly, when attempting to load saved notes, 
if there is an issue with fetching the notes from the backend, an error will be logged, and a toast 
notification will inform the user of the failure. During note saving, if there is an error in the save_note 
function call, it will throw an exception that is caught in the handleSaveNote function, leading to a console 
error log and a toast notification stating "Failed to save note." When deleting a note, if the remove_note function
fails to execute properly, it will also trigger an error message. Additionally, if there are issues reading a 
file during import (e.g., unsupported file format or read errors), appropriate error handling 
will log the issue and display a notification to inform the user. These conditions ensure that 
users are aware of any problems that arise during their interactions with the text editor.

*Side effects
State Updates: When users type in the title or edit 
content, the setTitle and setValue functions update the 
component's state, causing a re-render to reflect these 
changes in the UI.

Toast Notifications: Actions such as saving or 
deleting notes trigger toast notifications to 
inform users of success or failure. For example, 
a successful save will display a success message, 
while errors will show an error notification.

File Import: When a user selects a file to import, 
the handleFileChange function reads the file and 
updates the editor's content accordingly, replacing any 
existing content if the import is successful.


Tab Switching: Clicking on different tabs 
(e.g., "Saved Notes" or "Think Links") 
updates the activeTab state, resulting 
in a re-render that displays the appropriate 
content based on the selected tab.

Note Selection and Deletion: 
Selecting a note updates both the 
title and content displayed in the editor. 
If a note is deleted, it removes that note 
from the saved notes list and triggers confirmation 
dialogs or notifications.

Tag Management: Adding or removing tags through the 
TagsPanel component updates local state and 
affects how notes are categorized and displayed 
within the application.

• Invariants
State Integrity: The state variables 
(e.g., title, value, tags, savedNotes) 
must consistently reflect the current 
note's data. For instance, when a note is 
selected, the title and value states should
 accurately represent that note.

Valid Note Structure: Each note in the 
savedNotes array must maintain a consistent 
structure, containing properties such as title, 
content, and tags. This ensures uniform processing 
and rendering of notes.

Workspace ID Validity: The workspaceId
must always be a valid number. Operations 
that depend on this ID should not proceed 
if it is invalid (e.g., zero or undefined).

Editor Initialization: The editor 
should always start with a valid 
initial value defined by INITIAL_VALUE, 
ensuring there is a defined state for new notes.

Tag Consistency: The tags state must 
accurately reflect the tags associated 
with the currently selected note. Any 
changes to tags should update this 
state accordingly.

Cursor Position Accuracy: The tracked 
cursor position should always correspond 
to the current position within the editor's 
content, ensuring accurate user interactions.

Theme State Consistency: The darkMode 
state must consistently control the 
application's theme, reflecting user 
preferences throughout their interaction 
with the text editor.


*Error and exception condition values or types that can occur, and their meanings
Failed to Load Workspace: If the invoke('get_workspace_id') call fails, an error is logged, 
and a toast notification displays "Failed to load workspace," indicating 
that the workspace ID could not be retrieved.

Failed to Load Saved Notes: When calling 
invoke('files_in_workspace', { workspaceId }), /
if there is an issue fetching saved notes, an 
error is caught and logged, resulting in a 
toast notification saying "Failed to load 
saved notes," meaning notes associated with 
the current workspace could not be retrieved.

Failed to Save Note: In the 
handleSaveNote function, 
if invoke('save_note') encounters 
an error, it logs the error and 
shows a toast notification stating 
"Failed to save note," indicating 
that the note could not be saved.

Failed to Load Tags: If loading 
tags with invoke('get_tags', 
{ title: noteTitle, workspaceId }) 
 fails, it logs an error and displays 
 "Failed to load tags," meaning the 
 application could not retrieve tags 
 for the selected note.


 Failed to Read Note: When
 invoking read_note, 
 if there is a failure, 
 it logs an error and shows "Failed to load note," 
 indicating that the application could not read 
 the content of the specified note.


Failed to Delete Note: In the 
confirmDelete function, if invoke('remove_note') 
fails, it logs an error and displays "Failed to delete note," 
meaning that the deletion of the specified 
note was unsuccessful.

File Import Errors: If there are issues reading a 
file during import (e.g., unsupported format), 
errors will be logged, and users will receive 
notifications indicating that file import has failed.


*Any known faults
- The code may not handle all possible edge cases or error scenarios. 
*/


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
import { FaTrash, FaSun, FaMoon } from 'react-icons/fa';
import { Menu, MenuItem } from '@mui/material';

// Type definitions
type ParagraphElement = { type: 'paragraph'; children: CustomText[] }; // Add type definition for paragraph element
type HeadingElement = { type: 'heading'; children: CustomText[] }; // Add type definition for heading element
type CustomText = { text: string; bold?: boolean }; // Add type definition for custom text
type CustomElement = ParagraphElement | HeadingElement; // Add type definition for custom element
type CustomEditor = BaseEditor & ReactEditor & HistoryEditor; // Add type definition for custom editor
const threshold = 20;


/* The below code is declaring a module named 'slate' in TypeScript React. Within this module, it is
defining a custom interface called CustomTypes with three properties: Editor, Element, and Text.
Each of these properties is assigned a custom type: CustomEditor, CustomElement, and CustomText
respectively. This code is essentially extending the existing 'slate' module with custom types for
Editor, Element, and Text. */

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
const INITIAL_VALUE: Descendant[] = [ // Update the initial value to include a heading element
  {
    type: 'paragraph' as const, // Update the type to paragraph
    children: [{ text: '' }], // Update the text to an empty string
  },
];

const TextEditor: React.FC = () => { // Update the component to use the CustomTypes
  // State
  const [value, setValue] = useState<Descendant[]>(INITIAL_VALUE); // Update the initial value type
  const [title, setTitle] = useState<string>(''); // Add state for note title
  const [searchQuery, setSearchQuery] = useState<string>(''); // Add state for search query
  const [selectedTags, setSelectedTags] = useState<string[]>([]); // Add state for selected tags
  const [savedNotes, setSavedNotes] = useState<{ title: string, content: string, tags: { name: string, manual: boolean }[] }[]>([]); // Add state for saved notes
  const [workspaceId, setWorkspaceId] = useState<WorkspaceId>(0); // Use WorkspaceId type here
  const [tags, setTags] = useState<{ name: string, manual: boolean }[]>([]); // Add state for tags
  const [selectedNoteId, setSelectedNoteId] = useState<number | null>(null); // Add state for selected note ID
  const [showConfirmation, setShowConfirmation] = useState<boolean>(false); // Add state for confirmation dialog
  const [cursorPosition, setCursorPosition] = useState<number>(0); // Add state for cursor position
  const [activeTab, setActiveTab] = useState<string>('saved'); // Track active tab state
  const [contextResults, setContextResults] = useState<any[]>([]); // Add state for context results
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null); // Add state for menu anchor element
  const [darkMode, setDarkMode] = useState(true); // Default to dark mode
  const [isColorBlindMode, setIsColorBlindMode] = useState(false); // Add state for color-blind mode

  // Memoized values
  const editor = useMemo(() => withHistory(withReact(createEditor())), []); // Update the editor creation

  // Effects
  useEffect(() => {
    loadWorkspace(); // Load the workspace on initial render
  }, []);

  useEffect(() => {
    loadSavedState(); // Load the saved state on initial render
  }, []);

  useEffect(() => {
    if (title && value !== INITIAL_VALUE) {
      // This should call save_note instead.
    }
  }, [title, value]);

  useEffect(() => {
    console.log("Value changed:", value); // Log the value change
  }, [value]);

  useEffect(() => {
    console.log("Title changed:", title); // Log the title change
  }, [title]);

  useEffect(() => {
    // Pass the text content and cursor position to the side panel
    const textContent = Node.string(editor); // Get the text content
    console.log("Cursor position:", cursorPosition); // Log the cursor position
    // toast.info(`Cursor position: ${cursorPosition}`);
    console.log("Text content:", textContent); // Log the text content
    // Implement the logic to pass the information to the side panel here
  }, [cursorPosition, value]); // Update the dependencies
  
  useEffect(() => {
    document.body.className = darkMode ? 'dark-mode' : 'light-mode'; // Update the body class based on dark mode
  }, [darkMode]); // Update the dependency

  // Helper functions
  const loadWorkspace = async () => {
    try {
      const id = await invoke('get_workspace_id', { path: "./testing-workspace" }) as WorkspaceId; // Use WorkspaceId type here
      setWorkspaceId(id); // Set the workspace ID
      loadSavedNotes(id); // Load the saved notes for the workspace
    } catch (error) {
      console.error('Failed to load workspace:', error); // Log the error
      toast.error('Failed to load workspace'); // Show a toast notification
    }
  };

  const loadSavedNotes = async (workspaceId: WorkspaceId) => { // Use WorkspaceId type here
    try {
      const files = await invoke('files_in_workspace', { workspaceId }) as { document: {title: string, body: string}, tags: { name: string, manual: boolean }[] }[]; // Use WorkspaceId type here
      const notes = files.map((doc) => {
        return { title: doc.document.title, content: doc.document.body, tags: doc.tags }; // Update the note structure
      });
      setSavedNotes(notes); // Set the saved notes
      // set cursor to 0
      setCursorPosition(0); // Reset the cursor position
    } catch (error) {
      console.error('Failed to load saved notes:', error); // Log the error
      toast.error('Failed to load saved notes'); // Show a toast notification
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
      console.error('Failed to load saved state:', error); // Log the error
    }
  };

  const loadTags = async (noteTitle: string) => {
    try {
      const tags = await invoke('get_tags', { title: noteTitle, workspaceId }) as { name: string, manual: boolean }[]; // Use WorkspaceId type here
      setTags(tags); // Set the tags
    } catch (error) {
      console.error('Failed to load tags:', error); // Log the error
      toast.error('Failed to load tags'); // Show a toast notification
    }
  };

  // Event handlers
  const handleNewNote = () => {
    setTitle('');
    setValue(INITIAL_VALUE); // Reset the content
    setTags([]); // Clear the tags
    editor.children = INITIAL_VALUE; // Reset the editor content
    Transforms.select(editor, { path: [0, 0], offset: 0 }); // Reset the selection
    // toast.success('New note created'); 
  };

  const handleSaveNote = useCallback(async () => {
    // toast.info('Preparing to save note');
    const noteContent = JSON.stringify(value); // Convert the value to a string
    const note = {  // Create a note object 
      title: title || 'Untitled', // Use the title or 'Untitled' if no title is provided
      content: noteContent, // Set the content to the note content
      tags: [], // Initialize the tags to an empty array
    };
  
    try {
      console.log('Calling save_note function...'); // Log the save note action
  
      const parsedContent = JSON.parse(note.content); // Parse the content
      let extractedText = ''; // Initialize the extracted text
      parsedContent.forEach((block: any) => { // Iterate over the blocks
        if (block.type === 'paragraph') {   // Check if the block type is paragraph
          block.children.forEach((child: any) => {  // Iterate over the children
            extractedText += child.text + '\n'; // Append the text and a newline character
          });
        }
      });
      extractedText = extractedText.trim(); // Trim the extracted text
  
      await invoke('save_note', {   // Call the save_note function
        title: note.title,  // Pass the note title
        text: extractedText,    // Pass the extracted text
        workspaceId, // Pass the workspace ID
      });
  
      setSavedNotes(prevNotes => {
        const index = prevNotes.findIndex(n => n.title === title); // Find the index of the note
        if (index !== -1) {
          // Update existing note
          const updatedNotes = [...prevNotes]; // Create a copy of the notes
          updatedNotes[index] = { title: note.title, content: extractedText, tags: note.tags }; // Update the note
          return updatedNotes; // Return the updated notes
        } else { // If the note doesn't exist
          // Add new note
          return [...prevNotes, { title: note.title, content: extractedText, tags: note.tags }]; // Add the new note
        }
      });

      await loadTags(note.title); // Load the tags for the note
   
      toast.success(`Note saved successfully`); // Show a success toast
    } catch (error) {
      console.error('Failed to save note:', error);
      toast.error(`Failed to save note ${JSON.stringify(error)}`); // Show an error toast
    }
  }, [title, value, workspaceId]); // Update the dependencies

  const handleNoteSelect = useCallback(async (title: string) => {
    try {
      console.log("Attempting to read file:", title); // Log the file read action
      const content = await invoke('read_note', { title, workspaceId }) as { document: {title: string, body: string}, tags: { name: string, manual: boolean }[] }; // Use WorkspaceId type here
      console.log("Received content from backend:", content); // Log the content

      // Set the title
      setTitle(title); // Set the title

      // Create a new Slate-compatible value
      const newValue: Descendant[] = content.document.body.split('\n').map(line => { // Split the content by lines
        if (line.startsWith('#')) { //  Check if the line starts with a hash
          return {
            type: 'heading' as const,
            children: [{ text: line.slice(1).trim() }], // Trim the line and remove the hash
          };
        }
        return {
          type: 'paragraph' as const, // Set the type to paragraph
          children: [{ text: line }], // Set the text to the line
        };
      });

      // Update the editor's content
      setValue(newValue);
      editor.children = newValue;
      Transforms.select(editor, { path: [0, 0], offset: 0 }); // Select the first block

      await loadTags(title);

      // toast.success(`Loaded note: ${title}`);
    } catch (error) {
      console.error('Failed to load note:', error); // Log the error
      toast.error(`Failed to load note: ${error}`); // Show an error toast
    }
  }, [editor, setTitle, setValue, workspaceId]);

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => { // Add the event parameter
    if (event.metaKey && event.key === 'z') {
      event.preventDefault(); // Prevent the default behavior
      editor.undo(); // Call the undo function
      // toast.info('Undo action performed');
    } else if (event.metaKey && event.key === 's') { // Check if the key is Cmd/Ctrl + S
      event.preventDefault();
      handleSaveNote();
    }else if (event.metaKey && event.key === 'n') { // Check if the key is Cmd/Ctrl + N
      event.preventDefault();
      handleNewNote();
    } else if (event.metaKey && event.key === 'y') { // Check if the key is Cmd/Ctrl + Y
      event.preventDefault();
      editor.redo();
      // toast.info('Redo action performed');
    }
    

  };

  const handleTagClick = (tag: string) => { // Add the tag parameter
    setSelectedTags(prevTags => {
      if (prevTags.includes(tag)) { // Check if the tag is already selected
        return prevTags.filter(t => t !== tag); // Remove the tag if it's already selected
      } else {
        return [...prevTags, tag];  // Add the tag if it's not selected
      }
    });
    setSearchQuery(tag); // Update search query when a tag is clicked
  };

  const handleAddTag = async (newTag: string) => { // Add the newTag parameter
    try {
      await invoke('set_tags', {
        title,
        tags: [...tags, { name: newTag, manual: true }], // Add the new tag
        workspaceId,
      });
      await loadTags(title); // Reload the tags
    } catch (error) {
      console.error('Failed to add tag:', error); // Log the error
      toast.error('Failed to add tag'); // Show an error toast
    }
  };

  const handleDeleteClick = (noteId: number) => {
    setSelectedNoteId(noteId); // Set the selected note ID
    setShowConfirmation(true); // Show the confirmation dialog
  };

  const confirmDelete = async (noteId: number) => {
    try {
      console.log('Confirming delete for note ID:', noteId); // Log the confirmation 
      console.log('Deleting note with title:', savedNotes[noteId]?.title, 'Workspace ID:', workspaceId);
  
      // Call the Tauri backend
      const result = await invoke('remove_note', { title: savedNotes[noteId].title, workspaceId }); // Use WorkspaceId type here
      console.log('Backend response:', result);
  
      // Update the state
      setSavedNotes(prevNotes => {
        const updatedNotes = prevNotes.filter((_, index) => index !== noteId); // Filter out the note to delete
        console.log('Updated notes:', updatedNotes); 
        setTitle(''); // Clear the title
        setTags([]);
        setValue(INITIAL_VALUE); // Clear the content
        editor.children = INITIAL_VALUE; // Reset the editor content
        Transforms.select(editor, { path: [0, 0], offset: 0 }); 
        return updatedNotes;
      });
  
      setSelectedNoteId(null); // Clear the selected note ID
      setShowConfirmation(false); // Hide the confirmation dialog
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

  const handleMenuClick = (event: React.MouseEvent<HTMLElement>) => { 
    setAnchorEl(event.currentTarget);
  };

  const handleMenuClose = () => { 
    setAnchorEl(null);
  };
 
  const toggleTheme = () => { 
    setDarkMode(!darkMode);
  };

  const toggleColorBlindMode = () => { 
    setIsColorBlindMode(!isColorBlindMode);
  };

  // File change handler
  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => { 
    const file = event.target.files?.[0]; // Get the selected file
    if (file) {
      const reader = new FileReader(); // Create a new FileReader
      reader.onload = async (e) => {
        const text = e.target?.result as string;
        const newValue: Descendant[] = text.split('\n').map(line => ({ 
          type: 'paragraph' as const, // Set the type to paragraph
          children: [{ text: line }],
        }));
        setValue(newValue); // Update the value
        editor.children = newValue;
        Transforms.select(editor, { path: [0, 0], offset: 0 }); // Select the first block
      };
      reader.readAsText(file); // Read the file as text
    }
  };

  // Event handler to update cursor position
  const handleCursorPosition = () => { 
    const { selection } = editor; // Get the selection
    if (selection) {
      const { anchor } = selection; // Get the anchor point
      let cursorIndex = 0;

      // Traverse the nodes to calculate the cursor position
      for (let i = 0; i < anchor.path[0]; i++) { 
        const node = editor.children[i];
        cursorIndex += Node.string(node).length; 
      }
      cursorIndex += anchor.offset;

      // only update the cursor position if it has changed by a threshold value
      if (Math.abs(cursorIndex - cursorPosition) > threshold) {
        setCursorPosition(cursorIndex); // Update the cursor position
        getContextResult(cursorIndex, Node.string(editor));
      }
      

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
    const contextResults = await invoke('context_search', { 
      documentText: document_text,
      cursorUtf16Index: cursor_utf16_index,
      results: 3, // Adjust the number of results as needed
      contextSentences: 2, // Adjust the number of context sentences as needed
      workspaceId: workspaceId,
      documentTitle,
    }) as { distance: number, title: string, relevant_range: string, text: string }[]; // Use WorkspaceId type here
    console.log("Received context result:", contextResults); // Log the context results
    // Sort results by distance 
    const sortedResults = contextResults.sort((a: any, b: any) => a.distance - b.distance); // Sort the results
    setContextResults(sortedResults); // Set the context results
    // You can now display the context result (e.g., in a toast or sidebar)
  } catch (error) {
    console.error("Failed to get context:", error); // Log the error
    toast.error(`Failed to get context ${error}`); // Show an error toast
  }
  };
  
  const renderContextResults = () => {
  return contextResults
    .sort((a, b) => a.distance - b.distance) // Sort by distance
    .map((result, index) => {
      const { title, text, relevant_range } = result; // Destructure the result
      const beforeRelevant = text.slice(0, relevant_range.start);
      const relevantText = text.slice(relevant_range.start, relevant_range.end); // Extract the relevant text
      const afterRelevant = text.slice(relevant_range.end); // Extract the text after the relevant part
      
      return ( 
        <div key={index} className="context-result"> 
          <div className="note-content"> 
            <strong>{title}</strong> 
            <hr className="title-divider" /> 
            <p>
              {beforeRelevant}
              <strong>{relevantText}</strong>
              {afterRelevant}
            </p>
          </div>
        </div>
      );
    });
};
  // Render functions
  const renderElement = useCallback((props: RenderElementProps) => {
    switch (props.element.type) { 
      case 'heading':
        return <h1 {...props.attributes}>{props.children}</h1>; // Render a heading element
      case 'paragraph':
      default:
        return <p {...props.attributes}>{props.children}</p>; // Render a paragraph element
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
        <button onClick={() => document.getElementById('fileInput')?.click()} className="import-btn">Import</button> 
        <button onClick={handleMenuClick} className="menu-btn">⋮</button> 
        <Menu
          anchorEl={anchorEl} // Set the anchor element
          keepMounted // Keep the menu mounted
          open={Boolean(anchorEl)} // Check if the anchor element is present
          onClose={handleMenuClose} // Handle the menu close event
        >
          <MenuItem onClick={toggleTheme}>
            {darkMode ? <FaSun /> : <FaMoon />} {darkMode ? 'Light Mode' : 'Dark Mode'} 
          </MenuItem>
          <MenuItem onClick={toggleColorBlindMode}>
            {isColorBlindMode ? 'Disable Accessibility' : 'Enable Accessibility'} 
          </MenuItem>
        </Menu>
        <input
          type="file"
          id="fileInput"
          style={{ display: 'none' }} // Hide the file input
          accept=".txt"
          onChange={handleFileChange} // Handle the file change event
        />
      </div>
      <div className="editor-body">
        <div className="sidepanel">
        <div className="sidepanel-tabs">
            <div
              className={`sidepanel-tab ${activeTab === 'saved' ? 'active' : ''}`} // Add a class for the active tab
              onClick={() => handleTabClick('saved')} // Handle the tab click event
            >
              Saved Notes
            </div>
            <div
              className={`sidepanel-tab ${activeTab === 'other' ? 'active' : ''}`} // Add a class for the active tab
              onClick={() => handleTabClick('other')} // Handle the tab click event
            >
              Think Links
            </div>
          </div>
          <div className="sidepanel-content" id="saved-notes" style={{ display: activeTab === 'saved' ? 'block' : 'none'}}> 
          {savedNotes.length > 0 ? (
            <ul>
              {savedNotes.map((note, index) => (
                <li key={index} className="note-item" onClick={() => handleNoteSelect(note.title)}> 
                <div className="note-content">
                    <strong>{note.title}</strong> 
                    <hr className="title-divider" /> 
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
        <div className="sidepanel-content" id="other-panel" style={{ display: activeTab === 'other' ? 'block' : 'none', overflowY: 'auto', maxHeight: 'calc(100vh - 100px)' }}> 
          {renderContextResults()} 
        </div>
        </div>
        <div className="editor-content"> 
          <div className="editor-input"> 
            <div className="title-tags-container"> 
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)} // Handle the title change event
                placeholder="Title"
                className="title-input"
              />
              <TagsPanel onTagClick={handleTagClick} tags={tags} onAddTag={handleAddTag} isColorBlindMode={isColorBlindMode}/> 
            </div>
          </div>
          <Slate 
            editor={editor} 
            initialValue={value} 
            onChange={(newValue) => setValue(newValue)} // Handle the value change event
          >
            <Editable
              renderElement={renderElement} // Add the renderElement function
              placeholder="Start typing your note here..." // Add a placeholder
              className="editable"
              onKeyDown={handleKeyDown} // Add the onKeyDown event
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