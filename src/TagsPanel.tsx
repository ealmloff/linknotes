import React, { useState, useEffect, useRef } from 'react';
import Tag from './Tag';

interface TagsPanelProps {
  onTagClick: (tag: string) => void;
}

const TagsPanel: React.FC<TagsPanelProps> = ({ onTagClick }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [tags, setTags] = useState<string[]>(['Tag1', 'Tag2', 'Tag3', 'Tag4', 'Tag5']);
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
      setTags([customTag, ...tags]);
      setCustomTag('');
      setIsMenuOpen(false);
    }
  };

  return (
    <div className="tags-panel">
      <div className="tags-list">
        {tags.map((tag, index) => (
          <Tag key={index} title={tag} colorClass={`tag-color-${(index % 5) + 1}`} onClick={() => onTagClick(tag)} />
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
        </div>
      )}
    </div>
  );
};

export default TagsPanel;