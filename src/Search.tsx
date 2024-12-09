/**
 * Search.tsx
 * 
 * Brief Description:
 * This component provides a search bar with tag-based filtering functionality. 
 * It dynamically updates search results based on user input and displays a dropdown 
 * with suggestions, allowing users to select tags and view associated results.
 * 
 * Programmers: Suhaan, Trisha, Teja, Siddh
 * Created: 10/31/2024
 * Revised: 10/31/2024 -> Added handleNoteSelect as a prop
 * 
 * Preconditions:
 * - `workspace_id` must be a valid identifier for the workspace.
 * - `onTagClick`, `selectedTags`, `searchQuery`, `setSearchQuery`, and `handleNoteSelect` must be properly passed as props.
 * - `invoke` function must be properly configured to call the backend search function.
 * 
 * Postconditions:
 * - The component updates `searchQuery` and displays relevant search results in a dropdown.
 * - Selected tags and search input influence the search results displayed.
 * 
 * Error and Exception Conditions:
 * - Errors during the `invoke` call are logged to the console and displayed as toast notifications.
 * - If event listeners for clicks or keypresses fail, unexpected behavior in dropdown functionality may occur.
 * 
 * Side Effects:
 * - Adds `mousedown` and `keydown` event listeners to the document on mount.
 * - Removes event listeners on unmount.
 * 
 * Invariants:
 * - The dropdown should only be open when there is an active search query or the input is focused.
 * - Duplicate search results are filtered out before display.
 * 
 * Known Faults:
 * - Possible performance issues if the search query is very large due to frequent invocations of `performSearch`.
 */

/* The `import` statements in the code snippet are used to import specific functionalities and
libraries into the `Search.tsx` file. Here's what each import statement is doing: */
import React, { useState, useEffect, useRef } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

// Define the WorkspaceId type if it's not already defined
type WorkspaceId = number; // Adjust this based on the actual type

/* The `interface SearchProps` is defining the props that the `Search` component expects to receive.
Here's a breakdown of each prop: */
interface SearchProps {
  onTagClick: (tag: string) => void; // Callback for removing a selected tag
  selectedTags: string[];           // Array of currently selected tags
  searchQuery: string;              // Current search input value
  setSearchQuery: (query: string) => void; // Setter for updating search query
  workspace_id: WorkspaceId;        // Identifier for the workspace
  handleNoteSelect: (title: string) => void; // Callback when a search result is selected
}

/* The `interface SearchResult` is defining the structure of an object that represents a search result.
Here's a breakdown of each property within the `SearchResult` interface: */
interface SearchResult {
  distance: number;               // Relevance distance of the search result
  title: string;                  // Title of the result
  character_range: [number, number]; // Range of matching characters
}

/* The `const Search: React.FC<SearchProps> = ({ ... }) => { ... }` block of code defines the
functional component `Search` in TypeScript React. Here's a breakdown of what each part of the
component does: */
const Search: React.FC<SearchProps> = ({ onTagClick, selectedTags, searchQuery, setSearchQuery, workspace_id, handleNoteSelect }) => {
  const [isDropdownOpen, setIsDropdownOpen] = useState<boolean>(false); // Dropdown visibility state
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]); // Array of search results
  const searchRef = useRef<HTMLDivElement>(null); // Reference to the search container

  // Handler for changes in the search input
  /**
   * The function `handleSearchChange` updates the search query based on user input and opens a
   * dropdown menu.
   * @param event - The `event` parameter in the `handleSearchChange` function is of type
   * `React.ChangeEvent<HTMLInputElement>`. This means it is an event object that is triggered when the
   * value of an input element (in this case, a text input field) changes.
   */
  const handleSearchChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(event.target.value);
    setIsDropdownOpen(true);
  };

  // Handler for removing a tag
  /**
   * The `handleTagRemove` function calls the `onTagClick` function with the provided tag as an
   * argument.
   * @param {string} tag - The `handleTagRemove` function takes a `tag` parameter, which is a string
   * representing the tag that needs to be removed. This function then calls the `onTagClick` function
   * with the `tag` parameter to handle the removal of the tag.
   */
  const handleTagRemove = (tag: string) => {
    onTagClick(tag);
  };

  // Closes the dropdown if a click occurs outside the search container
  /**
   * The handleClickOutside function checks if a click event occurs outside a specified element and
   * closes a dropdown if it does.
   * @param {MouseEvent} event - The `event` parameter in the `handleClickOutside` function is of type
   * `MouseEvent`, which represents a mouse event that occurs when a user interacts with the webpage
   * using a mouse device.
   */
  const handleClickOutside = (event: MouseEvent) => {
    if (searchRef.current && !searchRef.current.contains(event.target as Node)) {
      setIsDropdownOpen(false);
    }
  };

  // Closes the dropdown on pressing the Escape key
  /**
   * The handleKeyDown function checks if the 'Escape' key is pressed and closes a dropdown if it is.
   * @param {KeyboardEvent} event - The `event` parameter in the `handleKeyDown` function is of type
   * `KeyboardEvent`, which represents an event that occurs when a key is pressed on the keyboard. In
   * this case, the function is checking if the key that was pressed is the 'Escape' key and then
   * setting the `is
   */
  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      setIsDropdownOpen(false);
    }
  };

  // Handler for selecting a note from the search results
  /**
   * The function `performSearch` takes a query and tags as input, performs a search using an external
   * API, removes duplicate results based on title, and updates the search results state.
   * @param {string} query - The `query` parameter in the `performSearch` function is a string that
   * represents the search query or keywords that will be used to search for results. It is the text
   * that the search function will use to find relevant information.
   * @param {string[]} tags - The `tags` parameter in the `performSearch` function is an array of
   * strings that represent the tags associated with the search query. These tags are used to filter
   * the search results and provide more relevant information to the user.
   */
  const performSearch = async (query: string, tags: string[]) => {
    /* The code snippet you provided is a `try-catch` block within the `performSearch` function in the
    `Search.tsx` file. Here's a breakdown of what it does: */
    try {
      const results = await invoke('search', {
        text: query,
        tags: tags,
        results: 10,
        workspaceId: workspace_id // Use the actual workspace ID
      }) as SearchResult[];

      // Remove duplicate titles
      const uniqueResults = results.filter((result, index, self) =>
        index === self.findIndex((r) => r.title === result.title)
      );

      setSearchResults(uniqueResults);
    } catch (error) {
      console.error('Failed to perform search:', error);
      toast.error(`Failed to perform search ${error}`);
    }
  };

  /* The `useEffect` hook you provided is responsible for adding event listeners for mouse clicks and
  keydown events when the component mounts, and removing those event listeners when the component
  unmounts. Here's a breakdown of what it does: */
  useEffect(() => {
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  /* The `useEffect` hook you provided is responsible for triggering a side effect when certain
  dependencies change. In this case, the effect is triggered whenever `searchQuery` or
  `selectedTags` change. */
  useEffect(() => {
    if (searchQuery) {
      setIsDropdownOpen(true);
    }
  }, [searchQuery, selectedTags]);

  /* The `return` statement in the `Search` component is responsible for rendering the JSX (JavaScript
  XML) elements that make up the search functionality UI. Here's a breakdown of what each part of
  the `return` block is doing: */
  return (
    <div className="search-container" ref={searchRef}>
      <input
        type="text"
        placeholder="Search..."
        value={searchQuery}
        onChange={handleSearchChange}
        className="search-input"
        onFocus={() => setIsDropdownOpen(true)}
        onKeyDown={() => {
          performSearch(searchQuery, selectedTags);
        }}
      />
      {isDropdownOpen && (
        <div className="search-dropdown">
          <div className="search-queries">
            {selectedTags.map((tag, index) => (
              <div key={index} className="search-tag" onClick={() => handleTagRemove(tag)}>
                {tag}
              </div>
            ))}
          </div>
          <div className="search-results">
            {searchResults.map((result, index) => (
              <div key={index} className="search-result" onClick={() => handleNoteSelect(result.title)}>
                {result.title}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default Search;