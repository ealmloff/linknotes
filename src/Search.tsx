import React, { useState, useEffect, useRef } from 'react';

interface SearchProps {
  onTagClick: (tag: string) => void;
  selectedTags: string[];
  searchQuery: string;
  setSearchQuery: (query: string) => void;
}

const Search: React.FC<SearchProps> = ({ onTagClick, selectedTags, searchQuery, setSearchQuery }) => {
  const [isDropdownOpen, setIsDropdownOpen] = useState<boolean>(false);
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
  }, [searchQuery]);

  return (
    <div className="search-container" ref={searchRef}>
      <input
        type="text"
        placeholder="Search..."
        value={searchQuery}
        onChange={handleSearchChange}
        className="search-input"
        onFocus={() => setIsDropdownOpen(true)}
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
            {/* Display search results here */}
          </div>
        </div>
      )}
    </div>
  );
};

export default Search;