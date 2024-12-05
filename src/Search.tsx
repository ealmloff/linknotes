import React, { useState, useEffect, useRef } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

// Define the WorkspaceId type if it's not already defined
type WorkspaceId = number; // Adjust this based on the actual type

interface SearchProps {
  onTagClick: (tag: string) => void;
  selectedTags: string[];
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  workspace_id: WorkspaceId; // Use WorkspaceId type here
  handleNoteSelect: (title: string) => void; // Add handleNoteSelect as a prop
}

interface SearchResult {
  distance: number;
  title: string;
  character_range: [number, number];
}

const Search: React.FC<SearchProps> = ({ onTagClick, selectedTags, searchQuery, setSearchQuery, workspace_id, handleNoteSelect }) => {
  const [isDropdownOpen, setIsDropdownOpen] = useState<boolean>(false);
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const searchRef = useRef<HTMLDivElement>(null);

  const handleSearchChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(event.target.value);
    setIsDropdownOpen(true);
  };

  const handleTagRemove = (tag: string) => {
    onTagClick(tag);
  };

  const handleClickOutside = (event: MouseEvent) => {
    if (searchRef.current && !searchRef.current.contains(event.target as Node)) {
      setIsDropdownOpen(false);
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      setIsDropdownOpen(false);
    }
  };

  const performSearch = async (query: string, tags: string[]) => {
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

  useEffect(() => {
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  useEffect(() => {
    if (searchQuery) {
      setIsDropdownOpen(true);
    }
  }, [searchQuery, selectedTags]);

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