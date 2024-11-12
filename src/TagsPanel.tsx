import React, { useState, useEffect, useRef } from 'react';
import Tag from './Tag';

interface TagsPanelProps {
  onTagClick: (tag: string) => void;
  tags: { name: string, manual: boolean }[];
  onAddTag: (tag: string) => void;
}

const TagsPanel: React.FC<TagsPanelProps> = ({ onTagClick, tags, onAddTag }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [customTag, setCustomTag] = useState('');
  const menuRef = useRef<HTMLDivElement>(null);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  const handleClickOutside = (event: MouseEvent) => {
    if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
      setIsMenuOpen(false);
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      setIsMenuOpen(false);
    }
  };

  useEffect(() => {
    if (isMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isMenuOpen]);

  const handleCustomTagInput = (event: React.ChangeEvent<HTMLInputElement>) => {
    setCustomTag(event.target.value);
  };

  const addCustomTag = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter' && customTag.trim() !== '') {
      onAddTag(customTag);
      setCustomTag('');
      setIsMenuOpen(false);
    }
  };

  // Extract only manual tags and limit the displayed tags to the last 5 added
  const displayedTags = tags.slice(-5); // Only the last 5 tags are displayed
  const remainingTags = tags.slice(0, -5); // Older tags go into the dropdown menu

  return (
    <div className="tags-panel">
      <div className="tags-list">
        {displayedTags.map((tag, index) => (
          <Tag key={index} title={tag.name} colorClass={`tag-color-${(index % 5) + 1}`} onClick={() => onTagClick(tag.name)} />
        ))}
      </div>
      <div className="menu-dots" onClick={toggleMenu}>⋮</div>
      {isMenuOpen && (
        <div className="dropdown-menu" ref={menuRef}>
          <input
            type="text"
            className="custom-tag-input"
            placeholder="Add a custom tag"
            value={customTag}
            onChange={handleCustomTagInput}
            onKeyDown={addCustomTag}
          />
          {remainingTags.map((tag, index) => (
            <Tag key={index} title={tag.name} colorClass={`tag-color-${(index % 5) + 1}`} onClick={() => onTagClick(tag.name)} />
          ))}
        </div>
      )}
    </div>
  );
};

export default TagsPanel;