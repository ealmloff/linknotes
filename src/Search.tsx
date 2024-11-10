// Search.tsx
import React from 'react';

const Search: React.FC = () => {
  const handleSearch = () => {
    console.log("Search function triggered");
  };

  return (
    <input
      type="text"
      placeholder="Search..."
      onChange={handleSearch}
      className="search-input"
    />
  );
};

export default Search;
